//! Nom-based IRC message parser.
//!
//! This module provides zero-copy parsing of IRC messages using the nom
//! parser combinator library.

use nom::{
    bytes::complete::{take_until, take_while1},
    character::complete::{char, space0},
    combinator::opt,
    error::ErrorKind,
    sequence::preceded,
    IResult,
};

/// Parse IRCv3 message tags (the part after `@` and before the first space).
fn parse_tags(input: &str) -> IResult<&str, &str> {
    preceded(char('@'), take_until(" "))(input)
}

/// Parse message prefix (the part after `:` and before the first space).
fn parse_prefix(input: &str) -> IResult<&str, &str> {
    preceded(char(':'), take_while1(|c| c != ' '))(input)
}

/// Parse the command name (alphanumeric characters).
fn parse_command(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric())(input)
}

/// Parse IRC message parameters from the remaining input after the command.
///
/// Handles both regular space-separated parameters and the trailing parameter
/// (prefixed with `:`) which may contain spaces. Multiple consecutive spaces
/// are treated as a single separator (RFC compliance).
fn parse_params(input: &str) -> (&str, Vec<&str>) {
    let mut params: Vec<&str> = Vec::new();
    let mut rest = input;

    while let Some(b' ') = rest.as_bytes().first().copied() {
        // Skip all leading spaces (handles multiple consecutive spaces)
        while rest.as_bytes().first() == Some(&b' ') {
            rest = &rest[1..];
        }

        // Check if we've reached the end after skipping spaces
        if rest.is_empty() || rest.starts_with('\r') || rest.starts_with('\n') {
            break;
        }

        if let Some(b':') = rest.as_bytes().first().copied() {
            // Trailing parameter - everything after `:` until line end
            let after_colon = &rest[1..];
            let end = after_colon.find(['\r', '\n']).unwrap_or(after_colon.len());
            params.push(&after_colon[..end]);
            rest = &after_colon[end..];
            break;
        }

        // Regular parameter - until next space or line end
        let end = rest.find([' ', '\r', '\n']).unwrap_or(rest.len());
        let param = &rest[..end];
        if param.is_empty() {
            break;
        }
        params.push(param);
        rest = &rest[end..];
    }

    (rest, params)
}

/// Parse a complete IRC message into its components.
///
/// IRC message format:
/// ```text
/// [@tags] [:prefix] <command> [params...] [:trailing]
/// ```
pub(crate) fn parse_message(input: &str) -> IResult<&str, ParsedMessage<'_>> {
    // Parse optional tags
    let (input, tags) = opt(parse_tags)(input)?;
    let (input, _) = space0(input)?;

    // Parse optional prefix
    let (input, prefix) = opt(parse_prefix)(input)?;
    let (input, _) = space0(input)?;

    // Parse command (required)
    let (input, command) = parse_command(input)?;

    // Parse parameters (including trailing)
    let (rest, params) = parse_params(input);

    Ok((
        rest,
        ParsedMessage {
            tags,
            prefix,
            command,
            params,
        },
    ))
}

/// A parsed IRC message with borrowed string slices.
///
/// This is the intermediate representation produced by the nom parser.
/// It holds references into the original input string for zero-copy parsing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedMessage<'a> {
    /// Raw tags string (without the leading `@`), if present.
    pub tags: Option<&'a str>,
    /// Raw prefix string (without the leading `:`), if present.
    pub prefix: Option<&'a str>,
    /// The command name.
    pub command: &'a str,
    /// Command parameters, including trailing.
    pub params: Vec<&'a str>,
}

impl<'a> ParsedMessage<'a> {
    /// Parse an IRC message string into a `ParsedMessage`.
    ///
    /// This is the primary entry point for parsing borrowed messages.
    /// Returns detailed error information for debugging failed parses.
    pub fn parse(input: &'a str) -> Result<Self, DetailedParseError> {
        match parse_message(input) {
            Ok((_remaining, msg)) => Ok(msg),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let position = input.len() - e.input.len();
                Err(DetailedParseError {
                    input: input.to_string(),
                    position,
                    kind: e.code,
                })
            }
            Err(nom::Err::Incomplete(_)) => Err(DetailedParseError {
                input: input.to_string(),
                position: input.len(),
                kind: ErrorKind::Eof,
            }),
        }
    }
}

/// Detailed parse error with position information.
#[derive(Debug, Clone)]
pub(crate) struct DetailedParseError {
    /// The original input string that failed to parse.
    pub input: String,
    /// Character position where parsing failed.
    pub position: usize,
    /// The nom error kind.
    pub kind: ErrorKind,
}

impl std::fmt::Display for DetailedParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at position {}: {:?}",
            self.position, self.kind
        )?;

        // Show the error position in the input
        if self.position < self.input.len() {
            let before = &self.input[..self.position];
            let after = &self.input[self.position..];
            write!(f, "\n  Input: {}<<<HERE>>>{}", before, after)?;
        } else {
            write!(f, "\n  Input: {}<<<EOF>>>", self.input)?;
        }

        Ok(())
    }
}

impl std::error::Error for DetailedParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let msg = ParsedMessage::parse("PING").unwrap();
        assert_eq!(msg.command, "PING");
        assert!(msg.tags.is_none());
        assert!(msg.prefix.is_none());
        assert!(msg.params.is_empty());
    }

    #[test]
    fn test_parse_command_with_params() {
        let msg = ParsedMessage::parse("PRIVMSG #channel :Hello, world!").unwrap();
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#channel", "Hello, world!"]);
    }

    #[test]
    fn test_parse_with_prefix() {
        let msg = ParsedMessage::parse(":nick!user@host PRIVMSG #channel :Hello").unwrap();
        assert_eq!(msg.prefix, Some("nick!user@host"));
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#channel", "Hello"]);
    }

    #[test]
    fn test_parse_with_tags() {
        let msg = ParsedMessage::parse("@time=2023-01-01T00:00:00Z :nick PRIVMSG #ch :Hi").unwrap();
        assert_eq!(msg.tags, Some("time=2023-01-01T00:00:00Z"));
        assert_eq!(msg.prefix, Some("nick"));
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#ch", "Hi"]);
    }

    #[test]
    fn test_parse_with_crlf() {
        let msg = ParsedMessage::parse("PING :server\r\n").unwrap();
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params, vec!["server"]);
    }

    #[test]
    fn test_parse_multiple_params() {
        let msg = ParsedMessage::parse("USER guest 0 * :Real Name").unwrap();
        assert_eq!(msg.command, "USER");
        assert_eq!(msg.params, vec!["guest", "0", "*", "Real Name"]);
    }

    #[test]
    fn test_parse_numeric_response() {
        let msg = ParsedMessage::parse(":server 001 nick :Welcome").unwrap();
        assert_eq!(msg.prefix, Some("server"));
        assert_eq!(msg.command, "001");
        assert_eq!(msg.params, vec!["nick", "Welcome"]);
    }

    #[test]
    fn test_parse_join() {
        let msg = ParsedMessage::parse(":nick!user@host JOIN #channel").unwrap();
        assert_eq!(msg.command, "JOIN");
        assert_eq!(msg.params, vec!["#channel"]);
    }

    #[test]
    fn test_parse_empty_trailing() {
        let msg = ParsedMessage::parse("PRIVMSG #channel :").unwrap();
        assert_eq!(msg.params, vec!["#channel", ""]);
    }

    #[test]
    fn test_parse_complex_tags() {
        let msg =
            ParsedMessage::parse("@msgid=abc123;time=2023-01-01 :nick PRIVMSG #ch :msg").unwrap();
        assert_eq!(msg.tags, Some("msgid=abc123;time=2023-01-01"));
    }
}

//! Nom-based IRC message parser.
//!
//! This module provides zero-copy parsing of IRC messages using the nom
//! parser combinator library.

use nom::{
    bytes::complete::{take_until, take_while1},
    character::complete::{char, space0},
    combinator::opt,
    error::{context, ErrorKind, VerboseError},
    sequence::preceded,
    IResult,
};

type ParseResult<I, O> = IResult<I, O, VerboseError<I>>;

/// Parse IRCv3 message tags (the part after `@` and before the first space).
fn parse_tags(input: &str) -> ParseResult<&str, &str> {
    context(
        "parsing IRCv3 message tags",
        preceded(char('@'), take_until(" "))
    )(input)
}

/// Parse message prefix (the part after `:` and before the first space).
fn parse_prefix(input: &str) -> ParseResult<&str, &str> {
    context(
        "parsing message prefix",
        preceded(char(':'), take_while1(|c| c != ' '))
    )(input)
}

/// Parse the command name (alphanumeric characters).
fn parse_command(input: &str) -> ParseResult<&str, &str> {
    context(
        "parsing IRC command",
        take_while1(|c: char| c.is_alphanumeric())
    )(input)
}

/// Parse a complete IRC message into its components.
///
/// IRC message format:
/// ```text
/// [@tags] [:prefix] <command> [params...] [:trailing]
/// ```
pub fn parse_message(input: &str) -> ParseResult<&str, ParsedMessage<'_>> {
    // Parse optional tags
    let (input, tags) = context("parsing optional tags", opt(parse_tags))(input)?;
    let (input, _) = space0(input)?;

    // Parse optional prefix  
    let (input, prefix) = context("parsing optional prefix", opt(parse_prefix))(input)?;
    let (input, _) = space0(input)?;

    // Parse command (required)
    let (input, command) = context("parsing required command", parse_command)(input)?;

    // Parse parameters (including trailing)
    let mut params: Vec<&str> = Vec::new();
    let mut rest = input;

    while let Some(b' ') = rest.as_bytes().first().copied() {
        // Skip the space
        rest = &rest[1..];
        
        if let Some(b':') = rest.as_bytes().first().copied() {
            // Trailing parameter - everything after `:` until line end
            let after_colon = &rest[1..];
            let end = after_colon
                .find(['\r', '\n'])
                .unwrap_or(after_colon.len());
            let trailing = &after_colon[..end];
            params.push(trailing);
            rest = &after_colon[end..];
            break;
        } else {
            // Regular parameter - until next space or line end
            let mut end = rest.len();
            if let Some(i) = rest.find(' ') {
                end = end.min(i);
            }
            if let Some(i) = rest.find('\r') {
                end = end.min(i);
            }
            if let Some(i) = rest.find('\n') {
                end = end.min(i);
            }
            let param = &rest[..end];
            if param.is_empty() {
                break;
            }
            params.push(param);
            rest = &rest[end..];
        }
    }

    Ok((rest, ParsedMessage {
        tags,
        prefix,
        command,
        params,
    }))
}

/// A parsed IRC message with borrowed string slices.
///
/// This is the intermediate representation produced by the nom parser.
/// It holds references into the original input string for zero-copy parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMessage<'a> {
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
                // Find the innermost error with context
                let mut context_info = None;
                let mut position = input.len();
                let mut kind = ErrorKind::Tag;
                
                for (error_input, error_kind) in &e.errors {
                    position = input.len() - error_input.len();
                    match error_kind {
                        nom::error::VerboseErrorKind::Context(ctx) => {
                            context_info = Some(*ctx);
                        }
                        nom::error::VerboseErrorKind::Nom(ek) => {
                            kind = *ek;
                        }
                        nom::error::VerboseErrorKind::Char(_) => {
                            kind = ErrorKind::Char;
                        }
                    }
                }
                
                Err(DetailedParseError {
                    input: input.to_string(),
                    position,
                    context: context_info,
                    kind,
                })
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(DetailedParseError {
                    input: input.to_string(),
                    position: input.len(),
                    context: Some("incomplete input"),
                    kind: ErrorKind::Eof,
                })
            }
        }
    }
}

/// Detailed parse error with position and context information.
#[derive(Debug, Clone)]
pub struct DetailedParseError {
    /// The original input string that failed to parse.
    pub input: String,
    /// Character position where parsing failed.
    pub position: usize,
    /// Context about what was being parsed when the error occurred.
    pub context: Option<&'static str>,
    /// The nom error kind.
    pub kind: ErrorKind,
}

impl std::fmt::Display for DetailedParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at position {}", self.position)?;
        if let Some(ctx) = self.context {
            write!(f, " while {}", ctx)?;
        }
        write!(f, ": {:?}", self.kind)?;
        
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
        let msg = ParsedMessage::parse("@msgid=abc123;time=2023-01-01 :nick PRIVMSG #ch :msg").unwrap();
        assert_eq!(msg.tags, Some("msgid=abc123;time=2023-01-01"));
    }
}


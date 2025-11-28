//! IRC control character validation utilities.
//!
//! This module provides comprehensive validation for IRC protocol elements,
//! including nicknames, usernames, hostnames, channel names, and general
//! control character detection.
//!
//! # Protocol-Level Control Characters
//!
//! At the transport layer, IRC rejects most control characters. The pattern
//! used in the transport layer is:
//! ```text
//! ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n')
//! ```
//!
//! This means:
//! - NUL (0x00) is always rejected
//! - All C0 control characters (0x01-0x1F) except CR (0x0D) and LF (0x0A) are rejected
//! - CR/LF are allowed as line delimiters
//!
//! Use [`is_illegal_control_char`] to check individual characters with this pattern.
//!
//! # Formatting Control Characters
//!
//! IRC clients commonly use certain control characters for text formatting:
//! - 0x02: Bold
//! - 0x03: Color (followed by color codes)
//! - 0x04: Hex color (IRCv3)
//! - 0x0F: Reset formatting
//! - 0x11: Monospace (IRCv3)
//! - 0x16: Reverse/Inverse
//! - 0x1D: Italic
//! - 0x1F: Underline
//!
//! Note: While these are commonly used, they ARE rejected by the transport
//! layer's control character validation. The [`colors`](crate::colors) module
//! provides utilities for stripping these codes from strings.
//!
//! # Protocol Element Validation
//!
//! Certain protocol elements have additional requirements:
//! - Nicknames: No spaces, control chars, or special IRC chars
//! - Channel names: Must start with #, &, +, or ! and have no spaces/commas
//! - Usernames: No spaces, NUL, CR, LF, or @

use std::borrow::Cow;

/// Control characters that are never valid in IRC messages.
///
/// These characters terminate or delimit IRC protocol lines.
pub const PROTOCOL_CONTROL_CHARS: &[char] = &[
    '\x00', // NUL - terminates strings
    '\x0D', // CR - line delimiter
    '\x0A', // LF - line delimiter
];

/// IRC formatting control characters.
///
/// These are valid in message content but represent formatting, not text.
pub const FORMAT_CONTROL_CHARS: &[char] = &[
    '\x02', // Bold
    '\x03', // Color (mIRC)
    '\x04', // Hex color (IRCv3)
    '\x0F', // Reset
    '\x11', // Monospace (IRCv3)
    '\x16', // Reverse
    '\x1D', // Italic
    '\x1F', // Underline
];

/// Characters that are invalid in channel names per RFC 2812.
const INVALID_CHAN_CHARS: &[char] = &[' ', ',', '\x07', '\x00'];

/// Valid channel prefix characters.
const CHANNEL_PREFIXES: &[char] = &['#', '&', '+', '!'];

/// Result of validation operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    /// The input was empty.
    Empty,
    /// The input was too long.
    TooLong {
        /// Maximum allowed length.
        max: usize,
        /// Actual length.
        actual: usize,
    },
    /// Invalid character found at position.
    InvalidChar {
        /// The invalid character.
        ch: char,
        /// Position in the string.
        position: usize,
    },
    /// Missing required prefix.
    MissingPrefix,
    /// Invalid first character.
    InvalidFirstChar {
        /// The invalid character.
        ch: char,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Empty => write!(f, "input is empty"),
            ValidationError::TooLong { max, actual } => {
                write!(f, "input too long: {} bytes (max {})", actual, max)
            }
            ValidationError::InvalidChar { ch, position } => {
                write!(f, "invalid character {:?} at position {}", ch, position)
            }
            ValidationError::MissingPrefix => write!(f, "missing required prefix"),
            ValidationError::InvalidFirstChar { ch } => {
                write!(f, "invalid first character: {:?}", ch)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Check if a character is illegal according to the transport layer rules.
///
/// This matches the pattern used in the transport layer:
/// `ch == '\0' || (ch.is_control() && ch != '\r' && ch != '\n')`
///
/// This rejects:
/// - NUL (0x00)
/// - All C0 control characters (0x01-0x1F) except CR and LF
///
/// This allows:
/// - CR (0x0D) and LF (0x0A) as line delimiters
/// - All printable characters
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_illegal_control_char;
///
/// assert!(is_illegal_control_char('\x00')); // NUL - always illegal
/// assert!(is_illegal_control_char('\x02')); // Bold - illegal at transport layer
/// assert!(is_illegal_control_char('\x03')); // Color - illegal at transport layer
/// assert!(is_illegal_control_char('\x01')); // SOH - illegal
/// assert!(!is_illegal_control_char('\r')); // CR - allowed as delimiter
/// assert!(!is_illegal_control_char('\n')); // LF - allowed as delimiter
/// assert!(!is_illegal_control_char('a')); // Normal chars allowed
/// ```
#[inline]
pub fn is_illegal_control_char(c: char) -> bool {
    c == '\0' || (c.is_control() && c != '\r' && c != '\n')
}

/// Check if a string contains any illegal control characters.
///
/// This uses the transport layer validation pattern. See [`is_illegal_control_char`].
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::contains_illegal_control_chars;
///
/// assert!(contains_illegal_control_chars("hello\x00world")); // NUL
/// assert!(contains_illegal_control_chars("\x02bold\x02")); // Bold codes
/// assert!(!contains_illegal_control_chars("hello world")); // Normal text
/// assert!(!contains_illegal_control_chars("line\r\n")); // CR/LF allowed
/// ```
pub fn contains_illegal_control_chars(s: &str) -> bool {
    s.chars().any(is_illegal_control_char)
}

/// Strip all illegal control characters from a string.
///
/// Removes all characters that would be rejected by the transport layer.
/// CR and LF are preserved.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::strip_illegal_control_chars;
///
/// assert_eq!(strip_illegal_control_chars("hello\x00world"), "helloworld");
/// assert_eq!(strip_illegal_control_chars("\x02bold\x02"), "bold");
/// assert_eq!(strip_illegal_control_chars("line\r\n"), "line\r\n"); // CR/LF kept
/// ```
pub fn strip_illegal_control_chars(s: &str) -> Cow<'_, str> {
    if !contains_illegal_control_chars(s) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(s.chars().filter(|c| !is_illegal_control_char(*c)).collect())
}

/// Check if a character is a protocol control character (NUL, CR, LF).
///
/// These characters are never valid in IRC messages as they are used
/// as protocol delimiters.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_protocol_control_char;
///
/// assert!(is_protocol_control_char('\x00')); // NUL
/// assert!(is_protocol_control_char('\x0D')); // CR
/// assert!(is_protocol_control_char('\x0A')); // LF
/// assert!(!is_protocol_control_char('a'));
/// assert!(!is_protocol_control_char('\x02')); // Bold - formatting, not protocol
/// ```
#[inline]
pub fn is_protocol_control_char(c: char) -> bool {
    PROTOCOL_CONTROL_CHARS.contains(&c)
}

/// Check if a character is an IRC formatting control character.
///
/// These characters are used for text formatting (bold, colors, etc.)
/// and are valid in message content.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_format_control_char;
///
/// assert!(is_format_control_char('\x02')); // Bold
/// assert!(is_format_control_char('\x03')); // Color
/// assert!(is_format_control_char('\x1F')); // Underline
/// assert!(!is_format_control_char('a'));
/// assert!(!is_format_control_char('\x00')); // NUL - protocol, not formatting
/// ```
#[inline]
pub fn is_format_control_char(c: char) -> bool {
    FORMAT_CONTROL_CHARS.contains(&c)
}

/// Check if a character is any IRC control character (protocol or formatting).
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_irc_control_char;
///
/// assert!(is_irc_control_char('\x00')); // NUL
/// assert!(is_irc_control_char('\x02')); // Bold
/// assert!(is_irc_control_char('\x03')); // Color
/// assert!(!is_irc_control_char('a'));
/// ```
#[inline]
pub fn is_irc_control_char(c: char) -> bool {
    is_protocol_control_char(c) || is_format_control_char(c)
}

/// Check if a string contains any protocol control characters.
///
/// Returns `true` if the string contains NUL, CR, or LF.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::contains_protocol_control_chars;
///
/// assert!(contains_protocol_control_chars("hello\x00world"));
/// assert!(contains_protocol_control_chars("line\r\n"));
/// assert!(!contains_protocol_control_chars("hello world"));
/// ```
pub fn contains_protocol_control_chars(s: &str) -> bool {
    s.chars().any(is_protocol_control_char)
}

/// Check if a string contains any IRC formatting control characters.
///
/// Returns `true` if the string contains bold, color, etc. codes.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::contains_format_control_chars;
///
/// assert!(contains_format_control_chars("\x02bold\x02"));
/// assert!(contains_format_control_chars("\x034red"));
/// assert!(!contains_format_control_chars("plain text"));
/// ```
pub fn contains_format_control_chars(s: &str) -> bool {
    s.chars().any(is_format_control_char)
}

/// Check if a string contains any C0 control characters (0x00-0x1F).
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::contains_c0_control_chars;
///
/// assert!(contains_c0_control_chars("hello\x00"));
/// assert!(contains_c0_control_chars("test\x1F"));
/// assert!(!contains_c0_control_chars("hello world"));
/// ```
pub fn contains_c0_control_chars(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_control())
}

/// Strip all protocol control characters from a string.
///
/// Removes NUL, CR, and LF characters.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::strip_protocol_control_chars;
///
/// assert_eq!(strip_protocol_control_chars("hello\x00world"), "helloworld");
/// assert_eq!(strip_protocol_control_chars("line\r\n"), "line");
/// ```
pub fn strip_protocol_control_chars(s: &str) -> Cow<'_, str> {
    if !contains_protocol_control_chars(s) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(s.chars().filter(|c| !is_protocol_control_char(*c)).collect())
}

/// Strip all C0 control characters (0x00-0x1F) from a string.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::strip_c0_control_chars;
///
/// assert_eq!(strip_c0_control_chars("hello\x00\x02world"), "helloworld");
/// ```
pub fn strip_c0_control_chars(s: &str) -> Cow<'_, str> {
    if !contains_c0_control_chars(s) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(s.chars().filter(|c| !c.is_ascii_control()).collect())
}

/// Validate an IRC nickname.
///
/// Per RFC 2812, nicknames must:
/// - Be 1-50 characters (configurable via ISUPPORT, default 9)
/// - Start with a letter or special char ([\]^_`{|})
/// - Contain only letters, digits, special chars, or hyphens
/// - Not contain spaces, NUL, CR, LF, or other invalid chars
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::validate_nickname;
///
/// assert!(validate_nickname("Nick").is_ok());
/// assert!(validate_nickname("Nick_123").is_ok());
/// assert!(validate_nickname("[test]").is_ok());
/// assert!(validate_nickname("").is_err()); // Empty
/// assert!(validate_nickname("123nick").is_err()); // Starts with digit
/// assert!(validate_nickname("nick name").is_err()); // Contains space
/// ```
pub fn validate_nickname(nick: &str) -> Result<(), ValidationError> {
    validate_nickname_with_max_len(nick, 50)
}

/// Validate an IRC nickname with a custom maximum length.
///
/// This is useful when you have ISUPPORT NICKLEN information.
pub fn validate_nickname_with_max_len(nick: &str, max_len: usize) -> Result<(), ValidationError> {
    if nick.is_empty() {
        return Err(ValidationError::Empty);
    }

    let len = nick.chars().count();
    if len > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: len,
        });
    }

    let mut chars = nick.chars();
    let first = chars.next().unwrap();

    // First character must be letter or special
    if !is_valid_nick_first_char(first) {
        return Err(ValidationError::InvalidFirstChar { ch: first });
    }

    // Rest must be letter, digit, special, or hyphen
    for (i, c) in chars.enumerate() {
        if !is_valid_nick_char(c) {
            return Err(ValidationError::InvalidChar {
                ch: c,
                position: i + 1,
            });
        }
    }

    Ok(())
}

/// Check if a character is valid as the first character of a nickname.
///
/// Per RFC 2812, first char must be a letter or special char ([\]^_`{|}).
#[inline]
pub fn is_valid_nick_first_char(c: char) -> bool {
    c.is_ascii_alphabetic() || is_nick_special_char(c)
}

/// Check if a character is valid in a nickname (after first char).
///
/// Per RFC 2812, subsequent chars can be letter, digit, special, or hyphen.
#[inline]
pub fn is_valid_nick_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || is_nick_special_char(c) || c == '-'
}

/// Check if a character is a special nickname character.
///
/// Special chars per RFC 2812: [ ] \ ` _ ^ { | }
/// These correspond to 0x5B-0x60 and 0x7B-0x7D in ASCII.
#[inline]
pub fn is_nick_special_char(c: char) -> bool {
    let code = c as u32;
    (0x5B..=0x60).contains(&code) || (0x7B..=0x7D).contains(&code)
}

/// Validate an IRC channel name.
///
/// Per RFC 2812, channel names must:
/// - Start with #, &, +, or !
/// - Be at most 50 characters (configurable via ISUPPORT)
/// - Not contain space, comma, BEL (0x07), or NUL
/// - Not contain control characters
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::validate_channel_name;
///
/// assert!(validate_channel_name("#channel").is_ok());
/// assert!(validate_channel_name("&local").is_ok());
/// assert!(validate_channel_name("channel").is_err()); // Missing prefix
/// assert!(validate_channel_name("#chan nel").is_err()); // Contains space
/// assert!(validate_channel_name("").is_err()); // Empty
/// ```
pub fn validate_channel_name(name: &str) -> Result<(), ValidationError> {
    validate_channel_name_with_max_len(name, 50)
}

/// Validate an IRC channel name with a custom maximum length.
///
/// This is useful when you have ISUPPORT CHANNELLEN information.
pub fn validate_channel_name_with_max_len(
    name: &str,
    max_len: usize,
) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::Empty);
    }

    let len = name.chars().count();
    if len > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: len,
        });
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    // Must have valid prefix
    if !CHANNEL_PREFIXES.contains(&first) {
        return Err(ValidationError::MissingPrefix);
    }

    // Check for invalid characters
    for (i, c) in chars.enumerate() {
        if INVALID_CHAN_CHARS.contains(&c) || c.is_control() {
            return Err(ValidationError::InvalidChar {
                ch: c,
                position: i + 1,
            });
        }
    }

    Ok(())
}

/// Check if a string is a valid channel name.
///
/// This is a convenience wrapper around `validate_channel_name`.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_valid_channel_name;
///
/// assert!(is_valid_channel_name("#rust"));
/// assert!(!is_valid_channel_name("rust"));
/// ```
#[inline]
pub fn is_valid_channel_name(name: &str) -> bool {
    validate_channel_name(name).is_ok()
}

/// Check if a string is a valid nickname.
///
/// This is a convenience wrapper around `validate_nickname`.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_valid_nickname;
///
/// assert!(is_valid_nickname("Nick"));
/// assert!(!is_valid_nickname(""));
/// ```
#[inline]
pub fn is_valid_nickname(nick: &str) -> bool {
    validate_nickname(nick).is_ok()
}

/// Validate an IRC username (ident).
///
/// Usernames must:
/// - Be 1-10 characters (default limit, configurable)
/// - Not contain spaces, NUL, CR, LF, or @
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::validate_username;
///
/// assert!(validate_username("user").is_ok());
/// assert!(validate_username("user123").is_ok());
/// assert!(validate_username("").is_err()); // Empty
/// assert!(validate_username("user name").is_err()); // Contains space
/// assert!(validate_username("user@host").is_err()); // Contains @
/// ```
pub fn validate_username(user: &str) -> Result<(), ValidationError> {
    validate_username_with_max_len(user, 10)
}

/// Validate an IRC username with a custom maximum length.
pub fn validate_username_with_max_len(user: &str, max_len: usize) -> Result<(), ValidationError> {
    if user.is_empty() {
        return Err(ValidationError::Empty);
    }

    let len = user.chars().count();
    if len > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: len,
        });
    }

    for (i, c) in user.chars().enumerate() {
        if c == ' ' || c == '@' || c.is_control() {
            return Err(ValidationError::InvalidChar { ch: c, position: i });
        }
    }

    Ok(())
}

/// Check if a string is a valid username.
///
/// This is a convenience wrapper around `validate_username`.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_valid_username;
///
/// assert!(is_valid_username("user"));
/// assert!(!is_valid_username(""));
/// ```
#[inline]
pub fn is_valid_username(user: &str) -> bool {
    validate_username(user).is_ok()
}

/// Validate an IRC hostname.
///
/// Hostnames must:
/// - Be non-empty
/// - Not contain spaces or control characters
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::validate_hostname;
///
/// assert!(validate_hostname("example.com").is_ok());
/// assert!(validate_hostname("192.168.1.1").is_ok());
/// assert!(validate_hostname("").is_err());
/// assert!(validate_hostname("host name").is_err());
/// ```
pub fn validate_hostname(host: &str) -> Result<(), ValidationError> {
    if host.is_empty() {
        return Err(ValidationError::Empty);
    }

    for (i, c) in host.chars().enumerate() {
        if c == ' ' || c.is_control() {
            return Err(ValidationError::InvalidChar { ch: c, position: i });
        }
    }

    Ok(())
}

/// Check if a string is a valid hostname.
///
/// This is a convenience wrapper around `validate_hostname`.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::is_valid_hostname;
///
/// assert!(is_valid_hostname("example.com"));
/// assert!(!is_valid_hostname(""));
/// ```
#[inline]
pub fn is_valid_hostname(host: &str) -> bool {
    validate_hostname(host).is_ok()
}

/// Validate that a message line doesn't contain protocol control characters.
///
/// IRC message lines cannot contain NUL, CR, or LF as these are protocol delimiters.
///
/// # Examples
///
/// ```
/// use slirc_proto::validation::validate_message_line;
///
/// assert!(validate_message_line("Hello, world!").is_ok());
/// assert!(validate_message_line("\x02bold text\x02").is_ok()); // Formatting OK
/// assert!(validate_message_line("line\r\n").is_err()); // CRLF not OK
/// assert!(validate_message_line("null\x00byte").is_err()); // NUL not OK
/// ```
pub fn validate_message_line(line: &str) -> Result<(), ValidationError> {
    for (i, c) in line.chars().enumerate() {
        if is_protocol_control_char(c) {
            return Err(ValidationError::InvalidChar { ch: c, position: i });
        }
    }
    Ok(())
}

/// Extension trait for validation on string types.
pub trait IrcValidationExt {
    /// Check if this string is a valid IRC nickname.
    fn is_valid_irc_nickname(&self) -> bool;

    /// Check if this string is a valid IRC channel name.
    fn is_valid_irc_channel(&self) -> bool;

    /// Check if this string is a valid IRC username.
    fn is_valid_irc_username(&self) -> bool;

    /// Check if this string is a valid IRC hostname.
    fn is_valid_irc_hostname(&self) -> bool;

    /// Check if this string contains protocol control characters.
    fn contains_irc_protocol_chars(&self) -> bool;

    /// Check if this string contains IRC formatting codes.
    fn contains_irc_format_codes(&self) -> bool;

    /// Check if this string contains illegal control characters.
    ///
    /// Uses the transport layer validation pattern.
    fn contains_illegal_control_chars(&self) -> bool;
}

impl IrcValidationExt for str {
    #[inline]
    fn is_valid_irc_nickname(&self) -> bool {
        is_valid_nickname(self)
    }

    #[inline]
    fn is_valid_irc_channel(&self) -> bool {
        is_valid_channel_name(self)
    }

    #[inline]
    fn is_valid_irc_username(&self) -> bool {
        is_valid_username(self)
    }

    #[inline]
    fn is_valid_irc_hostname(&self) -> bool {
        is_valid_hostname(self)
    }

    #[inline]
    fn contains_irc_protocol_chars(&self) -> bool {
        contains_protocol_control_chars(self)
    }

    #[inline]
    fn contains_irc_format_codes(&self) -> bool {
        contains_format_control_chars(self)
    }

    #[inline]
    fn contains_illegal_control_chars(&self) -> bool {
        contains_illegal_control_chars(self)
    }
}

impl IrcValidationExt for String {
    #[inline]
    fn is_valid_irc_nickname(&self) -> bool {
        is_valid_nickname(self)
    }

    #[inline]
    fn is_valid_irc_channel(&self) -> bool {
        is_valid_channel_name(self)
    }

    #[inline]
    fn is_valid_irc_username(&self) -> bool {
        is_valid_username(self)
    }

    #[inline]
    fn is_valid_irc_hostname(&self) -> bool {
        is_valid_hostname(self)
    }

    #[inline]
    fn contains_irc_protocol_chars(&self) -> bool {
        contains_protocol_control_chars(self)
    }

    #[inline]
    fn contains_irc_format_codes(&self) -> bool {
        contains_format_control_chars(self)
    }

    #[inline]
    fn contains_illegal_control_chars(&self) -> bool {
        contains_illegal_control_chars(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_control_chars() {
        assert!(is_protocol_control_char('\x00'));
        assert!(is_protocol_control_char('\x0D'));
        assert!(is_protocol_control_char('\x0A'));
        assert!(!is_protocol_control_char('a'));
        assert!(!is_protocol_control_char('\x02')); // Bold is formatting
    }

    #[test]
    fn test_format_control_chars() {
        assert!(is_format_control_char('\x02'));
        assert!(is_format_control_char('\x03'));
        assert!(is_format_control_char('\x0F'));
        assert!(is_format_control_char('\x16'));
        assert!(is_format_control_char('\x1F'));
        assert!(is_format_control_char('\x04')); // Hex color
        assert!(is_format_control_char('\x11')); // Monospace
        assert!(is_format_control_char('\x1D')); // Italic
        assert!(!is_format_control_char('a'));
        assert!(!is_format_control_char('\x00'));
    }

    #[test]
    fn test_contains_protocol_control() {
        assert!(contains_protocol_control_chars("hello\x00world"));
        assert!(contains_protocol_control_chars("line\r\n"));
        assert!(!contains_protocol_control_chars("hello world"));
        assert!(!contains_protocol_control_chars("\x02bold\x02"));
    }

    #[test]
    fn test_strip_protocol_control() {
        assert_eq!(strip_protocol_control_chars("hello"), "hello");
        assert_eq!(strip_protocol_control_chars("hello\x00"), "hello");
        assert_eq!(strip_protocol_control_chars("a\r\nb"), "ab");
    }

    #[test]
    fn test_nickname_validation() {
        // Valid nicknames
        assert!(validate_nickname("Nick").is_ok());
        assert!(validate_nickname("Nick123").is_ok());
        assert!(validate_nickname("Nick_").is_ok());
        assert!(validate_nickname("[test]").is_ok());
        assert!(validate_nickname("a").is_ok());

        // Invalid - empty
        assert!(matches!(validate_nickname(""), Err(ValidationError::Empty)));

        // Invalid - starts with digit
        assert!(matches!(
            validate_nickname("123"),
            Err(ValidationError::InvalidFirstChar { .. })
        ));

        // Invalid - contains space
        assert!(matches!(
            validate_nickname("nick name"),
            Err(ValidationError::InvalidChar { ch: ' ', .. })
        ));

        // Invalid - too long
        let long_nick = "a".repeat(51);
        assert!(matches!(
            validate_nickname(&long_nick),
            Err(ValidationError::TooLong { .. })
        ));
    }

    #[test]
    fn test_channel_validation() {
        // Valid channels
        assert!(validate_channel_name("#channel").is_ok());
        assert!(validate_channel_name("&local").is_ok());
        assert!(validate_channel_name("+modeless").is_ok());
        assert!(validate_channel_name("!safe").is_ok());

        // Invalid - no prefix
        assert!(matches!(
            validate_channel_name("channel"),
            Err(ValidationError::MissingPrefix)
        ));

        // Invalid - empty
        assert!(matches!(
            validate_channel_name(""),
            Err(ValidationError::Empty)
        ));

        // Invalid - contains space
        assert!(matches!(
            validate_channel_name("#chan nel"),
            Err(ValidationError::InvalidChar { ch: ' ', .. })
        ));
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("user").is_ok());
        assert!(validate_username("u").is_ok());

        // Invalid - empty
        assert!(matches!(validate_username(""), Err(ValidationError::Empty)));

        // Invalid - contains @
        assert!(matches!(
            validate_username("user@host"),
            Err(ValidationError::InvalidChar { ch: '@', .. })
        ));

        // Invalid - too long
        let long_user = "a".repeat(11);
        assert!(matches!(
            validate_username(&long_user),
            Err(ValidationError::TooLong { .. })
        ));
    }

    #[test]
    fn test_hostname_validation() {
        assert!(validate_hostname("example.com").is_ok());
        assert!(validate_hostname("192.168.1.1").is_ok());
        assert!(validate_hostname("host").is_ok());

        assert!(matches!(validate_hostname(""), Err(ValidationError::Empty)));
        assert!(matches!(
            validate_hostname("host name"),
            Err(ValidationError::InvalidChar { ch: ' ', .. })
        ));
    }

    #[test]
    fn test_message_line_validation() {
        assert!(validate_message_line("Hello, world!").is_ok());
        assert!(validate_message_line("\x02bold\x02").is_ok()); // Formatting OK
        assert!(validate_message_line("").is_ok()); // Empty OK

        // Protocol control chars not OK
        assert!(validate_message_line("line\r\n").is_err());
        assert!(validate_message_line("null\x00byte").is_err());
    }

    #[test]
    fn test_extension_trait() {
        assert!("Nick".is_valid_irc_nickname());
        assert!(!"".is_valid_irc_nickname());

        assert!("#channel".is_valid_irc_channel());
        assert!(!"channel".is_valid_irc_channel());

        assert!("user".is_valid_irc_username());
        assert!("example.com".is_valid_irc_hostname());

        assert!("hello\x00".contains_irc_protocol_chars());
        assert!(!("hello".contains_irc_protocol_chars()));

        assert!("\x02bold".contains_irc_format_codes());
        assert!(!"plain".contains_irc_format_codes());
    }

    #[test]
    fn test_validation_error_display() {
        assert_eq!(format!("{}", ValidationError::Empty), "input is empty");
        assert_eq!(
            format!(
                "{}",
                ValidationError::TooLong {
                    max: 10,
                    actual: 15
                }
            ),
            "input too long: 15 bytes (max 10)"
        );
        assert_eq!(
            format!(
                "{}",
                ValidationError::InvalidChar {
                    ch: '@',
                    position: 5
                }
            ),
            "invalid character '@' at position 5"
        );
    }

    #[test]
    fn test_nick_special_chars() {
        // RFC 2812 special chars: [ ] \ ` _ ^ { | }
        assert!(is_nick_special_char('['));
        assert!(is_nick_special_char(']'));
        assert!(is_nick_special_char('\\'));
        assert!(is_nick_special_char('`'));
        assert!(is_nick_special_char('_'));
        assert!(is_nick_special_char('^'));
        assert!(is_nick_special_char('{'));
        assert!(is_nick_special_char('|'));
        assert!(is_nick_special_char('}'));

        assert!(!is_nick_special_char('a'));
        assert!(!is_nick_special_char('1'));
        assert!(!is_nick_special_char('-'));
    }

    #[test]
    fn test_c0_control_chars() {
        assert!(contains_c0_control_chars("hello\x00"));
        assert!(contains_c0_control_chars("hello\x1F"));
        assert!(!contains_c0_control_chars("hello world"));

        assert_eq!(strip_c0_control_chars("hello"), "hello");
        assert_eq!(strip_c0_control_chars("a\x00\x02b"), "ab");
    }

    #[test]
    fn test_illegal_control_char() {
        // NUL is always illegal
        assert!(is_illegal_control_char('\x00'));

        // All C0 control chars except CR/LF are illegal
        assert!(is_illegal_control_char('\x01')); // SOH
        assert!(is_illegal_control_char('\x02')); // Bold
        assert!(is_illegal_control_char('\x03')); // Color
        assert!(is_illegal_control_char('\x07')); // BEL
        assert!(is_illegal_control_char('\x1F')); // Unit separator

        // CR and LF are allowed (line delimiters)
        assert!(!is_illegal_control_char('\r'));
        assert!(!is_illegal_control_char('\n'));

        // Normal printable chars are allowed
        assert!(!is_illegal_control_char('a'));
        assert!(!is_illegal_control_char(' '));
        assert!(!is_illegal_control_char('!'));
    }

    #[test]
    fn test_contains_illegal_control_chars() {
        // NUL is illegal
        assert!(contains_illegal_control_chars("hello\x00world"));

        // Formatting codes are illegal at transport layer
        assert!(contains_illegal_control_chars("\x02bold\x02"));
        assert!(contains_illegal_control_chars("\x03color"));

        // Normal text is fine
        assert!(!contains_illegal_control_chars("hello world"));

        // CR/LF are allowed
        assert!(!contains_illegal_control_chars("line\r\n"));
    }

    #[test]
    fn test_strip_illegal_control_chars() {
        // No changes needed
        assert_eq!(strip_illegal_control_chars("hello"), "hello");

        // Strip NUL
        assert_eq!(strip_illegal_control_chars("hello\x00world"), "helloworld");

        // Strip formatting codes
        assert_eq!(strip_illegal_control_chars("\x02bold\x02"), "bold");

        // CR/LF are preserved
        assert_eq!(strip_illegal_control_chars("line\r\n"), "line\r\n");

        // Mixed
        assert_eq!(
            strip_illegal_control_chars("a\x00\x02b\r\nc"),
            "ab\r\nc"
        );
    }
}

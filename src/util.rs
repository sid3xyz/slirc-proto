//! Utility functions for IRC protocol handling.
//!
//! This module provides helper functions that are commonly needed when
//! working with IRC messages, including safe string truncation and
//! length validation.

/// Maximum length for IRC tags section (per IRCv3 spec).
pub const MAX_TAGS_LENGTH: usize = 8191;

/// Maximum length for client-originated tag data.
pub const MAX_CLIENT_TAG_DATA: usize = 4094;

/// Maximum length for server-originated tag data.
pub const MAX_SERVER_TAG_DATA: usize = 4094;

/// Maximum length for IRC message body (excluding tags).
pub const MAX_MESSAGE_BODY: usize = 512;

/// Truncates a string to at most `max_bytes` bytes without breaking
/// a multi-byte UTF-8 codepoint at the end.
///
/// This is essential when working with IRC message limits, as naively
/// truncating at a byte boundary could produce invalid UTF-8.
///
/// # Examples
///
/// ```
/// use slirc_proto::util::truncate_utf8_safe;
///
/// // ASCII string truncates normally
/// assert_eq!(truncate_utf8_safe("hello world", 5), "hello");
///
/// // Multi-byte chars are not split
/// let emoji = "Hello 👋 World";
/// let truncated = truncate_utf8_safe(emoji, 8);
/// assert_eq!(truncated, "Hello "); // Stops before the 4-byte emoji
///
/// // String shorter than limit is unchanged
/// assert_eq!(truncate_utf8_safe("hi", 10), "hi");
/// ```
#[inline]
pub fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Find the last valid UTF-8 boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    &s[..end]
}

/// Truncates a string to at most `max_chars` characters.
///
/// Unlike [`truncate_utf8_safe`], this counts Unicode codepoints rather than bytes.
///
/// # Examples
///
/// ```
/// use slirc_proto::util::truncate_chars;
///
/// assert_eq!(truncate_chars("hello", 3), "hel");
/// assert_eq!(truncate_chars("héllo", 3), "hél");
/// assert_eq!(truncate_chars("👋🌍🚀", 2), "👋🌍");
/// ```
#[inline]
pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Checks if a string would exceed the IRC message body limit when serialized.
///
/// Returns `Some(len)` if the string exceeds 510 bytes (512 - CRLF),
/// or `None` if it's within limits.
#[inline]
pub fn check_body_length(s: &str) -> Option<usize> {
    // 512 bytes total, minus 2 for CRLF
    const MAX_BODY_CONTENT: usize = 510;
    if s.len() > MAX_BODY_CONTENT {
        Some(s.len())
    } else {
        None
    }
}

/// Checks if a tags section would exceed the IRC tags limit.
///
/// Returns `Some(len)` if the tags exceed 8191 bytes, or `None` if within limits.
#[inline]
pub fn check_tags_length(tags: &str) -> Option<usize> {
    if tags.len() > MAX_TAGS_LENGTH {
        Some(tags.len())
    } else {
        None
    }
}

/// Splits a long message into chunks that fit within IRC limits.
///
/// Each chunk will be at most `max_bytes` long, and will not break
/// multi-byte UTF-8 characters.
///
/// # Examples
///
/// ```
/// use slirc_proto::util::split_message;
///
/// let long_msg = "Hello World! This is a test.";
/// let chunks: Vec<_> = split_message(long_msg, 10).collect();
/// assert_eq!(chunks, vec!["Hello Worl", "d! This is", " a test."]);
/// ```
pub fn split_message(s: &str, max_bytes: usize) -> impl Iterator<Item = &str> {
    SplitMessage {
        remaining: s,
        max_bytes,
    }
}

struct SplitMessage<'a> {
    remaining: &'a str,
    max_bytes: usize,
}

impl<'a> Iterator for SplitMessage<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let chunk = truncate_utf8_safe(self.remaining, self.max_bytes);
        self.remaining = &self.remaining[chunk.len()..];
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_utf8_safe_ascii() {
        assert_eq!(truncate_utf8_safe("hello world", 5), "hello");
        assert_eq!(truncate_utf8_safe("hello", 10), "hello");
        assert_eq!(truncate_utf8_safe("", 5), "");
    }

    #[test]
    fn test_truncate_utf8_safe_multibyte() {
        // 2-byte UTF-8: é is 2 bytes (0xC3 0xA9)
        let s = "café";
        assert_eq!(truncate_utf8_safe(s, 4), "caf"); // Can't fit é
        assert_eq!(truncate_utf8_safe(s, 5), "café"); // Fits perfectly

        // 3-byte UTF-8: € is 3 bytes
        let s = "100€";
        assert_eq!(truncate_utf8_safe(s, 4), "100"); // Can't fit €
        assert_eq!(truncate_utf8_safe(s, 6), "100€"); // Fits

        // 4-byte UTF-8: 👋 is 4 bytes
        let s = "Hi👋";
        assert_eq!(truncate_utf8_safe(s, 3), "Hi"); // Can't fit emoji
        assert_eq!(truncate_utf8_safe(s, 6), "Hi👋"); // Fits
    }

    #[test]
    fn test_truncate_utf8_safe_edge_cases() {
        // All multibyte
        let s = "日本語";
        assert_eq!(truncate_utf8_safe(s, 3), "日");
        assert_eq!(truncate_utf8_safe(s, 6), "日本");
        assert_eq!(truncate_utf8_safe(s, 9), "日本語");

        // Max bytes = 0
        assert_eq!(truncate_utf8_safe("hello", 0), "");
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("hello", 3), "hel");
        assert_eq!(truncate_chars("日本語", 2), "日本");
        assert_eq!(truncate_chars("👋🌍🚀", 2), "👋🌍");
        assert_eq!(truncate_chars("short", 100), "short");
    }

    #[test]
    fn test_split_message() {
        let chunks: Vec<_> = split_message("hello world", 5).collect();
        assert_eq!(chunks, vec!["hello", " worl", "d"]);

        // With UTF-8
        let chunks: Vec<_> = split_message("日本語テスト", 6).collect();
        assert_eq!(chunks, vec!["日本", "語テ", "スト"]);

        // Empty string
        let chunks: Vec<_> = split_message("", 5).collect();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_check_body_length() {
        assert!(check_body_length("short").is_none());
        
        let long = "x".repeat(600);
        assert_eq!(check_body_length(&long), Some(600));
    }

    #[test]
    fn test_check_tags_length() {
        assert!(check_tags_length("short=tag").is_none());
        
        let long = "x".repeat(9000);
        assert_eq!(check_tags_length(&long), Some(9000));
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_TAGS_LENGTH, 8191);
        assert_eq!(MAX_CLIENT_TAG_DATA, 4094);
        assert_eq!(MAX_SERVER_TAG_DATA, 4094);
        assert_eq!(MAX_MESSAGE_BODY, 512);
    }
}

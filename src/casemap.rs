//! IRC case-mapping functions.
//!
//! IRC uses a special case-insensitive comparison where some characters
//! are considered equivalent (e.g., `[` and `{`). This implements the
//! `rfc1459` case mapping which is the most common.

/// Convert a string to IRC lowercase using RFC 1459 case mapping.
///
/// In addition to ASCII lowercase conversion, this maps:
/// - `[` → `{`
/// - `]` → `}`
/// - `\` → `|`
/// - `~` → `^`
pub fn irc_to_lower(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',

            'A'..='Z' => c.to_ascii_lowercase(),

            _ => c,
        })
        .collect()
}

/// Compare two strings using IRC case-insensitive comparison.
///
/// Uses the RFC 1459 case mapping where certain characters are equivalent.
pub fn irc_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.chars().zip(b.chars()).all(|(ca, cb)| {
        let ca_lower = match ca {
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',
            'A'..='Z' => ca.to_ascii_lowercase(),
            _ => ca,
        };
        let cb_lower = match cb {
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            '~' => '^',
            'A'..='Z' => cb.to_ascii_lowercase(),
            _ => cb,
        };
        ca_lower == cb_lower
    })
}

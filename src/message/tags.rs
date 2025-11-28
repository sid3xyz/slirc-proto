//! IRCv3 message tag escaping utilities.

use std::fmt::{Result as FmtResult, Write};

/// Escape a tag value for serialization.
///
/// Escapes special characters according to the IRCv3 message-tags spec.
pub fn escape_tag_value(f: &mut dyn Write, value: &str) -> FmtResult {
    for c in value.chars() {
        match c {
            ';' => f.write_str("\\:")?,
            ' ' => f.write_str("\\s")?,
            '\\' => f.write_str("\\\\")?,
            '\r' => f.write_str("\\r")?,
            '\n' => f.write_str("\\n")?,
            c => f.write_char(c)?,
        }
    }
    Ok(())
}

/// Unescape a tag value from wire format.
///
/// Reverses the escaping applied by [`escape_tag_value`].
pub fn unescape_tag_value(value: &str) -> String {
    let mut unescaped = String::with_capacity(value.len());
    let mut iter = value.chars();
    while let Some(c) = iter.next() {
        let r = if c == '\\' {
            match iter.next() {
                Some(':') => ';',
                Some('s') => ' ',
                Some('\\') => '\\',
                Some('r') => '\r',
                Some('n') => '\n',
                Some(c) => c,
                None => break,
            }
        } else {
            c
        };
        unescaped.push(r);
    }
    unescaped
}

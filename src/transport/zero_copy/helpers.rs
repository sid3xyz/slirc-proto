//! Shared helper functions for zero-copy transports.

use bytes::BytesMut;

use crate::error::ProtocolError;

use super::super::error::TransportReadError;

/// Maximum length for client-sent tag data (4094 bytes per IRCv3).
/// "Clients MUST NOT send messages with tag data exceeding 4094 bytes"
pub const MAX_CLIENT_TAG_DATA: usize = 4094;

/// Find the position of the next CRLF or LF line ending in the buffer.
///
/// Returns the position of the LF byte (newline character).
pub fn find_crlf(buffer: &BytesMut) -> Option<usize> {
    buffer.iter().position(|&b| b == b'\n')
}

/// Validate IRC line lengths according to IRCv3 message tags spec.
///
/// For client messages:
/// - Tag data (excluding the leading `@` and trailing space) must be ≤ 4094 bytes
/// - Message body (everything after tags) must be ≤ `max_body_len` bytes (including CRLF)
///
/// Returns Ok(()) if lengths are valid, or an appropriate error.
pub fn validate_irc_line_length(line: &[u8], max_body_len: usize) -> Result<(), TransportReadError> {
    // Find where tags end and body begins
    if line.first() == Some(&b'@') {
        // Message has tags - find the first space after the tag section
        if let Some(space_pos) = line.iter().position(|&b| b == b' ') {
            // Tags section is from byte 1 to space_pos (excluding @ and space)
            let tag_data_len = space_pos - 1;
            if tag_data_len > MAX_CLIENT_TAG_DATA {
                return Err(TransportReadError::Protocol(ProtocolError::TagsTooLong {
                    actual: tag_data_len,
                    limit: MAX_CLIENT_TAG_DATA,
                }));
            }

            // Body is everything after the space
            let body_len = line.len() - space_pos - 1;
            if body_len > max_body_len {
                return Err(TransportReadError::Protocol(
                    ProtocolError::MessageTooLong {
                        actual: body_len,
                        limit: max_body_len,
                    },
                ));
            }
        } else {
            // Tags only, no body - just check tag length
            let tag_data_len = line.len() - 1; // Exclude the @
            if tag_data_len > MAX_CLIENT_TAG_DATA {
                return Err(TransportReadError::Protocol(ProtocolError::TagsTooLong {
                    actual: tag_data_len,
                    limit: MAX_CLIENT_TAG_DATA,
                }));
            }
        }
    } else {
        // No tags - entire line is the body
        if line.len() > max_body_len {
            return Err(TransportReadError::Protocol(
                ProtocolError::MessageTooLong {
                    actual: line.len(),
                    limit: max_body_len,
                },
            ));
        }
    }

    Ok(())
}

/// Validate a line slice as valid UTF-8 and check for control characters.
///
/// Returns the validated string slice if valid, or an error if:
/// - The slice is not valid UTF-8
/// - The line contains illegal control characters (NUL, etc.)
pub fn validate_line(slice: &[u8]) -> Result<&str, TransportReadError> {
    let s = std::str::from_utf8(slice).map_err(|e| {
        TransportReadError::Protocol(ProtocolError::InvalidUtf8(format!(
            "byte position {}: {}",
            e.valid_up_to(),
            e
        )))
    })?;

    // Trim CRLF for validation
    let trimmed = s.trim_end_matches(['\r', '\n']);

    // Check for NUL and other illegal control characters
    for ch in trimmed.chars() {
        if crate::format::is_illegal_control_char(ch) {
            return Err(TransportReadError::Protocol(
                ProtocolError::IllegalControlChar(ch),
            ));
        }
    }

    Ok(s)
}

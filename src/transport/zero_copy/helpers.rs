//! Shared helper functions for zero-copy transports.

use bytes::BytesMut;

use crate::error::ProtocolError;

use super::super::error::TransportReadError;

/// Find the position of the next CRLF or LF line ending in the buffer.
///
/// Returns the position of the LF byte (newline character).
pub fn find_crlf(buffer: &BytesMut) -> Option<usize> {
    buffer.iter().position(|&b| b == b'\n')
}

/// Validate a line slice as valid UTF-8 and check for control characters.
///
/// Returns the validated string slice if valid, or an error if:
/// - The slice is not valid UTF-8
/// - The line contains illegal control characters (NUL, etc.)
pub fn validate_line(slice: &[u8]) -> Result<&str, TransportReadError> {
    let s = std::str::from_utf8(slice).map_err(|e| {
        TransportReadError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid UTF-8: {}", e),
        ))
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

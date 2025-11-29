//! Transport error types.

use crate::error::ProtocolError;

/// Errors that can occur when reading from a transport.
#[derive(Debug)]
#[non_exhaustive]
pub enum TransportReadError {
    /// An I/O error occurred.
    Io(std::io::Error),
    /// A protocol error occurred.
    Protocol(ProtocolError),
}

impl From<std::io::Error> for TransportReadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ProtocolError> for TransportReadError {
    fn from(err: ProtocolError) -> Self {
        Self::Protocol(err)
    }
}

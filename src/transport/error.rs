//! Transport error types.

use std::fmt;

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

impl fmt::Display for TransportReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "transport I/O error: {}", err),
            Self::Protocol(err) => write!(f, "transport protocol error: {}", err),
        }
    }
}

impl std::error::Error for TransportReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Protocol(err) => Some(err),
        }
    }
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

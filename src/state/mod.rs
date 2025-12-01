//! Sans-IO connection state machine for IRC protocol handling.
//!
//! This module provides a "sans-IO" state machine for managing IRC connection
//! lifecycle. It does not perform actual I/O—instead, it consumes events
//! (parsed messages) and produces actions (messages to send).
//!
//! # Design Philosophy
//!
//! The state machine is designed to be:
//! - **Sans-IO**: No network calls, timers, or blocking. Pure state transitions.
//! - **Runtime-agnostic**: Works with tokio, async-std, or blocking code.
//! - **Testable**: Easy to unit test without mocking network.
//!
//! # Example
//!
//! ```
//! use slirc_proto::state::{HandshakeMachine, HandshakeConfig, HandshakeAction};
//! use slirc_proto::MessageRef;
//!
//! let config = HandshakeConfig {
//!     nickname: "testbot".to_string(),
//!     username: "bot".to_string(),
//!     realname: "Test Bot".to_string(),
//!     password: None,
//!     request_caps: vec!["multi-prefix".to_string(), "sasl".to_string()],
//!     sasl_credentials: None,
//! };
//!
//! let mut machine = HandshakeMachine::new(config);
//!
//! // Get initial actions (CAP LS, NICK, USER)
//! let actions = machine.start();
//! for action in actions {
//!     // Send action.message() to server
//! }
//!
//! // Feed server responses
//! let cap_ack = MessageRef::parse(":server CAP * ACK :multi-prefix sasl").unwrap();
//! let actions = machine.feed(&cap_ack);
//! // Process actions...
//! ```

mod sync;
mod tracker;

pub use tracker::HandshakeMachine;

use crate::Message;

/// Current state of the IRC connection handshake.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConnectionState {
    /// Initial state, not yet connected.
    Disconnected,
    /// Sent CAP LS, awaiting capability list.
    CapabilityNegotiation,
    /// Performing SASL authentication.
    Authenticating,
    /// Sent CAP END, awaiting welcome (001).
    Registering,
    /// Received 001, fully connected.
    Connected,
    /// Connection terminated (QUIT sent or ERROR received).
    Terminated,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Configuration for the handshake state machine.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandshakeConfig {
    /// Desired nickname.
    pub nickname: String,
    /// Username (ident).
    pub username: String,
    /// Real name / GECOS.
    pub realname: String,
    /// Server password, if required.
    pub password: Option<String>,
    /// Capabilities to request (e.g., "multi-prefix", "sasl").
    pub request_caps: Vec<String>,
    /// SASL credentials, if SASL authentication is desired.
    pub sasl_credentials: Option<SaslCredentials>,
}

/// SASL authentication credentials.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaslCredentials {
    /// Account name (often same as nickname).
    pub account: String,
    /// Password.
    pub password: String,
}

/// Actions produced by the handshake state machine.
///
/// The caller is responsible for sending these messages to the server.
#[derive(Clone, Debug)]
pub enum HandshakeAction {
    /// Send this message to the server.
    ///
    /// Boxed to reduce enum size variance (Message is large).
    Send(Box<Message>),
    /// Connection is complete, proceed to normal operation.
    Complete,
    /// An error occurred during handshake.
    Error(HandshakeError),
}

/// Errors that can occur during handshake.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandshakeError {
    /// Server rejected capability request.
    CapabilityRejected(Vec<String>),
    /// SASL authentication failed.
    SaslFailed(String),
    /// Nickname collision.
    NicknameInUse(String),
    /// Server sent ERROR.
    ServerError(String),
    /// Unexpected message during handshake.
    ProtocolError(String),
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CapabilityRejected(caps) => {
                write!(f, "capability rejected: {}", caps.join(", "))
            }
            Self::SaslFailed(reason) => write!(f, "SASL authentication failed: {}", reason),
            Self::NicknameInUse(nick) => write!(f, "nickname in use: {}", nick),
            Self::ServerError(msg) => write!(f, "server error: {}", msg),
            Self::ProtocolError(msg) => write!(f, "protocol error: {}", msg),
        }
    }
}

impl std::error::Error for HandshakeError {}

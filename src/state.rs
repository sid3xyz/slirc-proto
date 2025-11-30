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

use std::collections::HashSet;

use crate::command::Command;
use crate::message::MessageRef;
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

/// Sans-IO state machine for IRC connection handshake.
///
/// This handles the CAP -> AUTHENTICATE -> NICK/USER -> 001 flow.
#[derive(Clone, Debug)]
pub struct HandshakeMachine {
    config: HandshakeConfig,
    state: ConnectionState,
    /// Capabilities acknowledged by server.
    enabled_caps: HashSet<String>,
    /// Capabilities available on server.
    available_caps: HashSet<String>,
    /// Whether we've sent NICK/USER.
    registration_sent: bool,
    /// Whether we're waiting for more CAP LS (multiline).
    waiting_for_more_caps: bool,
}

impl HandshakeMachine {
    /// Create a new handshake state machine with the given configuration.
    #[must_use]
    pub fn new(config: HandshakeConfig) -> Self {
        Self {
            config,
            state: ConnectionState::Disconnected,
            enabled_caps: HashSet::new(),
            available_caps: HashSet::new(),
            registration_sent: false,
            waiting_for_more_caps: false,
        }
    }

    /// Get the current connection state.
    #[must_use]
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Get the set of enabled capabilities.
    #[must_use]
    pub fn enabled_caps(&self) -> &HashSet<String> {
        &self.enabled_caps
    }

    /// Get the set of available capabilities.
    #[must_use]
    pub fn available_caps(&self) -> &HashSet<String> {
        &self.available_caps
    }

    /// Start the handshake. Returns initial messages to send.
    #[must_use]
    pub fn start(&mut self) -> Vec<HandshakeAction> {
        self.state = ConnectionState::CapabilityNegotiation;
        let mut actions = Vec::new();

        // Send PASS if configured
        if let Some(ref pass) = self.config.password {
            actions.push(HandshakeAction::Send(Box::new(
                Command::PASS(pass.clone()).into(),
            )));
        }

        // Request capability list (302 = IRCv3.2)
        actions.push(HandshakeAction::Send(Box::new(
            Command::CAP(None, crate::command::CapSubCommand::LS, Some("302".to_string()), None)
                .into(),
        )));

        actions
    }

    /// Feed a parsed message to the state machine.
    ///
    /// Returns actions to perform (messages to send, completion, or errors).
    #[must_use]
    pub fn feed(&mut self, msg: &MessageRef<'_>) -> Vec<HandshakeAction> {
        match self.state {
            ConnectionState::Disconnected => vec![],
            ConnectionState::CapabilityNegotiation => self.handle_cap_negotiation(msg),
            ConnectionState::Authenticating => self.handle_authentication(msg),
            ConnectionState::Registering => self.handle_registration(msg),
            ConnectionState::Connected | ConnectionState::Terminated => vec![],
        }
    }

    fn handle_cap_negotiation(&mut self, msg: &MessageRef<'_>) -> Vec<HandshakeAction> {
        let mut actions = Vec::new();

        if msg.command.name.eq_ignore_ascii_case("CAP") {
            let subcmd = msg.arg(1).unwrap_or("");
            match subcmd.to_ascii_uppercase().as_str() {
                "LS" => {
                    // Check for multiline (* prefix)
                    let (is_multiline, caps_str) = if msg.arg(2) == Some("*") {
                        (true, msg.arg(3).unwrap_or(""))
                    } else {
                        (false, msg.arg(2).unwrap_or(""))
                    };

                    // Parse available capabilities
                    for cap in caps_str.split_whitespace() {
                        // Handle capability values (cap=value)
                        let cap_name = cap.split('=').next().unwrap_or(cap);
                        self.available_caps.insert(cap_name.to_string());
                    }

                    if is_multiline {
                        self.waiting_for_more_caps = true;
                        return actions;
                    }

                    self.waiting_for_more_caps = false;

                    // Request capabilities we want that are available
                    let to_request: Vec<_> = self
                        .config
                        .request_caps
                        .iter()
                        .filter(|c| self.available_caps.contains(*c))
                        .cloned()
                        .collect();

                    if !to_request.is_empty() {
                        let caps_str = to_request.join(" ");
                        actions.push(HandshakeAction::Send(Box::new(
                            Command::CAP(
                                None,
                                crate::command::CapSubCommand::REQ,
                                None,
                                Some(caps_str),
                            )
                            .into(),
                        )));
                    } else {
                        // No caps to request, proceed to registration
                        actions.extend(self.finish_cap_negotiation());
                    }
                }
                "ACK" => {
                    let caps_str = msg.arg(2).unwrap_or("");
                    for cap in caps_str.split_whitespace() {
                        // Handle capability modifiers (-, ~, =)
                        let cap_name = cap.trim_start_matches(['-', '~', '=']);
                        if !cap.starts_with('-') {
                            self.enabled_caps.insert(cap_name.to_string());
                        }
                    }

                    // Check if SASL is enabled and we have credentials
                    if self.enabled_caps.contains("sasl")
                        && self.config.sasl_credentials.is_some()
                    {
                        self.state = ConnectionState::Authenticating;
                        actions.push(HandshakeAction::Send(Box::new(
                            Command::AUTHENTICATE("PLAIN".to_string()).into(),
                        )));
                    } else {
                        actions.extend(self.finish_cap_negotiation());
                    }
                }
                "NAK" => {
                    let caps_str = msg.arg(2).unwrap_or("");
                    let rejected: Vec<_> =
                        caps_str.split_whitespace().map(String::from).collect();
                    // NAK is not fatal, proceed with registration
                    actions.extend(self.finish_cap_negotiation());
                    if !rejected.is_empty() {
                        // Log but don't fail
                    }
                }
                _ => {}
            }
        }

        actions
    }

    fn handle_authentication(&mut self, msg: &MessageRef<'_>) -> Vec<HandshakeAction> {
        let mut actions = Vec::new();

        match msg.command.name.to_ascii_uppercase().as_str() {
            "AUTHENTICATE" => {
                let param = msg.arg(0).unwrap_or("");
                if param == "+" {
                    // Server ready for SASL payload
                    if let Some(ref creds) = self.config.sasl_credentials {
                        let payload = crate::sasl::encode_plain(&creds.account, &creds.password);
                        actions.push(HandshakeAction::Send(Box::new(
                            Command::AUTHENTICATE(payload).into(),
                        )));
                    }
                }
            }
            _ => {
                let cmd = msg.command.name;
                // Numeric responses
                if let Ok(numeric) = cmd.parse::<u16>() {
                    match numeric {
                        900 => {
                            // RPL_LOGGEDIN - SASL successful
                        }
                        903 => {
                            // RPL_SASLSUCCESS
                            actions.extend(self.finish_cap_negotiation());
                        }
                        902 | 904 | 905 | 906 | 907 => {
                            // SASL failures
                            let reason = msg.arg(1).unwrap_or("unknown error").to_string();
                            actions.push(HandshakeAction::Error(HandshakeError::SaslFailed(
                                reason,
                            )));
                            // Still try to continue without SASL
                            actions.extend(self.finish_cap_negotiation());
                        }
                        _ => {}
                    }
                }
            }
        }

        actions
    }

    fn handle_registration(&mut self, msg: &MessageRef<'_>) -> Vec<HandshakeAction> {
        let mut actions = Vec::new();

        match msg.command.name.to_ascii_uppercase().as_str() {
            "001" => {
                // RPL_WELCOME - fully connected
                self.state = ConnectionState::Connected;
                actions.push(HandshakeAction::Complete);
            }
            "433" | "432" => {
                // ERR_NICKNAMEINUSE or ERR_ERRONEUSNICKNAME
                let nick = msg.arg(1).unwrap_or(&self.config.nickname).to_string();
                actions.push(HandshakeAction::Error(HandshakeError::NicknameInUse(nick)));
            }
            "ERROR" => {
                let reason = msg.arg(0).unwrap_or("connection closed").to_string();
                self.state = ConnectionState::Terminated;
                actions.push(HandshakeAction::Error(HandshakeError::ServerError(reason)));
            }
            _ => {}
        }

        actions
    }

    fn finish_cap_negotiation(&mut self) -> Vec<HandshakeAction> {
        self.state = ConnectionState::Registering;
        let mut actions = Vec::new();

        // Send CAP END
        actions.push(HandshakeAction::Send(Box::new(
            Command::CAP(None, crate::command::CapSubCommand::END, None, None).into(),
        )));

        // Send NICK and USER if not already sent
        if !self.registration_sent {
            self.registration_sent = true;
            actions.push(HandshakeAction::Send(Box::new(
                Command::NICK(self.config.nickname.clone()).into(),
            )));
            actions.push(HandshakeAction::Send(Box::new(
                Command::USER(
                    self.config.username.clone(),
                    "0".to_string(),
                    self.config.realname.clone(),
                )
                .into(),
            )));
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> HandshakeConfig {
        HandshakeConfig {
            nickname: "testbot".to_string(),
            username: "bot".to_string(),
            realname: "Test Bot".to_string(),
            password: None,
            request_caps: vec!["multi-prefix".to_string()],
            sasl_credentials: None,
        }
    }

    #[test]
    fn test_start_sends_cap_ls() {
        let mut machine = HandshakeMachine::new(make_config());
        let actions = machine.start();

        assert_eq!(machine.state(), &ConnectionState::CapabilityNegotiation);
        assert_eq!(actions.len(), 1);

        if let HandshakeAction::Send(msg) = &actions[0] {
            assert!(matches!(msg.command, Command::CAP(_, _, _, _)));
        } else {
            panic!("Expected Send action");
        }
    }

    #[test]
    fn test_cap_ls_then_req() {
        let mut machine = HandshakeMachine::new(make_config());
        let _ = machine.start();

        let cap_ls = MessageRef::parse(":server CAP * LS :multi-prefix sasl").unwrap();
        let actions = machine.feed(&cap_ls);

        assert!(machine.available_caps().contains("multi-prefix"));
        assert!(machine.available_caps().contains("sasl"));

        // Should request multi-prefix (since it's in request_caps)
        assert!(!actions.is_empty());
        if let HandshakeAction::Send(msg) = &actions[0] {
            assert!(matches!(msg.command, Command::CAP(_, crate::command::CapSubCommand::REQ, _, _)));
        }
    }

    #[test]
    fn test_cap_ack_then_end() {
        let mut machine = HandshakeMachine::new(make_config());
        let _ = machine.start();

        let cap_ls = MessageRef::parse(":server CAP * LS :multi-prefix").unwrap();
        let _ = machine.feed(&cap_ls);

        let cap_ack = MessageRef::parse(":server CAP * ACK :multi-prefix").unwrap();
        let actions = machine.feed(&cap_ack);

        assert!(machine.enabled_caps().contains("multi-prefix"));
        assert_eq!(machine.state(), &ConnectionState::Registering);

        // Should have CAP END, NICK, USER
        assert!(actions.len() >= 3);
    }

    #[test]
    fn test_welcome_completes() {
        let mut machine = HandshakeMachine::new(make_config());
        let _ = machine.start();

        // Simulate full handshake
        let cap_ls = MessageRef::parse(":server CAP * LS :").unwrap();
        let _ = machine.feed(&cap_ls);

        let welcome = MessageRef::parse(":server 001 testbot :Welcome").unwrap();
        let actions = machine.feed(&welcome);

        assert_eq!(machine.state(), &ConnectionState::Connected);
        assert!(actions
            .iter()
            .any(|a| matches!(a, HandshakeAction::Complete)));
    }
}

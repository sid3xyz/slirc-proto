//! # slirc-proto
//!
//! A Rust library for parsing and serializing IRC protocol messages,
//! with full support for IRCv3 extensions.
//!
//! ## Features
//!
//! - IRC message parsing with tags, prefixes, commands, and parameters
//! - IRCv3 capability negotiation and message tags
//! - Zero-copy parsing with borrowed message types
//! - Optional Tokio integration for async networking
//! - User and channel mode parsing
//! - ISUPPORT (RPL_ISUPPORT) parsing
//! - Convenient message construction with builder pattern

#![deny(clippy::all)]
// TODO: Enable once documentation coverage is complete
// #![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! ## Quick Start
//!
//! ### Creating IRC Messages
//!
//! ```rust
//! use slirc_proto::{Message, prefix::Prefix};
//!
//! // Basic message construction
//! let privmsg = Message::privmsg("#rust", "Hello, world!");
//! let notice = Message::notice("nick", "Server notice");
//! let join = Message::join("#channel");
//!
//! // Messages with IRCv3 tags and prefixes
//! let tagged_msg = Message::privmsg("#dev", "Tagged message")
//!     .with_tag("time", Some("2023-01-01T12:00:00Z"))
//!     .with_tag("msgid", Some("abc123"))
//!     .with_prefix(Prefix::new_from_str("bot!bot@example.com"));
//!
//! println!("{}", tagged_msg); // Serializes to IRC protocol format
//! ```
//!
//! ### Parsing IRC Messages
//!
//! ```rust
//! use slirc_proto::Message;
//!
//! let raw = "@time=2023-01-01T12:00:00Z :nick!user@host PRIVMSG #channel :Hello!";
//! let message: Message = raw.parse().expect("Valid IRC message");
//!
//! if let Some(tags) = &message.tags {
//!     println!("Message has {} tags", tags.len());
//! }
//! ```
//!
//! ## Acknowledgments
//!
//! This project was inspired by the architectural patterns established by
//! [Aaron Weiss (aatxe)](https://github.com/aatxe) in the
//! [irc](https://github.com/aatxe/irc) crate. We are grateful for Aaron's
//! foundational work on IRC protocol handling in Rust.

pub mod caps;
pub mod chan;
pub mod colors;
pub mod command;
pub mod ctcp;
pub mod error;
#[cfg(feature = "tokio")]
pub mod irc;
#[cfg(feature = "tokio")]
pub mod line;
pub mod message;
pub mod mode;
pub mod prefix;
pub mod response;
pub mod sasl;
pub mod isupport;

pub use self::caps::{Capability, NegotiationVersion};
pub use self::chan::ChannelExt;
pub use self::colors::FormattedStringExt;
pub use self::command::{BatchSubCommand, CapSubCommand, Command};
pub use self::ctcp::{Ctcp, CtcpKind, CtcpOwned};

pub use self::command::{CommandRef, CommandRefEnum};
#[cfg(feature = "tokio")]
pub use self::irc::IrcCodec;
pub use self::message::{Message, Tag};
pub use self::mode::{ChannelMode, Mode, UserMode};
pub use self::prefix::Prefix;
pub use self::prefix::PrefixRef;
pub use self::message::MessageRef;
pub use self::response::Response;
pub use self::sasl::{SaslMechanism, SaslState, encode_plain, encode_external};
pub use self::isupport::{Isupport, IsupportEntry, PrefixSpec, ChanModes, TargMax, MaxList};

pub mod casemap;
pub use self::casemap::{irc_to_lower, irc_eq};

pub mod ircv3;
pub use self::ircv3::{generate_msgid, generate_batch_ref, format_server_time, format_timestamp};
pub mod scanner;
pub use scanner::{is_non_irc_protocol, detect_protocol, DetectedProtocol};

#[cfg(feature = "tokio")]
pub mod transport;
#[cfg(feature = "tokio")]
pub use self::transport::{Transport, TransportReadError, MAX_IRC_LINE_LEN};

#[cfg(feature = "tokio")]
pub mod websocket;
#[cfg(feature = "tokio")]
pub use self::websocket::{WebSocketConfig, HandshakeResult, validate_handshake, build_handshake_response};

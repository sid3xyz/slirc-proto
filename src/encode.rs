//! Zero-copy encoding for IRC messages.
//!
//! This module provides the [`IrcEncode`] trait for writing IRC messages directly
//! to byte buffers without intermediate `String` allocations.
//!
//! # Design
//!
//! The standard `Display` trait formats to a `String`, which requires allocation.
//! For high-performance servers handling thousands of messages per second,
//! `IrcEncode` writes directly to any `Write` implementor (sockets, `BytesMut`, etc.).
//!
//! # Example
//!
//! ```
//! use slirc_proto::encode::IrcEncode;
//! use slirc_proto::Message;
//!
//! let msg = Message::privmsg("#channel", "Hello!");
//! let mut buf = Vec::new();
//! msg.encode(&mut buf).unwrap();
//!
//! assert_eq!(&buf, b"PRIVMSG #channel :Hello!\r\n");
//! ```

use std::io::{self, Write};

use crate::command::Command;
use crate::message::tags::escape_tag_value_to_writer;
use crate::message::{Message, MessageRef, Tag};
use crate::mode::{Mode, ModeType};
use crate::prefix::Prefix;

/// A trait for encoding IRC protocol elements directly to a byte stream.
///
/// This provides zero-copy encoding by writing directly to any [`Write`]
/// implementor, avoiding the intermediate `String` allocation that `Display` requires.
///
/// # Implementors
///
/// - [`Message`] - Owned IRC message
/// - [`MessageRef`] - Borrowed IRC message
/// - [`Command`] - IRC command
/// - [`Prefix`] - Message source/prefix
pub trait IrcEncode {
    /// Encode this value to the given writer.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the write fails.
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<usize>;

    /// Encode this value to a new `Vec<u8>`.
    ///
    /// This is a convenience method for cases where you need a buffer.
    /// For optimal performance, prefer [`encode`](Self::encode) with a pre-allocated buffer.
    #[must_use]
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512); // IRC max line length
        let _ = self.encode(&mut buf);
        buf
    }
}

// ============================================================================
// Helper functions (zero-copy versions of serialize.rs helpers)
// ============================================================================

/// Check if a string needs colon-prefixing as a trailing IRC argument.
#[inline]
fn needs_colon_prefix(s: &str) -> bool {
    s.is_empty() || s.contains(' ') || s.starts_with(':')
}

/// Write a command with arguments. The last argument gets a `:` prefix if needed.
fn write_cmd<W: Write>(w: &mut W, cmd: &[u8], args: &[&str]) -> io::Result<usize> {
    let mut written = w.write(cmd)?;

    if args.is_empty() {
        return Ok(written);
    }

    let (middle, trailing) = args.split_at(args.len() - 1);
    let trailing = trailing[0];

    for param in middle {
        written += w.write(b" ")?;
        written += w.write(param.as_bytes())?;
    }

    written += w.write(b" ")?;

    if needs_colon_prefix(trailing) {
        written += w.write(b":")?;
    }

    written += w.write(trailing.as_bytes())?;
    Ok(written)
}

/// Write a command with a freeform (always colon-prefixed) trailing argument.
fn write_cmd_freeform<W: Write>(w: &mut W, cmd: &[u8], args: &[&str]) -> io::Result<usize> {
    let mut written = w.write(cmd)?;

    if args.is_empty() {
        return Ok(written);
    }

    let (middle, last) = args.split_at(args.len() - 1);

    for arg in middle {
        written += w.write(b" ")?;
        written += w.write(arg.as_bytes())?;
    }

    written += w.write(b" :")?;
    written += w.write(last[0].as_bytes())?;
    Ok(written)
}

/// Write mode flags with collapsed signs (e.g., +ovh instead of +o+v+h).
fn write_collapsed_mode_flags<W: Write, T: ModeType>(
    w: &mut W,
    modes: &[Mode<T>],
) -> io::Result<usize> {
    #[derive(PartialEq, Clone, Copy)]
    enum Sign {
        Plus,
        Minus,
        None,
    }

    let mut written = 0;
    let mut current_sign = Sign::None;

    for m in modes {
        let (new_sign, mode) = match m {
            Mode::Plus(mode, _) => (Sign::Plus, mode),
            Mode::Minus(mode, _) => (Sign::Minus, mode),
            Mode::NoPrefix(mode) => (Sign::None, mode),
        };

        if new_sign != current_sign {
            match new_sign {
                Sign::Plus => written += w.write(b"+")?,
                Sign::Minus => written += w.write(b"-")?,
                Sign::None => {}
            }
            current_sign = new_sign;
        }

        written += w.write(mode.to_string().as_bytes())?;
    }

    Ok(written)
}

/// Write service command arguments with trailing colon prefix.
fn write_service_args<W: Write>(w: &mut W, args: &[String]) -> io::Result<usize> {
    let mut written = 0;
    let len = args.len();

    for (i, arg) in args.iter().enumerate() {
        written += w.write(b" ")?;
        if i == len - 1 && needs_colon_prefix(arg) {
            written += w.write(b":")?;
        }
        written += w.write(arg.as_bytes())?;
    }

    Ok(written)
}

// ============================================================================
// IrcEncode implementations
// ============================================================================

impl IrcEncode for Message {
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<usize> {
        let mut written = 0;

        // Tags
        if let Some(ref tags) = self.tags {
            written += w.write(b"@")?;
            for (i, tag) in tags.iter().enumerate() {
                if i > 0 {
                    written += w.write(b";")?;
                }
                written += encode_tag(w, tag)?;
            }
            written += w.write(b" ")?;
        }

        // Prefix
        if let Some(ref prefix) = self.prefix {
            written += w.write(b":")?;
            written += prefix.encode(w)?;
            written += w.write(b" ")?;
        }

        // Command
        written += self.command.encode(w)?;

        // CRLF
        written += w.write(b"\r\n")?;

        Ok(written)
    }
}

impl<'a> IrcEncode for MessageRef<'a> {
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<usize> {
        let mut written = 0;

        // Tags (raw, already formatted)
        if let Some(tags) = self.tags {
            written += w.write(b"@")?;
            written += w.write(tags.as_bytes())?;
            written += w.write(b" ")?;
        }

        // Prefix
        if let Some(ref prefix) = self.prefix {
            written += w.write(b":")?;
            written += w.write(prefix.raw.as_bytes())?;
            written += w.write(b" ")?;
        }

        // Command (raw)
        written += w.write(self.command.name.as_bytes())?;
        for arg in &self.command.args {
            written += w.write(b" ")?;
            written += w.write(arg.as_bytes())?;
        }

        // CRLF
        written += w.write(b"\r\n")?;

        Ok(written)
    }
}

impl IrcEncode for Prefix {
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<usize> {
        match self {
            Prefix::ServerName(name) => w.write(name.as_bytes()),
            Prefix::Nickname(nick, user, host) => {
                let mut written = w.write(nick.as_bytes())?;
                if !user.is_empty() {
                    written += w.write(b"!")?;
                    written += w.write(user.as_bytes())?;
                }
                if !host.is_empty() {
                    written += w.write(b"@")?;
                    written += w.write(host.as_bytes())?;
                }
                Ok(written)
            }
        }
    }
}

/// Encode a single tag to the writer.
fn encode_tag<W: Write>(w: &mut W, tag: &Tag) -> io::Result<usize> {
    let mut written = w.write(tag.0.as_bytes())?;
    if let Some(ref value) = tag.1 {
        written += w.write(b"=")?;
        written += escape_tag_value_to_writer(w, value)?;
    }
    Ok(written)
}

impl IrcEncode for Command {
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<usize> {
        use crate::command::subcommands::ChatHistorySubCommand;

        match self {
            Command::PASS(p) => write_cmd(w, b"PASS", &[p]),
            Command::NICK(n) => write_cmd(w, b"NICK", &[n]),
            Command::USER(u, m, r) => write_cmd_freeform(w, b"USER", &[u, m, "*", r]),
            Command::OPER(u, p) => write_cmd(w, b"OPER", &[u, p]),
            Command::UserMODE(u, modes) => {
                let mut written = w.write(b"MODE ")?;
                written += w.write(u.as_bytes())?;
                if !modes.is_empty() {
                    written += w.write(b" ")?;
                    written += write_collapsed_mode_flags(w, modes)?;
                }
                Ok(written)
            }
            Command::SERVICE(nick, r0, dist, typ, r1, info) => {
                write_cmd_freeform(w, b"SERVICE", &[nick, r0, dist, typ, r1, info])
            }
            Command::QUIT(Some(m)) => write_cmd(w, b"QUIT", &[m]),
            Command::QUIT(None) => w.write(b"QUIT"),
            Command::SQUIT(s, c) => write_cmd_freeform(w, b"SQUIT", &[s, c]),

            // Channel Operations
            Command::JOIN(c, Some(k), Some(n)) => write_cmd(w, b"JOIN", &[c, k, n]),
            Command::JOIN(c, Some(k), None) => write_cmd(w, b"JOIN", &[c, k]),
            Command::JOIN(c, None, Some(n)) => write_cmd(w, b"JOIN", &[c, n]),
            Command::JOIN(c, None, None) => write_cmd(w, b"JOIN", &[c]),
            Command::PART(c, Some(m)) => write_cmd_freeform(w, b"PART", &[c, m]),
            Command::PART(c, None) => write_cmd(w, b"PART", &[c]),
            Command::ChannelMODE(c, modes) => {
                let mut written = w.write(b"MODE ")?;
                written += w.write(c.as_bytes())?;
                if !modes.is_empty() {
                    written += w.write(b" ")?;
                    written += write_collapsed_mode_flags(w, modes)?;
                    for m in modes {
                        if let Some(arg) = m.arg() {
                            written += w.write(b" ")?;
                            written += w.write(arg.as_bytes())?;
                        }
                    }
                }
                Ok(written)
            }
            Command::TOPIC(c, Some(t)) => write_cmd_freeform(w, b"TOPIC", &[c, t]),
            Command::TOPIC(c, None) => write_cmd(w, b"TOPIC", &[c]),
            Command::NAMES(Some(c), Some(t)) => write_cmd(w, b"NAMES", &[c, t]),
            Command::NAMES(Some(c), None) => write_cmd(w, b"NAMES", &[c]),
            Command::NAMES(None, _) => w.write(b"NAMES"),
            Command::LIST(Some(c), Some(t)) => write_cmd(w, b"LIST", &[c, t]),
            Command::LIST(Some(c), None) => write_cmd(w, b"LIST", &[c]),
            Command::LIST(None, _) => w.write(b"LIST"),
            Command::INVITE(n, c) => write_cmd_freeform(w, b"INVITE", &[n, c]),
            Command::KICK(c, n, Some(r)) => write_cmd_freeform(w, b"KICK", &[c, n, r]),
            Command::KICK(c, n, None) => write_cmd(w, b"KICK", &[c, n]),

            // Messaging
            Command::PRIVMSG(t, m) => write_cmd_freeform(w, b"PRIVMSG", &[t, m]),
            Command::NOTICE(t, m) => write_cmd_freeform(w, b"NOTICE", &[t, m]),

            // Server Queries
            Command::MOTD(Some(t)) => write_cmd(w, b"MOTD", &[t]),
            Command::MOTD(None) => w.write(b"MOTD"),
            Command::LUSERS(Some(m), Some(t)) => write_cmd(w, b"LUSERS", &[m, t]),
            Command::LUSERS(Some(m), None) => write_cmd(w, b"LUSERS", &[m]),
            Command::LUSERS(None, _) => w.write(b"LUSERS"),
            Command::VERSION(Some(t)) => write_cmd(w, b"VERSION", &[t]),
            Command::VERSION(None) => w.write(b"VERSION"),
            Command::STATS(Some(q), Some(t)) => write_cmd(w, b"STATS", &[q, t]),
            Command::STATS(Some(q), None) => write_cmd(w, b"STATS", &[q]),
            Command::STATS(None, _) => w.write(b"STATS"),
            Command::LINKS(Some(r), Some(s)) => write_cmd(w, b"LINKS", &[r, s]),
            Command::LINKS(None, Some(s)) => write_cmd(w, b"LINKS", &[s]),
            Command::LINKS(_, None) => w.write(b"LINKS"),
            Command::TIME(Some(t)) => write_cmd(w, b"TIME", &[t]),
            Command::TIME(None) => w.write(b"TIME"),
            Command::CONNECT(t, p, Some(r)) => write_cmd(w, b"CONNECT", &[t, p, r]),
            Command::CONNECT(t, p, None) => write_cmd(w, b"CONNECT", &[t, p]),
            Command::TRACE(Some(t)) => write_cmd(w, b"TRACE", &[t]),
            Command::TRACE(None) => w.write(b"TRACE"),
            Command::ADMIN(Some(t)) => write_cmd(w, b"ADMIN", &[t]),
            Command::ADMIN(None) => w.write(b"ADMIN"),
            Command::INFO(Some(t)) => write_cmd(w, b"INFO", &[t]),
            Command::INFO(None) => w.write(b"INFO"),
            Command::SERVLIST(Some(m), Some(t)) => write_cmd(w, b"SERVLIST", &[m, t]),
            Command::SERVLIST(Some(m), None) => write_cmd(w, b"SERVLIST", &[m]),
            Command::SERVLIST(None, _) => w.write(b"SERVLIST"),
            Command::SQUERY(s, t) => write_cmd_freeform(w, b"SQUERY", &[s, t]),

            // User Queries
            Command::WHO(Some(s), Some(true)) => write_cmd(w, b"WHO", &[s, "o"]),
            Command::WHO(Some(s), _) => write_cmd(w, b"WHO", &[s]),
            Command::WHO(None, _) => w.write(b"WHO"),
            Command::WHOIS(Some(t), m) => write_cmd(w, b"WHOIS", &[t, m]),
            Command::WHOIS(None, m) => write_cmd(w, b"WHOIS", &[m]),
            Command::WHOWAS(n, Some(c), Some(t)) => write_cmd(w, b"WHOWAS", &[n, c, t]),
            Command::WHOWAS(n, Some(c), None) => write_cmd(w, b"WHOWAS", &[n, c]),
            Command::WHOWAS(n, None, _) => write_cmd(w, b"WHOWAS", &[n]),

            // Miscellaneous
            Command::KILL(n, c) => write_cmd_freeform(w, b"KILL", &[n, c]),
            Command::PING(s, Some(t)) => write_cmd(w, b"PING", &[s, t]),
            Command::PING(s, None) => write_cmd(w, b"PING", &[s]),
            Command::PONG(s, Some(t)) => write_cmd(w, b"PONG", &[s, t]),
            Command::PONG(s, None) => write_cmd(w, b"PONG", &[s]),
            Command::ERROR(m) => write_cmd_freeform(w, b"ERROR", &[m]),
            Command::AWAY(Some(m)) => write_cmd_freeform(w, b"AWAY", &[m]),
            Command::AWAY(None) => w.write(b"AWAY"),
            Command::REHASH => w.write(b"REHASH"),
            Command::DIE => w.write(b"DIE"),
            Command::RESTART => w.write(b"RESTART"),
            Command::SUMMON(u, Some(t), Some(c)) => write_cmd(w, b"SUMMON", &[u, t, c]),
            Command::SUMMON(u, Some(t), None) => write_cmd(w, b"SUMMON", &[u, t]),
            Command::SUMMON(u, None, _) => write_cmd(w, b"SUMMON", &[u]),
            Command::USERS(Some(t)) => write_cmd(w, b"USERS", &[t]),
            Command::USERS(None) => w.write(b"USERS"),
            Command::WALLOPS(t) => write_cmd_freeform(w, b"WALLOPS", &[t]),
            Command::USERHOST(u) => {
                let mut written = w.write(b"USERHOST")?;
                written += write_service_args(w, u)?;
                Ok(written)
            }
            Command::ISON(u) => {
                let mut written = w.write(b"ISON")?;
                written += write_service_args(w, u)?;
                Ok(written)
            }

            // Operator Ban Commands
            Command::KLINE(Some(t), m, r) => write_cmd_freeform(w, b"KLINE", &[t, m, r]),
            Command::KLINE(None, m, r) => write_cmd_freeform(w, b"KLINE", &[m, r]),
            Command::DLINE(Some(t), h, r) => write_cmd_freeform(w, b"DLINE", &[t, h, r]),
            Command::DLINE(None, h, r) => write_cmd_freeform(w, b"DLINE", &[h, r]),
            Command::UNKLINE(m) => write_cmd(w, b"UNKLINE", &[m]),
            Command::UNDLINE(h) => write_cmd(w, b"UNDLINE", &[h]),
            Command::KNOCK(c, Some(m)) => write_cmd_freeform(w, b"KNOCK", &[c, m]),
            Command::KNOCK(c, None) => write_cmd(w, b"KNOCK", &[c]),

            // Services Commands
            Command::SAJOIN(n, c) => write_cmd(w, b"SAJOIN", &[n, c]),
            Command::SAMODE(t, m, Some(p)) => write_cmd(w, b"SAMODE", &[t, m, p]),
            Command::SAMODE(t, m, None) => write_cmd(w, b"SAMODE", &[t, m]),
            Command::SANICK(o, n) => write_cmd(w, b"SANICK", &[o, n]),
            Command::SAPART(c, r) => write_cmd(w, b"SAPART", &[c, r]),
            Command::SAQUIT(c, r) => write_cmd(w, b"SAQUIT", &[c, r]),
            Command::NICKSERV(p) => {
                let mut written = w.write(b"NICKSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::CHANSERV(p) => {
                let mut written = w.write(b"CHANSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::OPERSERV(p) => {
                let mut written = w.write(b"OPERSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::BOTSERV(p) => {
                let mut written = w.write(b"BOTSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::HOSTSERV(p) => {
                let mut written = w.write(b"HOSTSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::MEMOSERV(p) => {
                let mut written = w.write(b"MEMOSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::NS(p) => {
                let mut written = w.write(b"NS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::CS(p) => {
                let mut written = w.write(b"CS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::OS(p) => {
                let mut written = w.write(b"OS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::BS(p) => {
                let mut written = w.write(b"BS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::HS(p) => {
                let mut written = w.write(b"HS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::MS(p) => {
                let mut written = w.write(b"MS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }

            // IRCv3 Extensions
            Command::CAP(target, subcmd, code, params) => {
                let mut written = w.write(b"CAP")?;
                if let Some(t) = target {
                    written += w.write(b" ")?;
                    written += w.write(t.as_bytes())?;
                }
                written += w.write(b" ")?;
                written += w.write(subcmd.to_str().as_bytes())?;
                if let Some(c) = code {
                    written += w.write(b" ")?;
                    written += w.write(c.as_bytes())?;
                }
                if let Some(p) = params {
                    written += w.write(b" ")?;
                    written += w.write(p.as_bytes())?;
                }
                Ok(written)
            }
            Command::AUTHENTICATE(d) => write_cmd(w, b"AUTHENTICATE", &[d]),
            Command::ACCOUNT(a) => write_cmd(w, b"ACCOUNT", &[a]),
            Command::MONITOR(c, Some(t)) => write_cmd(w, b"MONITOR", &[c, t]),
            Command::MONITOR(c, None) => write_cmd(w, b"MONITOR", &[c]),
            Command::BATCH(t, Some(c), Some(a)) => {
                let mut written = w.write(b"BATCH ")?;
                written += w.write(t.as_bytes())?;
                written += w.write(b" ")?;
                written += w.write(c.to_str().as_bytes())?;
                written += write_service_args(w, a)?;
                Ok(written)
            }
            Command::BATCH(t, Some(c), None) => write_cmd(w, b"BATCH", &[t, c.to_str()]),
            Command::BATCH(t, None, Some(a)) => {
                let mut written = w.write(b"BATCH ")?;
                written += w.write(t.as_bytes())?;
                written += write_service_args(w, a)?;
                Ok(written)
            }
            Command::BATCH(t, None, None) => write_cmd(w, b"BATCH", &[t]),
            Command::CHGHOST(u, h) => write_cmd(w, b"CHGHOST", &[u, h]),
            Command::SETNAME(r) => write_cmd_freeform(w, b"SETNAME", &[r]),
            Command::TAGMSG(t) => write_cmd(w, b"TAGMSG", &[t]),
            Command::ACK => w.write(b"ACK"),
            Command::WEBIRC(pass, gateway, host, ip, Some(opts)) => {
                write_cmd(w, b"WEBIRC", &[pass, gateway, host, ip, opts])
            }
            Command::WEBIRC(pass, gateway, host, ip, None) => {
                write_cmd(w, b"WEBIRC", &[pass, gateway, host, ip])
            }
            Command::CHATHISTORY {
                subcommand,
                target,
                msg_ref1,
                msg_ref2,
                limit,
            } => {
                let mut written = w.write(b"CHATHISTORY ")?;
                let subcmd_str = subcommand.to_string();
                written += w.write(subcmd_str.as_bytes())?;

                match subcommand {
                    ChatHistorySubCommand::TARGETS => {
                        let ref1_str = msg_ref1.to_string();
                        written += w.write(b" ")?;
                        written += w.write(ref1_str.as_bytes())?;
                        if let Some(ref2) = msg_ref2 {
                            let ref2_str = ref2.to_string();
                            written += w.write(b" ")?;
                            written += w.write(ref2_str.as_bytes())?;
                        }
                        written += w.write(b" ")?;
                        let limit_str = limit.to_string();
                        written += w.write(limit_str.as_bytes())?;
                    }
                    ChatHistorySubCommand::BETWEEN => {
                        let ref1_str = msg_ref1.to_string();
                        written += w.write(b" ")?;
                        written += w.write(target.as_bytes())?;
                        written += w.write(b" ")?;
                        written += w.write(ref1_str.as_bytes())?;
                        if let Some(ref2) = msg_ref2 {
                            let ref2_str = ref2.to_string();
                            written += w.write(b" ")?;
                            written += w.write(ref2_str.as_bytes())?;
                        }
                        written += w.write(b" ")?;
                        let limit_str = limit.to_string();
                        written += w.write(limit_str.as_bytes())?;
                    }
                    _ => {
                        let ref1_str = msg_ref1.to_string();
                        let limit_str = limit.to_string();
                        written += w.write(b" ")?;
                        written += w.write(target.as_bytes())?;
                        written += w.write(b" ")?;
                        written += w.write(ref1_str.as_bytes())?;
                        written += w.write(b" ")?;
                        written += w.write(limit_str.as_bytes())?;
                    }
                }
                Ok(written)
            }

            // Standard Replies
            Command::FAIL(command, code, context) => {
                write_standard_reply(w, b"FAIL", command, code, context)
            }
            Command::WARN(command, code, context) => {
                write_standard_reply(w, b"WARN", command, code, context)
            }
            Command::NOTE(command, code, context) => {
                write_standard_reply(w, b"NOTE", command, code, context)
            }

            // Numeric Response
            Command::Response(resp, args) => {
                let code = *resp as u16;
                let mut written = w.write(&[
                    b'0' + (code / 100) as u8,
                    b'0' + ((code / 10) % 10) as u8,
                    b'0' + (code % 10) as u8,
                ])?;

                let len = args.len();
                for (i, arg) in args.iter().enumerate() {
                    written += w.write(b" ")?;
                    if i == len - 1 && needs_colon_prefix(arg) {
                        written += w.write(b":")?;
                    }
                    written += w.write(arg.as_bytes())?;
                }
                Ok(written)
            }

            // Raw
            Command::Raw(cmd, args) => {
                let mut written = w.write(cmd.as_bytes())?;
                written += write_service_args(w, args)?;
                Ok(written)
            }
        }
    }
}

/// Write a standard reply (FAIL/WARN/NOTE).
fn write_standard_reply<W: Write>(
    w: &mut W,
    reply_type: &[u8],
    command: &str,
    code: &str,
    context: &[String],
) -> io::Result<usize> {
    let mut written = w.write(reply_type)?;
    written += w.write(b" ")?;
    written += w.write(command.as_bytes())?;
    written += w.write(b" ")?;
    written += w.write(code.as_bytes())?;

    let len = context.len();
    for (i, arg) in context.iter().enumerate() {
        written += w.write(b" ")?;
        if i == len - 1 {
            written += w.write(b":")?;
        }
        written += w.write(arg.as_bytes())?;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_privmsg() {
        let msg = Message::privmsg("#channel", "Hello world!");
        let bytes = msg.to_bytes();
        assert_eq!(&bytes, b"PRIVMSG #channel :Hello world!\r\n");
    }

    #[test]
    fn test_encode_simple_command() {
        let msg = Message::nick("testnick");
        let bytes = msg.to_bytes();
        assert_eq!(&bytes, b"NICK testnick\r\n");
    }

    #[test]
    fn test_encode_with_prefix() {
        let msg = Message::privmsg("#test", "Hello")
            .with_prefix(Prefix::new_from_str("nick!user@host"));
        let bytes = msg.to_bytes();
        assert_eq!(&bytes, b":nick!user@host PRIVMSG #test :Hello\r\n");
    }

    #[test]
    fn test_encode_with_tags() {
        let msg = Message::privmsg("#test", "Hi")
            .with_tag("time", Some("2023-01-01T00:00:00Z"));
        let bytes = msg.to_bytes();
        assert_eq!(
            &bytes,
            b"@time=2023-01-01T00:00:00Z PRIVMSG #test :Hi\r\n"
        );
    }

    #[test]
    fn test_encode_returns_byte_count() {
        let msg = Message::ping("server");
        let mut buf = Vec::new();
        let written = msg.encode(&mut buf).unwrap();
        assert_eq!(written, buf.len());
    }
}

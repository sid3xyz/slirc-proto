//! Command encoding implementation.

use std::io::{self, Write};

use crate::command::Command;
use crate::mode::{Mode, ModeType};

use super::{needs_colon_prefix, write_cmd, write_cmd_freeform, write_service_args, IrcEncode};

/// Write mode flags with collapsed signs (e.g., +ovh instead of +o+v+h).
pub(crate) fn write_collapsed_mode_flags<W: Write, T: ModeType>(
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

//! Command encoding implementation.

use std::io::{self, Write};

use crate::command::Command;
use crate::command::util::{
    write_args_with_trailing, write_cmd, write_cmd_freeform, write_collapsed_mode_flags,
    write_service_args, write_standard_reply, needs_colon_prefix, IrcSink, IoWriteSink,
};
use super::IrcEncode;

impl IrcEncode for Command {
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<usize> {
        use crate::command::subcommands::ChatHistorySubCommand;
        let mut sink = IoWriteSink(w);
        let w = &mut sink;

        match self {
            Command::PASS(p) => write_cmd(w, "PASS", &[p]),
            Command::NICK(n) => write_cmd(w, "NICK", &[n]),
            Command::USER(u, m, r) => write_cmd_freeform(w, "USER", &[u, m, "*", r]),
            Command::OPER(u, p) => write_cmd(w, "OPER", &[u, p]),
            Command::UserMODE(u, modes) => {
                let mut written = w.write_str("MODE ")?;
                written += w.write_str(u)?;
                if !modes.is_empty() {
                    written += w.write_char(' ')?;
                    written += write_collapsed_mode_flags(w, modes)?;
                }
                Ok(written)
            }
            Command::SERVICE(nick, r0, dist, typ, r1, info) => {
                write_cmd_freeform(w, "SERVICE", &[nick, r0, dist, typ, r1, info])
            }
            Command::QUIT(Some(m)) => write_cmd(w, "QUIT", &[m]),
            Command::QUIT(None) => w.write_str("QUIT"),
            Command::SQUIT(s, c) => write_cmd_freeform(w, "SQUIT", &[s, c]),

            // Channel Operations
            Command::JOIN(c, Some(k), Some(n)) => write_cmd(w, "JOIN", &[c, k, n]),
            Command::JOIN(c, Some(k), None) => write_cmd(w, "JOIN", &[c, k]),
            Command::JOIN(c, None, Some(n)) => write_cmd(w, "JOIN", &[c, n]),
            Command::JOIN(c, None, None) => write_cmd(w, "JOIN", &[c]),
            Command::PART(c, Some(m)) => write_cmd_freeform(w, "PART", &[c, m]),
            Command::PART(c, None) => write_cmd(w, "PART", &[c]),
            Command::ChannelMODE(c, modes) => {
                let mut written = w.write_str("MODE ")?;
                written += w.write_str(c)?;
                if !modes.is_empty() {
                    written += w.write_char(' ')?;
                    written += write_collapsed_mode_flags(w, modes)?;
                    for m in modes {
                        if let Some(arg) = m.arg() {
                            written += w.write_char(' ')?;
                            written += w.write_str(arg)?;
                        }
                    }
                }
                Ok(written)
            }
            Command::TOPIC(c, Some(t)) => write_cmd_freeform(w, "TOPIC", &[c, t]),
            Command::TOPIC(c, None) => write_cmd(w, "TOPIC", &[c]),
            Command::NAMES(Some(c), Some(t)) => write_cmd(w, "NAMES", &[c, t]),
            Command::NAMES(Some(c), None) => write_cmd(w, "NAMES", &[c]),
            Command::NAMES(None, _) => w.write_str("NAMES"),
            Command::LIST(Some(c), Some(t)) => write_cmd(w, "LIST", &[c, t]),
            Command::LIST(Some(c), None) => write_cmd(w, "LIST", &[c]),
            Command::LIST(None, _) => w.write_str("LIST"),
            Command::INVITE(n, c) => write_cmd_freeform(w, "INVITE", &[n, c]),
            Command::KICK(c, n, Some(r)) => write_cmd_freeform(w, "KICK", &[c, n, r]),
            Command::KICK(c, n, None) => write_cmd(w, "KICK", &[c, n]),

            // Messaging
            Command::PRIVMSG(t, m) => write_cmd_freeform(w, "PRIVMSG", &[t, m]),
            Command::NOTICE(t, m) => write_cmd_freeform(w, "NOTICE", &[t, m]),
            Command::ACCEPT(n) => write_cmd(w, "ACCEPT", &[n]),

            // Server Queries
            Command::MOTD(Some(t)) => write_cmd(w, "MOTD", &[t]),
            Command::MOTD(None) => w.write_str("MOTD"),
            Command::LUSERS(Some(m), Some(t)) => write_cmd(w, "LUSERS", &[m, t]),
            Command::LUSERS(Some(m), None) => write_cmd(w, "LUSERS", &[m]),
            Command::LUSERS(None, _) => w.write_str("LUSERS"),
            Command::VERSION(Some(t)) => write_cmd(w, "VERSION", &[t]),
            Command::VERSION(None) => w.write_str("VERSION"),
            Command::STATS(Some(q), Some(t)) => write_cmd(w, "STATS", &[q, t]),
            Command::STATS(Some(q), None) => write_cmd(w, "STATS", &[q]),
            Command::STATS(None, _) => w.write_str("STATS"),
            Command::LINKS(Some(r), Some(s)) => write_cmd(w, "LINKS", &[r, s]),
            Command::LINKS(None, Some(s)) => write_cmd(w, "LINKS", &[s]),
            Command::LINKS(_, None) => w.write_str("LINKS"),
            Command::TIME(Some(t)) => write_cmd(w, "TIME", &[t]),
            Command::TIME(None) => w.write_str("TIME"),
            Command::CONNECT(t, p, Some(r)) => write_cmd(w, "CONNECT", &[t, p, r]),
            Command::CONNECT(t, p, None) => write_cmd(w, "CONNECT", &[t, p]),
            Command::TRACE(Some(t)) => write_cmd(w, "TRACE", &[t]),
            Command::TRACE(None) => w.write_str("TRACE"),
            Command::ADMIN(Some(t)) => write_cmd(w, "ADMIN", &[t]),
            Command::ADMIN(None) => w.write_str("ADMIN"),
            Command::INFO(Some(t)) => write_cmd(w, "INFO", &[t]),
            Command::INFO(None) => w.write_str("INFO"),
            Command::SID(name, hop, sid, desc) => {
                write_cmd_freeform(w, "SID", &[name, hop, sid, desc])
            }
            Command::UID(nick, hop, ts, user, host, uid, modes, real) => {
                write_cmd_freeform(w, "UID", &[nick, hop, ts, user, host, uid, modes, real])
            }
            Command::SJOIN(ts, channel, modes, args, users) => {
                let mut written = w.write_str("SJOIN ")?;
                written += w.write_str(&ts.to_string())?;
                written += w.write_char(' ')?;
                written += w.write_str(channel)?;
                written += w.write_char(' ')?;
                written += w.write_str(modes)?;
                for arg in args {
                    written += w.write_char(' ')?;
                    written += w.write_str(arg)?;
                }
                written += w.write_str(" :")?;
                for (i, (prefixes, uid)) in users.iter().enumerate() {
                    if i > 0 {
                        written += w.write_char(' ')?;
                    }
                    written += w.write_str(prefixes)?;
                    written += w.write_str(uid)?;
                }
                Ok(written)
            }
            Command::TMODE(ts, channel, modes, args) => {
                let mut written = w.write_str("TMODE ")?;
                written += w.write_str(&ts.to_string())?;
                written += w.write_char(' ')?;
                written += w.write_str(channel)?;
                written += w.write_char(' ')?;
                written += w.write_str(modes)?;
                for arg in args {
                    written += w.write_char(' ')?;
                    written += w.write_str(arg)?;
                }
                Ok(written)
            }
            Command::MAP => w.write_str("MAP"),
            Command::RULES => w.write_str("RULES"),
            Command::USERIP(u) => {
                let mut written = w.write_str("USERIP")?;
                written += write_service_args(w, u)?;
                Ok(written)
            }
            Command::HELP(Some(t)) => write_cmd(w, "HELP", &[t]),
            Command::HELP(None) => w.write_str("HELP"),
            Command::SERVLIST(Some(m), Some(t)) => write_cmd(w, "SERVLIST", &[m, t]),
            Command::SERVLIST(Some(m), None) => write_cmd(w, "SERVLIST", &[m]),
            Command::SERVLIST(None, _) => w.write_str("SERVLIST"),
            Command::SQUERY(s, t) => write_cmd_freeform(w, "SQUERY", &[s, t]),

            // User Queries
            Command::WHO(Some(s), Some(true)) => write_cmd(w, "WHO", &[s, "o"]),
            Command::WHO(Some(s), _) => write_cmd(w, "WHO", &[s]),
            Command::WHO(None, _) => w.write_str("WHO"),
            Command::WHOIS(Some(t), m) => write_cmd(w, "WHOIS", &[t, m]),
            Command::WHOIS(None, m) => write_cmd(w, "WHOIS", &[m]),
            Command::WHOWAS(n, Some(c), Some(t)) => write_cmd(w, "WHOWAS", &[n, c, t]),
            Command::WHOWAS(n, Some(c), None) => write_cmd(w, "WHOWAS", &[n, c]),
            Command::WHOWAS(n, None, _) => write_cmd(w, "WHOWAS", &[n]),

            // Miscellaneous
            Command::KILL(n, c) => write_cmd_freeform(w, "KILL", &[n, c]),
            Command::PING(s, Some(t)) => write_cmd(w, "PING", &[s, t]),
            Command::PING(s, None) => write_cmd(w, "PING", &[s]),
            Command::PONG(s, Some(t)) => write_cmd(w, "PONG", &[s, t]),
            Command::PONG(s, None) => write_cmd(w, "PONG", &[s]),
            Command::ERROR(m) => write_cmd_freeform(w, "ERROR", &[m]),
            Command::AWAY(Some(m)) => write_cmd_freeform(w, "AWAY", &[m]),
            Command::AWAY(None) => w.write_str("AWAY"),
            Command::REHASH => w.write_str("REHASH"),
            Command::DIE => w.write_str("DIE"),
            Command::RESTART => w.write_str("RESTART"),
            Command::SUMMON(u, Some(t), Some(c)) => write_cmd(w, "SUMMON", &[u, t, c]),
            Command::SUMMON(u, Some(t), None) => write_cmd(w, "SUMMON", &[u, t]),
            Command::SUMMON(u, None, _) => write_cmd(w, "SUMMON", &[u]),
            Command::USERS(Some(t)) => write_cmd(w, "USERS", &[t]),
            Command::USERS(None) => w.write_str("USERS"),
            Command::WALLOPS(t) => write_cmd_freeform(w, "WALLOPS", &[t]),
            Command::GLOBOPS(t) => write_cmd_freeform(w, "GLOBOPS", &[t]),
            Command::USERHOST(u) => {
                let mut written = w.write_str("USERHOST")?;
                written += write_service_args(w, u)?;
                Ok(written)
            }
            Command::ISON(u) => {
                let mut written = w.write_str("ISON")?;
                written += write_service_args(w, u)?;
                Ok(written)
            }

            // Operator Ban Commands
            Command::KLINE(Some(t), m, r) => write_cmd_freeform(w, "KLINE", &[t, m, r]),
            Command::KLINE(None, m, r) => write_cmd_freeform(w, "KLINE", &[m, r]),
            Command::DLINE(Some(t), h, r) => write_cmd_freeform(w, "DLINE", &[t, h, r]),
            Command::DLINE(None, h, r) => write_cmd_freeform(w, "DLINE", &[h, r]),
            Command::UNKLINE(m) => write_cmd(w, "UNKLINE", &[m]),
            Command::UNDLINE(h) => write_cmd(w, "UNDLINE", &[h]),
            Command::GLINE(m, Some(r)) => write_cmd_freeform(w, "GLINE", &[m, r]),
            Command::GLINE(m, None) => write_cmd(w, "GLINE", &[m]),
            Command::UNGLINE(m) => write_cmd(w, "UNGLINE", &[m]),
            Command::ZLINE(ip, Some(r)) => write_cmd_freeform(w, "ZLINE", &[ip, r]),
            Command::ZLINE(ip, None) => write_cmd(w, "ZLINE", &[ip]),
            Command::UNZLINE(ip) => write_cmd(w, "UNZLINE", &[ip]),
            Command::RLINE(p, Some(r)) => write_cmd_freeform(w, "RLINE", &[p, r]),
            Command::RLINE(p, None) => write_cmd(w, "RLINE", &[p]),
            Command::UNRLINE(p) => write_cmd(w, "UNRLINE", &[p]),
            Command::SHUN(m, Some(r)) => write_cmd_freeform(w, "SHUN", &[m, r]),
            Command::SHUN(m, None) => write_cmd(w, "SHUN", &[m]),
            Command::UNSHUN(m) => write_cmd(w, "UNSHUN", &[m]),
            Command::KNOCK(c, Some(m)) => write_cmd_freeform(w, "KNOCK", &[c, m]),
            Command::KNOCK(c, None) => write_cmd(w, "KNOCK", &[c]),

            // Server-to-Server
            Command::SERVER(n, h, t, i) => {
                write_cmd_freeform(w, "SERVER", &[n, &h.to_string(), t, i])
            }
            Command::BURST(t, p) => write_cmd_freeform(w, "BURST", &[t, p]),
            Command::DELTA(t, p) => write_cmd_freeform(w, "DELTA", &[t, p]),

            // Services Commands
            Command::SAJOIN(n, c) => write_cmd(w, "SAJOIN", &[n, c]),
            Command::SAMODE(t, m, Some(p)) => write_cmd(w, "SAMODE", &[t, m, p]),
            Command::SAMODE(t, m, None) => write_cmd(w, "SAMODE", &[t, m]),
            Command::SANICK(o, n) => write_cmd(w, "SANICK", &[o, n]),
            Command::SAPART(c, r) => write_cmd(w, "SAPART", &[c, r]),
            Command::SAQUIT(c, r) => write_cmd(w, "SAQUIT", &[c, r]),
            Command::NICKSERV(p) => {
                let mut written = w.write_str("NICKSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::CHANSERV(p) => {
                let mut written = w.write_str("CHANSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::OPERSERV(p) => {
                let mut written = w.write_str("OPERSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::BOTSERV(p) => {
                let mut written = w.write_str("BOTSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::HOSTSERV(p) => {
                let mut written = w.write_str("HOSTSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::MEMOSERV(p) => {
                let mut written = w.write_str("MEMOSERV")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::NS(p) => {
                let mut written = w.write_str("NS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::CS(p) => {
                let mut written = w.write_str("CS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::OS(p) => {
                let mut written = w.write_str("OS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::BS(p) => {
                let mut written = w.write_str("BS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::HS(p) => {
                let mut written = w.write_str("HS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }
            Command::MS(p) => {
                let mut written = w.write_str("MS")?;
                written += write_service_args(w, p)?;
                Ok(written)
            }

            // IRCv3 Extensions
            Command::CAP(target, subcmd, code, params) => {
                let mut written = w.write_str("CAP")?;
                if let Some(t) = target {
                    written += w.write_char(' ')?;
                    written += w.write_str(t)?;
                }
                written += w.write_char(' ')?;
                written += w.write_str(subcmd.to_str())?;
                if let Some(c) = code {
                    written += w.write_char(' ')?;
                    written += w.write_str(c)?;
                }
                if let Some(p) = params {
                    written += w.write_char(' ')?;
                    written += w.write_str(p)?;
                }
                Ok(written)
            }
            Command::AUTHENTICATE(d) => write_cmd(w, "AUTHENTICATE", &[d]),
            Command::ACCOUNT(a) => write_cmd(w, "ACCOUNT", &[a]),
            Command::MONITOR(c, Some(t)) => write_cmd(w, "MONITOR", &[c, t]),
            Command::MONITOR(c, None) => write_cmd(w, "MONITOR", &[c]),
            Command::BATCH(t, Some(c), Some(a)) => {
                let mut written = w.write_str("BATCH ")?;
                written += w.write_str(t)?;
                written += w.write_char(' ')?;
                written += w.write_str(c.to_str())?;
                written += write_service_args(w, a)?;
                Ok(written)
            }
            Command::BATCH(t, Some(c), None) => write_cmd(w, "BATCH", &[t, c.to_str()]),
            Command::BATCH(t, None, Some(a)) => {
                let mut written = w.write_str("BATCH ")?;
                written += w.write_str(t)?;
                written += write_service_args(w, a)?;
                Ok(written)
            }
            Command::BATCH(t, None, None) => write_cmd(w, "BATCH", &[t]),
            Command::CHGHOST(u, h) => write_cmd(w, "CHGHOST", &[u, h]),
            Command::CHGIDENT(u, i) => write_cmd(w, "CHGIDENT", &[u, i]),
            Command::SETNAME(r) => write_cmd_freeform(w, "SETNAME", &[r]),
            Command::TAGMSG(t) => write_cmd(w, "TAGMSG", &[t]),
            Command::ACK => w.write_str("ACK"),
            Command::WEBIRC(pass, gateway, host, ip, Some(opts)) => {
                write_cmd(w, "WEBIRC", &[pass, gateway, host, ip, opts])
            }
            Command::WEBIRC(pass, gateway, host, ip, None) => {
                write_cmd(w, "WEBIRC", &[pass, gateway, host, ip])
            }
            Command::CHATHISTORY {
                subcommand,
                target,
                msg_ref1,
                msg_ref2,
                limit,
            } => {
                let mut written = w.write_str("CHATHISTORY ")?;
                let subcmd_str = subcommand.to_string();
                written += w.write_str(&subcmd_str)?;

                match subcommand {
                    ChatHistorySubCommand::TARGETS => {
                        let ref1_str = msg_ref1.to_string();
                        written += w.write_char(' ')?;
                        written += w.write_str(&ref1_str)?;
                        if let Some(ref2) = msg_ref2 {
                            let ref2_str = ref2.to_string();
                            written += w.write_char(' ')?;
                            written += w.write_str(&ref2_str)?;
                        }
                        written += w.write_char(' ')?;
                        let limit_str = limit.to_string();
                        written += w.write_str(&limit_str)?;
                    }
                    ChatHistorySubCommand::BETWEEN => {
                        let ref1_str = msg_ref1.to_string();
                        written += w.write_char(' ')?;
                        written += w.write_str(target)?;
                        written += w.write_char(' ')?;
                        written += w.write_str(&ref1_str)?;
                        if let Some(ref2) = msg_ref2 {
                            let ref2_str = ref2.to_string();
                            written += w.write_char(' ')?;
                            written += w.write_str(&ref2_str)?;
                        }
                        written += w.write_char(' ')?;
                        let limit_str = limit.to_string();
                        written += w.write_str(&limit_str)?;
                    }
                    _ => {
                        let ref1_str = msg_ref1.to_string();
                        let limit_str = limit.to_string();
                        written += w.write_char(' ')?;
                        written += w.write_str(target)?;
                        written += w.write_char(' ')?;
                        written += w.write_str(&ref1_str)?;
                        written += w.write_char(' ')?;
                        written += w.write_str(&limit_str)?;
                    }
                }
                Ok(written)
            }

            // Standard Replies
            Command::FAIL(command, code, context) => {
                write_standard_reply(w, "FAIL", command, code, context)
            }
            Command::WARN(command, code, context) => {
                write_standard_reply(w, "WARN", command, code, context)
            }
            Command::NOTE(command, code, context) => {
                write_standard_reply(w, "NOTE", command, code, context)
            }

            // Numeric Response
            Command::Response(resp, args) => {
                let code = *resp as u16;
                let mut written = w.write_fmt(format_args!("{:03}", code))?;

                let len = args.len();
                for (i, arg) in args.iter().enumerate() {
                    written += w.write_char(' ')?;
                    if i == len - 1 && needs_colon_prefix(arg) {
                        written += w.write_char(':')?;
                    }
                    written += w.write_str(arg)?;
                }
                Ok(written)
            }

            // Raw
            Command::Raw(cmd, args) => {
                let mut written = w.write_str(cmd)?;
                written += write_args_with_trailing(w, args.iter().map(String::as_str))?;
                Ok(written)
            }
        }
    }
}

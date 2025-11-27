use std::fmt::{self, Write};

use super::types::Command;

/// Write a command with arguments directly to a formatter.
/// The last argument is treated as trailing and gets a `:` prefix if needed.
fn write_cmd(f: &mut fmt::Formatter<'_>, cmd: &str, args: &[&str]) -> fmt::Result {
    if args.is_empty() {
        return f.write_str(cmd);
    }

    let (middle_params, trailing) = args.split_at(args.len() - 1);
    let trailing = trailing[0];

    f.write_str(cmd)?;

    for param in middle_params {
        f.write_char(' ')?;
        f.write_str(param)?;
    }

    f.write_char(' ')?;

    // Add colon prefix if trailing is empty, contains a space, or starts with ':'
    if trailing.is_empty() || trailing.contains(' ') || trailing.starts_with(':') {
        f.write_char(':')?;
    }

    f.write_str(trailing)
}

/// Write a command with a freeform (always colon-prefixed) trailing argument.
fn write_cmd_freeform(f: &mut fmt::Formatter<'_>, cmd: &str, args: &[&str]) -> fmt::Result {
    match args.split_last() {
        Some((suffix, middle)) => {
            f.write_str(cmd)?;
            for arg in middle {
                f.write_char(' ')?;
                f.write_str(arg)?;
            }
            f.write_str(" :")?;
            f.write_str(suffix)
        }
        None => f.write_str(cmd),
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::PASS(p) => write_cmd(f, "PASS", &[p]),
            Command::NICK(n) => write_cmd(f, "NICK", &[n]),
            Command::USER(u, m, r) => write_cmd_freeform(f, "USER", &[u, m, "*", r]),
            Command::OPER(u, p) => write_cmd(f, "OPER", &[u, p]),
            Command::UserMODE(u, modes) => {
                f.write_str("MODE ")?;
                f.write_str(u)?;
                f.write_char(' ')?;
                for m in modes {
                    write!(f, "{}", m.flag())?;
                }
                Ok(())
            }
            Command::SERVICE(nick, r0, dist, typ, r1, info) => {
                write_cmd_freeform(f, "SERVICE", &[nick, r0, dist, typ, r1, info])
            }
            Command::QUIT(Some(m)) => write_cmd(f, "QUIT", &[m]),
            Command::QUIT(None) => write_cmd(f, "QUIT", &[]),
            Command::SQUIT(s, c) => write_cmd_freeform(f, "SQUIT", &[s, c]),
            Command::JOIN(c, Some(k), Some(n)) => write_cmd(f, "JOIN", &[c, k, n]),
            Command::JOIN(c, Some(k), None) => write_cmd(f, "JOIN", &[c, k]),
            Command::JOIN(c, None, Some(n)) => write_cmd(f, "JOIN", &[c, n]),
            Command::JOIN(c, None, None) => write_cmd(f, "JOIN", &[c]),
            Command::PART(c, Some(m)) => write_cmd_freeform(f, "PART", &[c, m]),
            Command::PART(c, None) => write_cmd(f, "PART", &[c]),
            Command::ChannelMODE(c, modes) => {
                f.write_str("MODE ")?;
                f.write_str(c)?;
                f.write_char(' ')?;
                for m in modes {
                    write!(f, "{}", m.flag())?;
                }
                for m in modes {
                    if let Some(arg) = m.arg() {
                        f.write_char(' ')?;
                        f.write_str(arg)?;
                    }
                }
                Ok(())
            }
            Command::TOPIC(c, Some(t)) => write_cmd_freeform(f, "TOPIC", &[c, t]),
            Command::TOPIC(c, None) => write_cmd(f, "TOPIC", &[c]),
            Command::NAMES(Some(c), Some(t)) => write_cmd(f, "NAMES", &[c, t]),
            Command::NAMES(Some(c), None) => write_cmd(f, "NAMES", &[c]),
            Command::NAMES(None, _) => write_cmd(f, "NAMES", &[]),
            Command::LIST(Some(c), Some(t)) => write_cmd(f, "LIST", &[c, t]),
            Command::LIST(Some(c), None) => write_cmd(f, "LIST", &[c]),
            Command::LIST(None, _) => write_cmd(f, "LIST", &[]),
            Command::INVITE(n, c) => write_cmd_freeform(f, "INVITE", &[n, c]),
            Command::KICK(c, n, Some(r)) => write_cmd_freeform(f, "KICK", &[c, n, r]),
            Command::KICK(c, n, None) => write_cmd(f, "KICK", &[c, n]),
            Command::PRIVMSG(t, m) => write_cmd_freeform(f, "PRIVMSG", &[t, m]),
            Command::NOTICE(t, m) => write_cmd_freeform(f, "NOTICE", &[t, m]),
            Command::MOTD(Some(t)) => write_cmd(f, "MOTD", &[t]),
            Command::MOTD(None) => write_cmd(f, "MOTD", &[]),
            Command::LUSERS(Some(m), Some(t)) => write_cmd(f, "LUSERS", &[m, t]),
            Command::LUSERS(Some(m), None) => write_cmd(f, "LUSERS", &[m]),
            Command::LUSERS(None, _) => write_cmd(f, "LUSERS", &[]),
            Command::VERSION(Some(t)) => write_cmd(f, "VERSION", &[t]),
            Command::VERSION(None) => write_cmd(f, "VERSION", &[]),
            Command::STATS(Some(q), Some(t)) => write_cmd(f, "STATS", &[q, t]),
            Command::STATS(Some(q), None) => write_cmd(f, "STATS", &[q]),
            Command::STATS(None, _) => write_cmd(f, "STATS", &[]),
            Command::LINKS(Some(r), Some(s)) => write_cmd(f, "LINKS", &[r, s]),
            Command::LINKS(None, Some(s)) => write_cmd(f, "LINKS", &[s]),
            Command::LINKS(_, None) => write_cmd(f, "LINKS", &[]),
            Command::TIME(Some(t)) => write_cmd(f, "TIME", &[t]),
            Command::TIME(None) => write_cmd(f, "TIME", &[]),
            Command::CONNECT(t, p, Some(r)) => write_cmd(f, "CONNECT", &[t, p, r]),
            Command::CONNECT(t, p, None) => write_cmd(f, "CONNECT", &[t, p]),
            Command::TRACE(Some(t)) => write_cmd(f, "TRACE", &[t]),
            Command::TRACE(None) => write_cmd(f, "TRACE", &[]),
            Command::ADMIN(Some(t)) => write_cmd(f, "ADMIN", &[t]),
            Command::ADMIN(None) => write_cmd(f, "ADMIN", &[]),
            Command::INFO(Some(t)) => write_cmd(f, "INFO", &[t]),
            Command::INFO(None) => write_cmd(f, "INFO", &[]),
            Command::SERVLIST(Some(m), Some(t)) => write_cmd(f, "SERVLIST", &[m, t]),
            Command::SERVLIST(Some(m), None) => write_cmd(f, "SERVLIST", &[m]),
            Command::SERVLIST(None, _) => write_cmd(f, "SERVLIST", &[]),
            Command::SQUERY(s, t) => write_cmd_freeform(f, "SQUERY", &[s, t]),
            Command::WHO(Some(s), Some(true)) => write_cmd(f, "WHO", &[s, "o"]),
            Command::WHO(Some(s), _) => write_cmd(f, "WHO", &[s]),
            Command::WHO(None, _) => write_cmd(f, "WHO", &[]),
            Command::WHOIS(Some(t), m) => write_cmd(f, "WHOIS", &[t, m]),
            Command::WHOIS(None, m) => write_cmd(f, "WHOIS", &[m]),
            Command::WHOWAS(n, Some(c), Some(t)) => write_cmd(f, "WHOWAS", &[n, c, t]),
            Command::WHOWAS(n, Some(c), None) => write_cmd(f, "WHOWAS", &[n, c]),
            Command::WHOWAS(n, None, _) => write_cmd(f, "WHOWAS", &[n]),
            Command::KILL(n, c) => write_cmd_freeform(f, "KILL", &[n, c]),
            Command::PING(s, Some(t)) => write_cmd(f, "PING", &[s, t]),
            Command::PING(s, None) => write_cmd(f, "PING", &[s]),
            Command::PONG(s, Some(t)) => write_cmd(f, "PONG", &[s, t]),
            Command::PONG(s, None) => write_cmd(f, "PONG", &[s]),
            Command::ERROR(m) => write_cmd_freeform(f, "ERROR", &[m]),
            Command::AWAY(Some(m)) => write_cmd_freeform(f, "AWAY", &[m]),
            Command::AWAY(None) => write_cmd(f, "AWAY", &[]),
            Command::REHASH => write_cmd(f, "REHASH", &[]),
            Command::DIE => write_cmd(f, "DIE", &[]),
            Command::RESTART => write_cmd(f, "RESTART", &[]),
            Command::SUMMON(u, Some(t), Some(c)) => write_cmd(f, "SUMMON", &[u, t, c]),
            Command::SUMMON(u, Some(t), None) => write_cmd(f, "SUMMON", &[u, t]),
            Command::SUMMON(u, None, _) => write_cmd(f, "SUMMON", &[u]),
            Command::USERS(Some(t)) => write_cmd(f, "USERS", &[t]),
            Command::USERS(None) => write_cmd(f, "USERS", &[]),
            Command::WALLOPS(t) => write_cmd_freeform(f, "WALLOPS", &[t]),
            Command::USERHOST(u) => {
                write_cmd(f, "USERHOST", &u.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            }
            Command::ISON(u) => {
                write_cmd(f, "ISON", &u.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            }
            Command::SAJOIN(n, c) => write_cmd(f, "SAJOIN", &[n, c]),
            Command::SAMODE(t, m, Some(p)) => write_cmd(f, "SAMODE", &[t, m, p]),
            Command::SAMODE(t, m, None) => write_cmd(f, "SAMODE", &[t, m]),
            Command::SANICK(o, n) => write_cmd(f, "SANICK", &[o, n]),
            Command::SAPART(c, r) => write_cmd(f, "SAPART", &[c, r]),
            Command::SAQUIT(c, r) => write_cmd(f, "SAQUIT", &[c, r]),
            Command::NICKSERV(p) => {
                write_cmd(f, "NICKSERV", &p.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            }
            Command::CHANSERV(m) => write_cmd(f, "CHANSERV", &[m]),
            Command::OPERSERV(m) => write_cmd(f, "OPERSERV", &[m]),
            Command::BOTSERV(m) => write_cmd(f, "BOTSERV", &[m]),
            Command::HOSTSERV(m) => write_cmd(f, "HOSTSERV", &[m]),
            Command::MEMOSERV(m) => write_cmd(f, "MEMOSERV", &[m]),
            Command::CAP(None, s, None, Some(p)) => write_cmd(f, "CAP", &[s.to_str(), p]),
            Command::CAP(None, s, None, None) => write_cmd(f, "CAP", &[s.to_str()]),
            Command::CAP(Some(k), s, None, Some(p)) => write_cmd(f, "CAP", &[k, s.to_str(), p]),
            Command::CAP(Some(k), s, None, None) => write_cmd(f, "CAP", &[k, s.to_str()]),
            Command::CAP(None, s, Some(c), Some(p)) => write_cmd(f, "CAP", &[s.to_str(), c, p]),
            Command::CAP(None, s, Some(c), None) => write_cmd(f, "CAP", &[s.to_str(), c]),
            Command::CAP(Some(k), s, Some(c), Some(p)) => {
                write_cmd(f, "CAP", &[k, s.to_str(), c, p])
            }
            Command::CAP(Some(k), s, Some(c), None) => write_cmd(f, "CAP", &[k, s.to_str(), c]),
            Command::AUTHENTICATE(d) => write_cmd(f, "AUTHENTICATE", &[d]),
            Command::ACCOUNT(a) => write_cmd(f, "ACCOUNT", &[a]),
            Command::MONITOR(c, Some(t)) => write_cmd(f, "MONITOR", &[c, t]),
            Command::MONITOR(c, None) => write_cmd(f, "MONITOR", &[c]),
            Command::BATCH(t, Some(c), Some(a)) => {
                let mut args: Vec<&str> = vec![t, c.to_str()];
                args.extend(a.iter().map(|s| s.as_str()));
                write_cmd(f, "BATCH", &args)
            }
            Command::BATCH(t, Some(c), None) => write_cmd(f, "BATCH", &[t, c.to_str()]),
            Command::BATCH(t, None, Some(a)) => {
                let mut args: Vec<&str> = vec![t];
                args.extend(a.iter().map(|s| s.as_str()));
                write_cmd(f, "BATCH", &args)
            }
            Command::BATCH(t, None, None) => write_cmd(f, "BATCH", &[t]),
            Command::CHGHOST(u, h) => write_cmd(f, "CHGHOST", &[u, h]),
            Command::SETNAME(r) => write_cmd_freeform(f, "SETNAME", &[r]),
            Command::FAIL(command, code, context) => {
                let mut args: Vec<&str> = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                write_cmd_freeform(f, "FAIL", &args)
            }
            Command::WARN(command, code, context) => {
                let mut args: Vec<&str> = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                write_cmd_freeform(f, "WARN", &args)
            }
            Command::NOTE(command, code, context) => {
                let mut args: Vec<&str> = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                write_cmd_freeform(f, "NOTE", &args)
            }
            Command::Response(resp, a) => {
                write!(f, "{:03}", *resp as u16)?;
                for arg in a.iter().take(a.len().saturating_sub(1)) {
                    f.write_char(' ')?;
                    f.write_str(arg)?;
                }
                if let Some(last) = a.last() {
                    f.write_char(' ')?;
                    if last.is_empty() || last.contains(' ') || last.starts_with(':') {
                        f.write_char(':')?;
                    }
                    f.write_str(last)?;
                }
                Ok(())
            }
            Command::Raw(c, a) => {
                write_cmd(f, c, &a.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            }
        }
    }
}

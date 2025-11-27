
use super::types::Command;

fn stringify(cmd: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return cmd.to_string();
    }


    let (middle_params, trailing) = args.split_at(args.len() - 1);
    let trailing = trailing[0];


    let mut result = String::with_capacity(512);
    result.push_str(cmd);

    for param in middle_params {
        result.push(' ');
        result.push_str(param);
    }


    result.push(' ');







    if trailing.is_empty() || trailing.contains(' ') || trailing.starts_with(':') {
        result.push(':');
    }

    result.push_str(trailing);
    result
}

fn stringify_freeform(cmd: &str, args: &[&str]) -> String {
    match args.split_last() {
        Some((suffix, args)) => {
            let args = args.join(" ");
            let sp = if args.is_empty() { "" } else { " " };

            format!("{}{}{} :{}", cmd, sp, args, suffix)
        }
        None => cmd.to_string(),
    }
}

impl<'a> From<&'a Command> for String {
    fn from(cmd: &'a Command) -> String {
        
        
        
        
        
        
        
        match *cmd {
            Command::PASS(ref p) => stringify("PASS", &[p]),
            Command::NICK(ref n) => stringify("NICK", &[n]),
            Command::USER(ref u, ref m, ref r) => stringify_freeform("USER", &[u, m, "*", r]),
            Command::OPER(ref u, ref p) => stringify("OPER", &[u, p]),
            Command::UserMODE(ref u, ref m) => {
                
                m.iter().fold(format!("MODE {u} "), |mut acc, m| {
                    acc.push_str(&m.flag());
                    acc
                })
            }
            Command::SERVICE(ref nick, ref r0, ref dist, ref typ, ref r1, ref info) => {
                stringify_freeform("SERVICE", &[nick, r0, dist, typ, r1, info])
            }
            Command::QUIT(Some(ref m)) => stringify("QUIT", &[m]),
            Command::QUIT(None) => stringify("QUIT", &[]),
            Command::SQUIT(ref s, ref c) => stringify_freeform("SQUIT", &[s, c]),
            Command::JOIN(ref c, Some(ref k), Some(ref n)) => stringify("JOIN", &[c, k, n]),
            Command::JOIN(ref c, Some(ref k), None) => stringify("JOIN", &[c, k]),
            Command::JOIN(ref c, None, Some(ref n)) => stringify("JOIN", &[c, n]),
            Command::JOIN(ref c, None, None) => stringify("JOIN", &[c]),
            Command::PART(ref c, Some(ref m)) => stringify_freeform("PART", &[c, m]),
            Command::PART(ref c, None) => stringify("PART", &[c]),
            Command::ChannelMODE(ref c, ref m) => {
                let cmd = m.iter().fold(format!("MODE {c} "), |mut acc, m| {
                    acc.push_str(&m.flag());
                    acc
                });
                m.iter().filter_map(|m| m.arg()).fold(cmd, |mut acc, arg| {
                    acc.push(' ');
                    acc.push_str(arg);
                    acc
                })
            }
            Command::TOPIC(ref c, Some(ref t)) => stringify_freeform("TOPIC", &[c, t]),
            Command::TOPIC(ref c, None) => stringify("TOPIC", &[c]),
            Command::NAMES(Some(ref c), Some(ref t)) => stringify("NAMES", &[c, t]),
            Command::NAMES(Some(ref c), None) => stringify("NAMES", &[c]),
            Command::NAMES(None, _) => stringify("NAMES", &[]),
            Command::LIST(Some(ref c), Some(ref t)) => stringify("LIST", &[c, t]),
            Command::LIST(Some(ref c), None) => stringify("LIST", &[c]),
            Command::LIST(None, _) => stringify("LIST", &[]),
            Command::INVITE(ref n, ref c) => stringify_freeform("INVITE", &[n, c]),
            Command::KICK(ref c, ref n, Some(ref r)) => stringify_freeform("KICK", &[c, n, r]),
            Command::KICK(ref c, ref n, None) => stringify("KICK", &[c, n]),
            Command::PRIVMSG(ref t, ref m) => stringify_freeform("PRIVMSG", &[t, m]),
            Command::NOTICE(ref t, ref m) => stringify_freeform("NOTICE", &[t, m]),
            Command::MOTD(Some(ref t)) => stringify("MOTD", &[t]),
            Command::MOTD(None) => stringify("MOTD", &[]),
            Command::LUSERS(Some(ref m), Some(ref t)) => stringify("LUSERS", &[m, t]),
            Command::LUSERS(Some(ref m), None) => stringify("LUSERS", &[m]),
            Command::LUSERS(None, _) => stringify("LUSERS", &[]),
            Command::VERSION(Some(ref t)) => stringify("VERSION", &[t]),
            Command::VERSION(None) => stringify("VERSION", &[]),
            Command::STATS(Some(ref q), Some(ref t)) => stringify("STATS", &[q, t]),
            Command::STATS(Some(ref q), None) => stringify("STATS", &[q]),
            Command::STATS(None, _) => stringify("STATS", &[]),
            Command::LINKS(Some(ref r), Some(ref s)) => stringify("LINKS", &[r, s]),
            Command::LINKS(None, Some(ref s)) => stringify("LINKS", &[s]),
            Command::LINKS(_, None) => stringify("LINKS", &[]),
            Command::TIME(Some(ref t)) => stringify("TIME", &[t]),
            Command::TIME(None) => stringify("TIME", &[]),
            Command::CONNECT(ref t, ref p, Some(ref r)) => stringify("CONNECT", &[t, p, r]),
            Command::CONNECT(ref t, ref p, None) => stringify("CONNECT", &[t, p]),
            Command::TRACE(Some(ref t)) => stringify("TRACE", &[t]),
            Command::TRACE(None) => stringify("TRACE", &[]),
            Command::ADMIN(Some(ref t)) => stringify("ADMIN", &[t]),
            Command::ADMIN(None) => stringify("ADMIN", &[]),
            Command::INFO(Some(ref t)) => stringify("INFO", &[t]),
            Command::INFO(None) => stringify("INFO", &[]),
            Command::SERVLIST(Some(ref m), Some(ref t)) => stringify("SERVLIST", &[m, t]),
            Command::SERVLIST(Some(ref m), None) => stringify("SERVLIST", &[m]),
            Command::SERVLIST(None, _) => stringify("SERVLIST", &[]),
            Command::SQUERY(ref s, ref t) => stringify_freeform("SQUERY", &[s, t]),
            Command::WHO(Some(ref s), Some(true)) => stringify("WHO", &[s, "o"]),
            Command::WHO(Some(ref s), _) => stringify("WHO", &[s]),
            Command::WHO(None, _) => stringify("WHO", &[]),
            Command::WHOIS(Some(ref t), ref m) => stringify("WHOIS", &[t, m]),
            Command::WHOIS(None, ref m) => stringify("WHOIS", &[m]),
            Command::WHOWAS(ref n, Some(ref c), Some(ref t)) => stringify("WHOWAS", &[n, c, t]),
            Command::WHOWAS(ref n, Some(ref c), None) => stringify("WHOWAS", &[n, c]),
            Command::WHOWAS(ref n, None, _) => stringify("WHOWAS", &[n]),
            Command::KILL(ref n, ref c) => stringify_freeform("KILL", &[n, c]),
            Command::PING(ref s, Some(ref t)) => stringify("PING", &[s, t]),
            Command::PING(ref s, None) => stringify("PING", &[s]),
            Command::PONG(ref s, Some(ref t)) => stringify("PONG", &[s, t]),
            Command::PONG(ref s, None) => stringify("PONG", &[s]),
            Command::ERROR(ref m) => stringify_freeform("ERROR", &[m]),
            Command::AWAY(Some(ref m)) => stringify_freeform("AWAY", &[m]),
            Command::AWAY(None) => stringify("AWAY", &[]),
            Command::REHASH => stringify("REHASH", &[]),
            Command::DIE => stringify("DIE", &[]),
            Command::RESTART => stringify("RESTART", &[]),
            Command::SUMMON(ref u, Some(ref t), Some(ref c)) => stringify("SUMMON", &[u, t, c]),
            Command::SUMMON(ref u, Some(ref t), None) => stringify("SUMMON", &[u, t]),
            Command::SUMMON(ref u, None, _) => stringify("SUMMON", &[u]),
            Command::USERS(Some(ref t)) => stringify("USERS", &[t]),
            Command::USERS(None) => stringify("USERS", &[]),
            Command::WALLOPS(ref t) => stringify_freeform("WALLOPS", &[t]),
            Command::USERHOST(ref u) => {
                stringify("USERHOST", &u.iter().map(|s| &s[..]).collect::<Vec<_>>())
            }
            Command::ISON(ref u) => {
                stringify("ISON", &u.iter().map(|s| &s[..]).collect::<Vec<_>>())
            }

            Command::SAJOIN(ref n, ref c) => stringify("SAJOIN", &[n, c]),
            Command::SAMODE(ref t, ref m, Some(ref p)) => stringify("SAMODE", &[t, m, p]),
            Command::SAMODE(ref t, ref m, None) => stringify("SAMODE", &[t, m]),
            Command::SANICK(ref o, ref n) => stringify("SANICK", &[o, n]),
            Command::SAPART(ref c, ref r) => stringify("SAPART", &[c, r]),
            Command::SAQUIT(ref c, ref r) => stringify("SAQUIT", &[c, r]),

            Command::NICKSERV(ref p) => {
                stringify("NICKSERV", &p.iter().map(|s| &s[..]).collect::<Vec<_>>())
            }
            Command::CHANSERV(ref m) => stringify("CHANSERV", &[m]),
            Command::OPERSERV(ref m) => stringify("OPERSERV", &[m]),
            Command::BOTSERV(ref m) => stringify("BOTSERV", &[m]),
            Command::HOSTSERV(ref m) => stringify("HOSTSERV", &[m]),
            Command::MEMOSERV(ref m) => stringify("MEMOSERV", &[m]),

            Command::CAP(None, ref s, None, Some(ref p)) => stringify("CAP", &[s.to_str(), p]),
            Command::CAP(None, ref s, None, None) => stringify("CAP", &[s.to_str()]),
            Command::CAP(Some(ref k), ref s, None, Some(ref p)) => {
                stringify("CAP", &[k, s.to_str(), p])
            }
            Command::CAP(Some(ref k), ref s, None, None) => stringify("CAP", &[k, s.to_str()]),
            Command::CAP(None, ref s, Some(ref c), Some(ref p)) => {
                stringify("CAP", &[s.to_str(), c, p])
            }
            Command::CAP(None, ref s, Some(ref c), None) => stringify("CAP", &[s.to_str(), c]),
            Command::CAP(Some(ref k), ref s, Some(ref c), Some(ref p)) => {
                stringify("CAP", &[k, s.to_str(), c, p])
            }
            Command::CAP(Some(ref k), ref s, Some(ref c), None) => {
                stringify("CAP", &[k, s.to_str(), c])
            }

            Command::AUTHENTICATE(ref d) => stringify("AUTHENTICATE", &[d]),
            Command::ACCOUNT(ref a) => stringify("ACCOUNT", &[a]),

            Command::MONITOR(ref c, Some(ref t)) => stringify("MONITOR", &[c, t]),
            Command::MONITOR(ref c, None) => stringify("MONITOR", &[c]),
            Command::BATCH(ref t, Some(ref c), Some(ref a)) => stringify(
                "BATCH",
                &[t, &c.to_str().to_owned()]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
            ),
            Command::BATCH(ref t, Some(ref c), None) => stringify("BATCH", &[t, c.to_str()]),
            Command::BATCH(ref t, None, Some(ref a)) => stringify(
                "BATCH",
                &[t].iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
            ),
            Command::BATCH(ref t, None, None) => stringify("BATCH", &[t]),
            Command::CHGHOST(ref u, ref h) => stringify("CHGHOST", &[u, h]),
            Command::SETNAME(ref r) => stringify_freeform("SETNAME", &[r]),


            Command::FAIL(ref command, ref code, ref context) => {
                let mut args = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                stringify_freeform("FAIL", &args)
            }
            Command::WARN(ref command, ref code, ref context) => {
                let mut args = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                stringify_freeform("WARN", &args)
            }
            Command::NOTE(ref command, ref code, ref context) => {
                let mut args = vec![command.as_str(), code.as_str()];
                args.extend(context.iter().map(|s| s.as_str()));
                stringify_freeform("NOTE", &args)
            }

            Command::Response(ref resp, ref a) => stringify(
                &format!("{:03}", *resp as u16),
                &a.iter().map(|s| &s[..]).collect::<Vec<_>>(),
            ),
            Command::Raw(ref c, ref a) => {
                stringify(c, &a.iter().map(|s| &s[..]).collect::<Vec<_>>())
            }
        }
    }
}

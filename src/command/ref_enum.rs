
use crate::mode::{ChannelMode, Mode, UserMode};
use crate::response::Response;

use crate::command::subcommands::{BatchSubCommand, CapSubCommand};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandRefEnum<'a> {
    
    PASS(&'a str),
    NICK(&'a str),
    USER(&'a str, &'a str, &'a str),
    OPER(&'a str, &'a str),
    UserMODE(&'a str, Vec<Mode<UserMode>>),
    SERVICE(&'a str, &'a str, &'a str, &'a str, &'a str, &'a str),
    QUIT(Option<&'a str>),
    SQUIT(&'a str, &'a str),


    JOIN(&'a str, Option<&'a str>, Option<&'a str>),
    PART(&'a str, Option<&'a str>),
    ChannelMODE(&'a str, Vec<Mode<ChannelMode>>),
    TOPIC(&'a str, Option<&'a str>),
    NAMES(Option<&'a str>, Option<&'a str>),
    LIST(Option<&'a str>, Option<&'a str>),
    INVITE(&'a str, &'a str),
    KICK(&'a str, &'a str, Option<&'a str>),

    
    PRIVMSG(&'a str, &'a str),
    NOTICE(&'a str, &'a str),

    
    MOTD(Option<&'a str>),
    LUSERS(Option<&'a str>, Option<&'a str>),
    VERSION(Option<&'a str>),
    STATS(Option<&'a str>, Option<&'a str>),
    LINKS(Option<&'a str>, Option<&'a str>),
    TIME(Option<&'a str>),
    CONNECT(&'a str, &'a str, Option<&'a str>),
    TRACE(Option<&'a str>),
    ADMIN(Option<&'a str>),
    INFO(Option<&'a str>),


    SERVLIST(Option<&'a str>, Option<&'a str>),
    SQUERY(&'a str, &'a str),


    WHO(Option<&'a str>, Option<bool>),
    WHOIS(Option<&'a str>, &'a str),
    WHOWAS(&'a str, Option<&'a str>, Option<&'a str>),


    KILL(&'a str, &'a str),
    PING(&'a str, Option<&'a str>),
    PONG(&'a str, Option<&'a str>),
    ERROR(&'a str),

    
    AWAY(Option<&'a str>),
    REHASH,
    DIE,
    RESTART,
    SUMMON(&'a str, Option<&'a str>, Option<&'a str>),
    USERS(Option<&'a str>),
    WALLOPS(&'a str),
    USERHOST(Vec<&'a str>),
    ISON(Vec<&'a str>),

    
    SAJOIN(&'a str, &'a str),
    SAMODE(&'a str, &'a str, Option<&'a str>),
    SANICK(&'a str, &'a str),
    SAPART(&'a str, &'a str),
    SAQUIT(&'a str, &'a str),
    NICKSERV(Vec<&'a str>),
    CHANSERV(&'a str),
    OPERSERV(&'a str),
    BOTSERV(&'a str),
    HOSTSERV(&'a str),
    MEMOSERV(&'a str),


    CAP(
        Option<&'a str>,
        CapSubCommand,
        Option<&'a str>,
        Option<&'a str>,
    ),

    
    AUTHENTICATE(&'a str),
    ACCOUNT(&'a str),

    
    MONITOR(&'a str, Option<&'a str>),
    BATCH(&'a str, Option<BatchSubCommand>, Option<Vec<&'a str>>),
    CHGHOST(&'a str, &'a str),
    SETNAME(&'a str),


    Response(Response, Vec<&'a str>),
    Raw(&'a str, Vec<&'a str>),
}

impl<'a> CommandRefEnum<'a> {
    pub fn to_owned(&self) -> crate::command::Command {
        use crate::command::Command;

        match self {
            Self::PASS(p) => Command::PASS(p.to_string()),
            Self::NICK(n) => Command::NICK(n.to_string()),
            Self::USER(u, m, r) => Command::USER(u.to_string(), m.to_string(), r.to_string()),
            Self::OPER(n, p) => Command::OPER(n.to_string(), p.to_string()),
            Self::UserMODE(n, modes) => Command::UserMODE(n.to_string(), modes.clone()),
            Self::SERVICE(n, r1, d, t, r2, i) => {
                Command::SERVICE(
                    n.to_string(),
                    r1.to_string(),
                    d.to_string(),
                    t.to_string(),
                    r2.to_string(),
                    i.to_string(),
                )
            }
            Self::QUIT(c) => Command::QUIT(c.map(|s| s.to_string())),
            Self::SQUIT(s, c) => Command::SQUIT(s.to_string(), c.to_string()),
            Self::JOIN(c, k, r) => {
                Command::JOIN(c.to_string(), k.map(|s| s.to_string()), r.map(|s| s.to_string()))
            }
            Self::PART(c, m) => Command::PART(c.to_string(), m.map(|s| s.to_string())),
            Self::ChannelMODE(c, modes) => Command::ChannelMODE(c.to_string(), modes.clone()),
            Self::TOPIC(c, t) => Command::TOPIC(c.to_string(), t.map(|s| s.to_string())),
            Self::NAMES(c, t) => {
                Command::NAMES(c.map(|s| s.to_string()), t.map(|s| s.to_string()))
            }
            Self::LIST(c, t) => Command::LIST(c.map(|s| s.to_string()), t.map(|s| s.to_string())),
            Self::INVITE(n, c) => Command::INVITE(n.to_string(), c.to_string()),
            Self::KICK(c, u, m) => {
                Command::KICK(c.to_string(), u.to_string(), m.map(|s| s.to_string()))
            }
            Self::PRIVMSG(t, m) => Command::PRIVMSG(t.to_string(), m.to_string()),
            Self::NOTICE(t, m) => Command::NOTICE(t.to_string(), m.to_string()),
            Self::MOTD(t) => Command::MOTD(t.map(|s| s.to_string())),
            Self::LUSERS(m, t) => {
                Command::LUSERS(m.map(|s| s.to_string()), t.map(|s| s.to_string()))
            }
            Self::VERSION(t) => Command::VERSION(t.map(|s| s.to_string())),
            Self::STATS(q, t) => {
                Command::STATS(q.map(|s| s.to_string()), t.map(|s| s.to_string()))
            }
            Self::LINKS(r, s) => {
                Command::LINKS(r.map(|s| s.to_string()), s.map(|s| s.to_string()))
            }
            Self::TIME(t) => Command::TIME(t.map(|s| s.to_string())),
            Self::CONNECT(t, s, r) => {
                Command::CONNECT(t.to_string(), s.to_string(), r.map(|s| s.to_string()))
            }
            Self::TRACE(t) => Command::TRACE(t.map(|s| s.to_string())),
            Self::ADMIN(t) => Command::ADMIN(t.map(|s| s.to_string())),
            Self::INFO(t) => Command::INFO(t.map(|s| s.to_string())),
            Self::SERVLIST(m, t) => {
                Command::SERVLIST(m.map(|s| s.to_string()), t.map(|s| s.to_string()))
            }
            Self::SQUERY(s, t) => Command::SQUERY(s.to_string(), t.to_string()),
            Self::WHO(m, o) => Command::WHO(m.map(|s| s.to_string()), *o),
            Self::WHOIS(t, m) => Command::WHOIS(t.map(|s| s.to_string()), m.to_string()),
            Self::WHOWAS(n, c, t) => {
                Command::WHOWAS(n.to_string(), c.map(|s| s.to_string()), t.map(|s| s.to_string()))
            }
            Self::KILL(n, c) => Command::KILL(n.to_string(), c.to_string()),
            Self::PING(s1, s2) => Command::PING(s1.to_string(), s2.map(|s| s.to_string())),
            Self::PONG(s1, s2) => Command::PONG(s1.to_string(), s2.map(|s| s.to_string())),
            Self::ERROR(m) => Command::ERROR(m.to_string()),
            Self::AWAY(m) => Command::AWAY(m.map(|s| s.to_string())),
            Self::REHASH => Command::REHASH,
            Self::DIE => Command::DIE,
            Self::RESTART => Command::RESTART,
            Self::SUMMON(u, t, c) => {
                Command::SUMMON(u.to_string(), t.map(|s| s.to_string()), c.map(|s| s.to_string()))
            }
            Self::USERS(t) => Command::USERS(t.map(|s| s.to_string())),
            Self::WALLOPS(t) => Command::WALLOPS(t.to_string()),
            Self::USERHOST(list) => Command::USERHOST(list.iter().map(|s| s.to_string()).collect()),
            Self::ISON(list) => Command::ISON(list.iter().map(|s| s.to_string()).collect()),
            Self::SAJOIN(n, c) => Command::SAJOIN(n.to_string(), c.to_string()),
            Self::SAMODE(t, m, p) => {
                Command::SAMODE(t.to_string(), m.to_string(), p.map(|s| s.to_string()))
            }
            Self::SANICK(o, n) => Command::SANICK(o.to_string(), n.to_string()),
            Self::SAPART(n, c) => Command::SAPART(n.to_string(), c.to_string()),
            Self::SAQUIT(n, c) => Command::SAQUIT(n.to_string(), c.to_string()),
            Self::NICKSERV(args) => Command::NICKSERV(args.iter().map(|s| s.to_string()).collect()),
            Self::CHANSERV(m) => Command::CHANSERV(m.to_string()),
            Self::OPERSERV(m) => Command::OPERSERV(m.to_string()),
            Self::BOTSERV(m) => Command::BOTSERV(m.to_string()),
            Self::HOSTSERV(m) => Command::HOSTSERV(m.to_string()),
            Self::MEMOSERV(m) => Command::MEMOSERV(m.to_string()),
            Self::CAP(a, sc, b, c) => {
                Command::CAP(
                    a.map(|s| s.to_string()),
                    *sc,
                    b.map(|s| s.to_string()),
                    c.map(|s| s.to_string()),
                )
            }
            Self::AUTHENTICATE(d) => Command::AUTHENTICATE(d.to_string()),
            Self::ACCOUNT(a) => Command::ACCOUNT(a.to_string()),
            Self::MONITOR(c, n) => Command::MONITOR(c.to_string(), n.map(|s| s.to_string())),
            Self::BATCH(r, sc, params) => Command::BATCH(
                r.to_string(),
                sc.clone(),
                params.as_ref().map(|v| v.iter().map(|s| s.to_string()).collect()),
            ),
            Self::CHGHOST(u, h) => Command::CHGHOST(u.to_string(), h.to_string()),
            Self::SETNAME(r) => Command::SETNAME(r.to_string()),
            Self::Response(r, args) => {
                Command::Response(*r, args.iter().map(|s| s.to_string()).collect())
            }
            Self::Raw(cmd, args) => {
                Command::Raw(cmd.to_string(), args.iter().map(|s| s.to_string()).collect())
            }
        }
    }

    pub fn command_name(&self) -> &str {
        match self {
            Self::PASS(_) => "PASS",
            Self::NICK(_) => "NICK",
            Self::USER(_, _, _) => "USER",
            Self::OPER(_, _) => "OPER",
            Self::UserMODE(_, _) => "MODE",
            Self::SERVICE(_, _, _, _, _, _) => "SERVICE",
            Self::QUIT(_) => "QUIT",
            Self::SQUIT(_, _) => "SQUIT",
            Self::JOIN(_, _, _) => "JOIN",
            Self::PART(_, _) => "PART",
            Self::ChannelMODE(_, _) => "MODE",
            Self::TOPIC(_, _) => "TOPIC",
            Self::NAMES(_, _) => "NAMES",
            Self::LIST(_, _) => "LIST",
            Self::INVITE(_, _) => "INVITE",
            Self::KICK(_, _, _) => "KICK",
            Self::PRIVMSG(_, _) => "PRIVMSG",
            Self::NOTICE(_, _) => "NOTICE",
            Self::MOTD(_) => "MOTD",
            Self::LUSERS(_, _) => "LUSERS",
            Self::VERSION(_) => "VERSION",
            Self::STATS(_, _) => "STATS",
            Self::LINKS(_, _) => "LINKS",
            Self::TIME(_) => "TIME",
            Self::CONNECT(_, _, _) => "CONNECT",
            Self::TRACE(_) => "TRACE",
            Self::ADMIN(_) => "ADMIN",
            Self::INFO(_) => "INFO",
            Self::SERVLIST(_, _) => "SERVLIST",
            Self::SQUERY(_, _) => "SQUERY",
            Self::WHO(_, _) => "WHO",
            Self::WHOIS(_, _) => "WHOIS",
            Self::WHOWAS(_, _, _) => "WHOWAS",
            Self::KILL(_, _) => "KILL",
            Self::PING(_, _) => "PING",
            Self::PONG(_, _) => "PONG",
            Self::ERROR(_) => "ERROR",
            Self::AWAY(_) => "AWAY",
            Self::REHASH => "REHASH",
            Self::DIE => "DIE",
            Self::RESTART => "RESTART",
            Self::SUMMON(_, _, _) => "SUMMON",
            Self::USERS(_) => "USERS",
            Self::WALLOPS(_) => "WALLOPS",
            Self::USERHOST(_) => "USERHOST",
            Self::ISON(_) => "ISON",
            Self::SAJOIN(_, _) => "SAJOIN",
            Self::SAMODE(_, _, _) => "SAMODE",
            Self::SANICK(_, _) => "SANICK",
            Self::SAPART(_, _) => "SAPART",
            Self::SAQUIT(_, _) => "SAQUIT",
            Self::NICKSERV(_) => "NICKSERV",
            Self::CHANSERV(_) => "CHANSERV",
            Self::OPERSERV(_) => "OPERSERV",
            Self::BOTSERV(_) => "BOTSERV",
            Self::HOSTSERV(_) => "HOSTSERV",
            Self::MEMOSERV(_) => "MEMOSERV",
            Self::CAP(_, _, _, _) => "CAP",
            Self::AUTHENTICATE(_) => "AUTHENTICATE",
            Self::ACCOUNT(_) => "ACCOUNT",
            Self::MONITOR(_, _) => "MONITOR",
            Self::BATCH(_, _, _) => "BATCH",
            Self::CHGHOST(_, _) => "CHGHOST",
            Self::SETNAME(_) => "SETNAME",
            Self::Response(_, _) => "Response",  
            Self::Raw(cmd, _) => cmd,
        }
    }
}

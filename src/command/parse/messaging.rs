use super::super::types::Command;
use super::connection::raw;
use crate::error::MessageParseError;

pub(super) fn parse(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
    let result = match cmd {
        "PRIVMSG" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::PRIVMSG(args[0].to_owned(), args[1].to_owned())
            }
        }
        "NOTICE" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::NOTICE(args[0].to_owned(), args[1].to_owned())
            }
        }

        "KILL" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::KILL(args[0].to_owned(), args[1].to_owned())
            }
        }
        "PING" => {
            if args.len() == 1 {
                Command::PING(args[0].to_owned(), None)
            } else if args.len() == 2 {
                Command::PING(args[0].to_owned(), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "PONG" => {
            if args.len() == 1 {
                Command::PONG(args[0].to_owned(), None)
            } else if args.len() == 2 {
                Command::PONG(args[0].to_owned(), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "ERROR" => {
            if args.len() != 1 {
                raw(cmd, args)
            } else {
                Command::ERROR(args[0].to_owned())
            }
        }

        "AWAY" => {
            if args.is_empty() {
                Command::AWAY(None)
            } else if args.len() == 1 {
                Command::AWAY(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "REHASH" => {
            if args.is_empty() {
                Command::REHASH
            } else {
                raw(cmd, args)
            }
        }
        "DIE" => {
            if args.is_empty() {
                Command::DIE
            } else {
                raw(cmd, args)
            }
        }
        "RESTART" => {
            if args.is_empty() {
                Command::RESTART
            } else {
                raw(cmd, args)
            }
        }
        "SUMMON" => {
            if args.len() == 1 {
                Command::SUMMON(args[0].to_owned(), None, None)
            } else if args.len() == 2 {
                Command::SUMMON(args[0].to_owned(), Some(args[1].to_owned()), None)
            } else if args.len() == 3 {
                Command::SUMMON(
                    args[0].to_owned(),
                    Some(args[1].to_owned()),
                    Some(args[2].to_owned()),
                )
            } else {
                raw(cmd, args)
            }
        }
        "USERS" => {
            if args.len() != 1 {
                raw(cmd, args)
            } else {
                Command::USERS(Some(args[0].to_owned()))
            }
        }
        "WALLOPS" => {
            if args.len() != 1 {
                raw(cmd, args)
            } else {
                Command::WALLOPS(args[0].to_owned())
            }
        }
        "USERHOST" => Command::USERHOST(args.into_iter().map(|s| s.to_owned()).collect()),
        "ISON" => Command::ISON(args.into_iter().map(|s| s.to_owned()).collect()),

        "SAJOIN" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SAJOIN(args[0].to_owned(), args[1].to_owned())
            }
        }
        "SAMODE" => {
            if args.len() == 2 {
                Command::SAMODE(args[0].to_owned(), args[1].to_owned(), None)
            } else if args.len() == 3 {
                Command::SAMODE(
                    args[0].to_owned(),
                    args[1].to_owned(),
                    Some(args[2].to_owned()),
                )
            } else {
                raw(cmd, args)
            }
        }
        "SANICK" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SANICK(args[0].to_owned(), args[1].to_owned())
            }
        }
        "SAPART" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SAPART(args[0].to_owned(), args[1].to_owned())
            }
        }
        "SAQUIT" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SAQUIT(args[0].to_owned(), args[1].to_owned())
            }
        }

        "NICKSERV" => Command::NICKSERV(args.into_iter().map(|s| s.to_owned()).collect()),
        "CHANSERV" => Command::CHANSERV(args.into_iter().map(|s| s.to_owned()).collect()),
        "OPERSERV" => Command::OPERSERV(args.into_iter().map(|s| s.to_owned()).collect()),
        "BOTSERV" => Command::BOTSERV(args.into_iter().map(|s| s.to_owned()).collect()),
        "HOSTSERV" => Command::HOSTSERV(args.into_iter().map(|s| s.to_owned()).collect()),
        "MEMOSERV" => Command::MEMOSERV(args.into_iter().map(|s| s.to_owned()).collect()),

        _ => unreachable!(
            "messaging::parse called with non-messaging command: {}",
            cmd
        ),
    };

    Ok(result)
}

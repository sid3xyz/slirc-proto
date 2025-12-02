use super::super::types::Command;
use super::connection::raw;
use crate::error::MessageParseError;

pub(super) fn parse(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
    let result = match cmd {
        "MOTD" => {
            if args.is_empty() {
                Command::MOTD(None)
            } else if args.len() == 1 {
                Command::MOTD(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "LUSERS" => {
            if args.is_empty() {
                Command::LUSERS(None, None)
            } else if args.len() == 1 {
                Command::LUSERS(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::LUSERS(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "VERSION" => {
            if args.is_empty() {
                Command::VERSION(None)
            } else if args.len() == 1 {
                Command::VERSION(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "STATS" => {
            if args.is_empty() {
                Command::STATS(None, None)
            } else if args.len() == 1 {
                Command::STATS(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::STATS(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "LINKS" => {
            if args.is_empty() {
                Command::LINKS(None, None)
            } else if args.len() == 1 {
                Command::LINKS(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::LINKS(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "TIME" => {
            if args.is_empty() {
                Command::TIME(None)
            } else if args.len() == 1 {
                Command::TIME(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "CONNECT" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::CONNECT(args[0].to_owned(), args[1].to_owned(), None)
            }
        }
        "TRACE" => {
            if args.is_empty() {
                Command::TRACE(None)
            } else if args.len() == 1 {
                Command::TRACE(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "ADMIN" => {
            if args.is_empty() {
                Command::ADMIN(None)
            } else if args.len() == 1 {
                Command::ADMIN(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "INFO" => {
            if args.is_empty() {
                Command::INFO(None)
            } else if args.len() == 1 {
                Command::INFO(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "MAP" => {
            if args.is_empty() {
                Command::MAP
            } else {
                raw(cmd, args)
            }
        }
        "RULES" => {
            if args.is_empty() {
                Command::RULES
            } else {
                raw(cmd, args)
            }
        }
        "USERIP" => Command::USERIP(args.into_iter().map(|s| s.to_owned()).collect()),
        "HELP" => {
            if args.is_empty() {
                Command::HELP(None)
            } else {
                Command::HELP(Some(args[0].to_owned()))
            }
        }
        "SERVLIST" => {
            if args.is_empty() {
                Command::SERVLIST(None, None)
            } else if args.len() == 1 {
                Command::SERVLIST(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::SERVLIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "SQUERY" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SQUERY(args[0].to_owned(), args[1].to_owned())
            }
        }
        _ => unreachable!("server::parse called with non-server command: {}", cmd),
    };

    Ok(result)
}

use super::super::types::Command;
use super::connection::raw;
use crate::error::MessageParseError;

pub(super) fn parse(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
    let result = match cmd {
        "JOIN" => {
            if args.len() == 1 {
                Command::JOIN(args[0].to_owned(), None, None)
            } else if args.len() == 2 {
                Command::JOIN(args[0].to_owned(), Some(args[1].to_owned()), None)
            } else if args.len() == 3 {
                Command::JOIN(
                    args[0].to_owned(),
                    Some(args[1].to_owned()),
                    Some(args[2].to_owned()),
                )
            } else {
                raw(cmd, args)
            }
        }
        "PART" => {
            if args.len() == 1 {
                Command::PART(args[0].to_owned(), None)
            } else if args.len() == 2 {
                Command::PART(args[0].to_owned(), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "TOPIC" => {
            if args.len() == 1 {
                Command::TOPIC(args[0].to_owned(), None)
            } else if args.len() == 2 {
                Command::TOPIC(args[0].to_owned(), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "NAMES" => {
            if args.is_empty() {
                Command::NAMES(None, None)
            } else if args.len() == 1 {
                Command::NAMES(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::NAMES(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "LIST" => {
            if args.is_empty() {
                Command::LIST(None, None)
            } else if args.len() == 1 {
                Command::LIST(Some(args[0].to_owned()), None)
            } else if args.len() == 2 {
                Command::LIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "INVITE" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::INVITE(args[0].to_owned(), args[1].to_owned())
            }
        }
        "KICK" => {
            if args.len() == 3 {
                Command::KICK(
                    args[0].to_owned(),
                    args[1].to_owned(),
                    Some(args[2].to_owned()),
                )
            } else if args.len() == 2 {
                Command::KICK(args[0].to_owned(), args[1].to_owned(), None)
            } else {
                raw(cmd, args)
            }
        }
        _ => unreachable!("channel::parse called with non-channel command: {}", cmd),
    };

    Ok(result)
}

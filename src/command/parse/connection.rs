
use crate::error::MessageParseError;
use super::super::types::Command;

pub(super) fn raw(cmd: &str, args: Vec<&str>) -> Command {
    Command::Raw(
        cmd.to_owned(),
        args.into_iter().map(|s| s.to_owned()).collect(),
    )
}

pub(super) fn parse(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
    let result = match cmd {
        "PASS" => {
            if args.len() != 1 {
                raw(cmd, args)
            } else {
                Command::PASS(args[0].to_owned())
            }
        }
        "NICK" => {
            if args.len() != 1 {
                raw(cmd, args)
            } else {
                Command::NICK(args[0].to_owned())
            }
        }
        "USER" => {
            if args.len() != 4 {
                raw(cmd, args)
            } else {
                Command::USER(args[0].to_owned(), args[1].to_owned(), args[3].to_owned())
            }
        }
        "OPER" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::OPER(args[0].to_owned(), args[1].to_owned())
            }
        }
        "SERVICE" => {
            if args.len() != 6 {
                raw(cmd, args)
            } else {
                Command::SERVICE(
                    args[0].to_owned(),
                    args[1].to_owned(),
                    args[2].to_owned(),
                    args[3].to_owned(),
                    args[4].to_owned(),
                    args[5].to_owned(),
                )
            }
        }
        "QUIT" => {
            if args.is_empty() {
                Command::QUIT(None)
            } else if args.len() == 1 {
                Command::QUIT(Some(args[0].to_owned()))
            } else {
                raw(cmd, args)
            }
        }
        "SQUIT" => {
            if args.len() != 2 {
                raw(cmd, args)
            } else {
                Command::SQUIT(args[0].to_owned(), args[1].to_owned())
            }
        }
        _ => unreachable!("connection::parse called with non-connection command: {}", cmd),
    };

    Ok(result)
}

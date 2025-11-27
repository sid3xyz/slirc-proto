use super::super::types::Command;
use super::connection::raw;
use crate::error::MessageParseError;

pub(super) fn parse(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
    let result = match cmd {
        "CAP" => {
            if args.len() == 1 {
                match args[0].parse() {
                    Ok(c) => Command::CAP(None, c, None, None),
                    Err(_) => raw(cmd, args),
                }
            } else if args.len() == 2 {
                match args[0].parse() {
                    Ok(c) => Command::CAP(None, c, Some(args[1].to_owned()), None),
                    Err(_) => raw(cmd, args),
                }
            } else if args.len() == 3 {
                if let Ok(cmd_parsed) = args[1].parse() {
                    Command::CAP(
                        Some(args[0].to_owned()),
                        cmd_parsed,
                        Some(args[2].to_owned()),
                        None,
                    )
                } else {
                    raw(cmd, args)
                }
            } else if args.len() == 4 {
                if let Ok(cmd_parsed) = args[1].parse() {
                    Command::CAP(
                        Some(args[0].to_owned()),
                        cmd_parsed,
                        Some(args[2].to_owned()),
                        Some(args[3].to_owned()),
                    )
                } else {
                    raw(cmd, args)
                }
            } else {
                raw(cmd, args)
            }
        }
        "AUTHENTICATE" => {
            if args.len() == 1 {
                Command::AUTHENTICATE(args[0].to_owned())
            } else {
                raw(cmd, args)
            }
        }
        "ACCOUNT" => {
            if args.len() == 1 {
                Command::ACCOUNT(args[0].to_owned())
            } else {
                raw(cmd, args)
            }
        }
        "MONITOR" => {
            if args.len() == 2 {
                Command::MONITOR(args[0].to_owned(), Some(args[1].to_owned()))
            } else if args.len() == 1 {
                Command::MONITOR(args[0].to_owned(), None)
            } else {
                raw(cmd, args)
            }
        }
        "BATCH" => {
            if args.len() == 1 {
                Command::BATCH(args[0].to_owned(), None, None)
            } else if args.len() == 2 {
                match args[1].parse() {
                    Ok(sub) => Command::BATCH(args[0].to_owned(), Some(sub), None),
                    Err(_) => raw(cmd, args),
                }
            } else if args.len() > 2 {
                match args[1].parse() {
                    Ok(sub) => Command::BATCH(
                        args[0].to_owned(),
                        Some(sub),
                        Some(args.iter().skip(2).map(|s| s.to_string()).collect()),
                    ),
                    Err(_) => raw(cmd, args),
                }
            } else {
                raw(cmd, args)
            }
        }
        "CHGHOST" => {
            if args.len() == 2 {
                Command::CHGHOST(args[0].to_owned(), args[1].to_owned())
            } else {
                raw(cmd, args)
            }
        }
        "SETNAME" => {
            if args.len() == 1 {
                Command::SETNAME(args[0].to_owned())
            } else {
                raw(cmd, args)
            }
        }
        _ => unreachable!("ircv3::parse called with non-ircv3 command: {}", cmd),
    };

    Ok(result)
}

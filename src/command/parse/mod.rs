
mod connection;
mod channel;
mod server;
mod user;
mod messaging;
mod ircv3;

use crate::chan::ChannelExt;
use crate::error::MessageParseError;
use crate::mode::Mode;
use super::types::Command;

impl Command {
    pub fn new(cmd: &str, args: Vec<&str>) -> Result<Command, MessageParseError> {
        let cmd_upper = cmd.to_ascii_uppercase();
        let cmd_str = cmd_upper.as_str();


        match cmd_str {

            "PASS" | "NICK" | "USER" | "OPER" | "SERVICE" | "QUIT" | "SQUIT" => {
                connection::parse(cmd_str, args)
            }


            "JOIN" | "PART" | "TOPIC" | "NAMES" | "LIST" | "INVITE" | "KICK" => {
                channel::parse(cmd_str, args)
            }


            "MOTD" | "LUSERS" | "VERSION" | "STATS" | "LINKS" | "TIME" |
            "CONNECT" | "TRACE" | "ADMIN" | "INFO" | "SERVLIST" | "SQUERY" => {
                server::parse(cmd_str, args)
            }


            "WHO" | "WHOIS" | "WHOWAS" => {
                user::parse(cmd_str, args)
            }


            "PRIVMSG" | "NOTICE" | "PING" | "PONG" | "ERROR" |
            "AWAY" | "REHASH" | "DIE" | "RESTART" | "SUMMON" | "USERS" |
            "WALLOPS" | "USERHOST" | "ISON" | "KILL" |

            "SAJOIN" | "SAMODE" | "SANICK" | "SAPART" | "SAQUIT" |

            "NICKSERV" | "CHANSERV" | "OPERSERV" | "BOTSERV" | "HOSTSERV" | "MEMOSERV" => {
                messaging::parse(cmd_str, args)
            }


            "CAP" | "AUTHENTICATE" | "ACCOUNT" | "BATCH" | "CHGHOST" | "SETNAME" |
            "MONITOR" | "TAGMSG" | "WEBIRC" => {
                ircv3::parse(cmd_str, args)
            }


            "MODE" => {
                Ok(if args.is_empty() {
                    connection::raw(cmd, args)
                } else if args[0].is_channel_name() {
                    Command::ChannelMODE(args[0].to_owned(), Mode::as_channel_modes(&args[1..])?)
                } else {
                    Command::UserMODE(args[0].to_owned(), Mode::as_user_modes(&args[1..])?)
                })
            }


            _ => {
                if let Ok(resp) = cmd.parse() {
                    Ok(Command::Response(resp, args.into_iter().map(|s| s.to_owned()).collect()))
                } else {

                    Ok(connection::raw(cmd, args))
                }
            }
        }
    }
}

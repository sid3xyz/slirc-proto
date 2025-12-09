//! Semantic error constructors for `Response`.
//!
//! This module provides static methods on `Response` to construct standard error messages
//! as defined in RFC 2812 and modern IRC specifications.

use crate::command::Command;
use crate::message::Message;
use crate::response::Response;

impl Response {
    /// Helper to construct a Message with a Response command.
    fn error_msg(response: Response, args: Vec<String>) -> Message {
        Message {
            tags: None,
            prefix: None,
            command: Command::Response(response, args),
        }
    }

    // === 400-499 Error Replies ===

    /// `401 ERR_NOSUCHNICK`
    /// `<nickname> :No such nick/channel`
    pub fn err_nosuchnick(client: &str, nickname: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOSUCHNICK,
            vec![
                client.to_string(),
                nickname.to_string(),
                "No such nick/channel".to_string(),
            ],
        )
    }

    /// `403 ERR_NOSUCHCHANNEL`
    /// `<channel name> :No such channel`
    pub fn err_nosuchchannel(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOSUCHCHANNEL,
            vec![
                client.to_string(),
                channel.to_string(),
                "No such channel".to_string(),
            ],
        )
    }

    /// `404 ERR_CANNOTSENDTOCHAN`
    /// `<channel name> :Cannot send to channel`
    pub fn err_cannotsendtochan(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_CANNOTSENDTOCHAN,
            vec![
                client.to_string(),
                channel.to_string(),
                "Cannot send to channel".to_string(),
            ],
        )
    }

    /// `405 ERR_TOOMANYCHANNELS`
    /// `<channel name> :You have joined too many channels`
    pub fn err_toomanychannels(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_TOOMANYCHANNELS,
            vec![
                client.to_string(),
                channel.to_string(),
                "You have joined too many channels".to_string(),
            ],
        )
    }

    /// `406 ERR_WASNOSUCHNICK`
    /// `<nickname> :There was no such nickname`
    pub fn err_wasnosuchnick(client: &str, nickname: &str) -> Message {
        Self::error_msg(
            Response::ERR_WASNOSUCHNICK,
            vec![
                client.to_string(),
                nickname.to_string(),
                "There was no such nickname".to_string(),
            ],
        )
    }

    /// `407 ERR_TOOMANYTARGETS`
    /// `<target> :<error code> recipients. <abort message>`
    pub fn err_toomanytargets(
        client: &str,
        target: &str,
        error_code: &str,
        abort_msg: &str,
    ) -> Message {
        Self::error_msg(
            Response::ERR_TOOMANYTARGETS,
            vec![
                client.to_string(),
                target.to_string(),
                format!("{} recipients. {}", error_code, abort_msg),
            ],
        )
    }

    /// `409 ERR_NOORIGIN`
    /// `:No origin specified`
    pub fn err_noorigin(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOORIGIN,
            vec![client.to_string(), "No origin specified".to_string()],
        )
    }

    /// `411 ERR_NORECIPIENT`
    /// `:No recipient given (<command>)`
    pub fn err_norecipient(client: &str, command: &str) -> Message {
        Self::error_msg(
            Response::ERR_NORECIPIENT,
            vec![
                client.to_string(),
                format!("No recipient given ({})", command),
            ],
        )
    }

    /// `412 ERR_NOTEXTTOSEND`
    /// `:No text to send`
    pub fn err_notexttosend(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOTEXTTOSEND,
            vec![client.to_string(), "No text to send".to_string()],
        )
    }

    /// `413 ERR_NOTOPLEVEL`
    /// `<mask> :No toplevel domain specified`
    pub fn err_notoplevel(client: &str, mask: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOTOPLEVEL,
            vec![
                client.to_string(),
                mask.to_string(),
                "No toplevel domain specified".to_string(),
            ],
        )
    }

    /// `414 ERR_WILDTOPLEVEL`
    /// `<mask> :Wildcard in toplevel domain`
    pub fn err_wildtoplevel(client: &str, mask: &str) -> Message {
        Self::error_msg(
            Response::ERR_WILDTOPLEVEL,
            vec![
                client.to_string(),
                mask.to_string(),
                "Wildcard in toplevel domain".to_string(),
            ],
        )
    }

    /// `415 ERR_BADMASK`
    /// `<mask> :Bad Server/host mask`
    pub fn err_badmask(client: &str, mask: &str) -> Message {
        Self::error_msg(
            Response::ERR_BADMASK,
            vec![
                client.to_string(),
                mask.to_string(),
                "Bad Server/host mask".to_string(),
            ],
        )
    }

    /// `421 ERR_UNKNOWNCOMMAND`
    /// `<command> :Unknown command`
    pub fn err_unknowncommand(client: &str, command: &str) -> Message {
        Self::error_msg(
            Response::ERR_UNKNOWNCOMMAND,
            vec![
                client.to_string(),
                command.to_string(),
                "Unknown command".to_string(),
            ],
        )
    }

    /// `422 ERR_NOMOTD`
    /// `:MOTD File is missing`
    pub fn err_nomotd(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOMOTD,
            vec![client.to_string(), "MOTD File is missing".to_string()],
        )
    }

    /// `423 ERR_NOADMININFO`
    /// `<server> :No administrative info available`
    pub fn err_noadmininfo(client: &str, server: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOADMININFO,
            vec![
                client.to_string(),
                server.to_string(),
                "No administrative info available".to_string(),
            ],
        )
    }

    /// `424 ERR_FILEERROR`
    /// `:File error doing <file op> on <file>`
    pub fn err_fileerror(client: &str, op: &str, file: &str) -> Message {
        Self::error_msg(
            Response::ERR_FILEERROR,
            vec![
                client.to_string(),
                format!("File error doing {} on {}", op, file),
            ],
        )
    }

    /// `431 ERR_NONICKNAMEGIVEN`
    /// `:No nickname given`
    pub fn err_nonicknamegiven(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NONICKNAMEGIVEN,
            vec![client.to_string(), "No nickname given".to_string()],
        )
    }

    /// `432 ERR_ERRONEUSNICKNAME`
    /// `<nick> :Erroneous nickname`
    pub fn err_erroneusnickname(client: &str, nick: &str) -> Message {
        Self::error_msg(
            Response::ERR_ERRONEOUSNICKNAME,
            vec![
                client.to_string(),
                nick.to_string(),
                "Erroneous nickname".to_string(),
            ],
        )
    }

    /// `433 ERR_NICKNAMEINUSE`
    /// `<nick> :Nickname is already in use`
    pub fn err_nicknameinuse(client: &str, nick: &str) -> Message {
        Self::error_msg(
            Response::ERR_NICKNAMEINUSE,
            vec![
                client.to_string(),
                nick.to_string(),
                "Nickname is already in use".to_string(),
            ],
        )
    }

    /// `436 ERR_NICKCOLLISION`
    /// `<nick> :Nickname collision KILL from <user>@<host>`
    pub fn err_nickcollision(client: &str, nick: &str, user: &str, host: &str) -> Message {
        Self::error_msg(
            Response::ERR_NICKCOLLISION,
            vec![
                client.to_string(),
                nick.to_string(),
                format!("Nickname collision KILL from {}@{}", user, host),
            ],
        )
    }

    /// `437 ERR_UNAVAILRESOURCE`
    /// `<nick/channel> :Nick/channel is temporarily unavailable`
    pub fn err_unavailresource(client: &str, resource: &str) -> Message {
        Self::error_msg(
            Response::ERR_UNAVAILRESOURCE,
            vec![
                client.to_string(),
                resource.to_string(),
                "Nick/channel is temporarily unavailable".to_string(),
            ],
        )
    }

    /// `441 ERR_USERNOTINCHANNEL`
    /// `<nick> <channel> :They aren't on that channel`
    pub fn err_usernotinchannel(client: &str, nick: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_USERNOTINCHANNEL,
            vec![
                client.to_string(),
                nick.to_string(),
                channel.to_string(),
                "They aren't on that channel".to_string(),
            ],
        )
    }

    /// `442 ERR_NOTONCHANNEL`
    /// `<channel> :You're not on that channel`
    pub fn err_notonchannel(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOTONCHANNEL,
            vec![
                client.to_string(),
                channel.to_string(),
                "You're not on that channel".to_string(),
            ],
        )
    }

    /// `443 ERR_USERONCHANNEL`
    /// `<user> <channel> :is already on channel`
    pub fn err_useronchannel(client: &str, user: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_USERONCHANNEL,
            vec![
                client.to_string(),
                user.to_string(),
                channel.to_string(),
                "is already on channel".to_string(),
            ],
        )
    }

    /// `444 ERR_NOLOGIN`
    /// `<user> :User not logged in`
    pub fn err_nologin(client: &str, user: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOLOGIN,
            vec![
                client.to_string(),
                user.to_string(),
                "User not logged in".to_string(),
            ],
        )
    }

    /// `445 ERR_SUMMONDISABLED`
    /// `:SUMMON has been disabled`
    pub fn err_summondisabled(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_SUMMONDISABLED,
            vec![client.to_string(), "SUMMON has been disabled".to_string()],
        )
    }

    /// `446 ERR_USERSDISABLED`
    /// `:USERS has been disabled`
    pub fn err_usersdisabled(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_USERSDISABLED,
            vec![client.to_string(), "USERS has been disabled".to_string()],
        )
    }

    /// `451 ERR_NOTREGISTERED`
    /// `:You have not registered`
    pub fn err_notregistered(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOTREGISTERED,
            vec![client.to_string(), "You have not registered".to_string()],
        )
    }

    /// `461 ERR_NEEDMOREPARAMS`
    /// `<command> :Not enough parameters`
    pub fn err_needmoreparams(client: &str, command: &str) -> Message {
        Self::error_msg(
            Response::ERR_NEEDMOREPARAMS,
            vec![
                client.to_string(),
                command.to_string(),
                "Not enough parameters".to_string(),
            ],
        )
    }

    /// `462 ERR_ALREADYREGISTRED`
    /// `:Unauthorized command (already registered)`
    pub fn err_alreadyregistred(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_ALREADYREGISTERED,
            vec![
                client.to_string(),
                "Unauthorized command (already registered)".to_string(),
            ],
        )
    }

    /// `463 ERR_NOPERMFORHOST`
    /// `:Your host isn't among the privileged`
    pub fn err_nopermforhost(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOPERMFORHOST,
            vec![
                client.to_string(),
                "Your host isn't among the privileged".to_string(),
            ],
        )
    }

    /// `464 ERR_PASSWDMISMATCH`
    /// `:Password incorrect`
    pub fn err_passwdmismatch(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_PASSWDMISMATCH,
            vec![client.to_string(), "Password incorrect".to_string()],
        )
    }

    /// `465 ERR_YOUREBANNEDCREEP`
    /// `:You are banned from this server`
    pub fn err_yourebannedcreep(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_YOUREBANNEDCREEP,
            vec![
                client.to_string(),
                "You are banned from this server".to_string(),
            ],
        )
    }

    /// `466 ERR_YOUWILLBEBANNED`
    pub fn err_youwillbebanned(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_YOUWILLBEBANNED,
            vec![client.to_string(), "You will be banned".to_string()],
        )
    }

    /// `467 ERR_KEYSET`
    /// `<channel> :Channel key already set`
    pub fn err_keyset(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_KEYSET,
            vec![
                client.to_string(),
                channel.to_string(),
                "Channel key already set".to_string(),
            ],
        )
    }

    /// `471 ERR_CHANNELISFULL`
    /// `<channel> :Cannot join channel (+l)`
    pub fn err_channelisfull(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_CHANNELISFULL,
            vec![
                client.to_string(),
                channel.to_string(),
                "Cannot join channel (+l)".to_string(),
            ],
        )
    }

    /// `472 ERR_UNKNOWNMODE`
    /// `<char> :is unknown mode char to me for <channel>`
    pub fn err_unknownmode(client: &str, mode_char: char, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_UNKNOWNMODE,
            vec![
                client.to_string(),
                mode_char.to_string(),
                format!("is unknown mode char to me for {}", channel),
            ],
        )
    }

    /// `473 ERR_INVITEONLYCHAN`
    /// `<channel> :Cannot join channel (+i)`
    pub fn err_inviteonlychan(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_INVITEONLYCHAN,
            vec![
                client.to_string(),
                channel.to_string(),
                "Cannot join channel (+i)".to_string(),
            ],
        )
    }

    /// `474 ERR_BANNEDFROMCHAN`
    /// `<channel> :Cannot join channel (+b)`
    pub fn err_bannedfromchan(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_BANNEDFROMCHAN,
            vec![
                client.to_string(),
                channel.to_string(),
                "Cannot join channel (+b)".to_string(),
            ],
        )
    }

    /// `475 ERR_BADCHANNELKEY`
    /// `<channel> :Cannot join channel (+k)`
    pub fn err_badchannelkey(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_BADCHANNELKEY,
            vec![
                client.to_string(),
                channel.to_string(),
                "Cannot join channel (+k)".to_string(),
            ],
        )
    }

    /// `476 ERR_BADCHANMASK`
    /// `<channel> :Bad Channel Mask`
    pub fn err_badchanmask(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_BADCHANMASK,
            vec![
                client.to_string(),
                channel.to_string(),
                "Bad Channel Mask".to_string(),
            ],
        )
    }

    /// `477 ERR_NOCHANMODES`
    /// `<channel> :Channel doesn't support modes`
    pub fn err_nochanmodes(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOCHANMODES,
            vec![
                client.to_string(),
                channel.to_string(),
                "Channel doesn't support modes".to_string(),
            ],
        )
    }

    /// `478 ERR_BANLISTFULL`
    /// `<channel> <char> :Channel list is full`
    pub fn err_banlistfull(client: &str, channel: &str, mode_char: char) -> Message {
        Self::error_msg(
            Response::ERR_BANLISTFULL,
            vec![
                client.to_string(),
                channel.to_string(),
                mode_char.to_string(),
                "Channel list is full".to_string(),
            ],
        )
    }

    /// `481 ERR_NOPRIVILEGES`
    /// `:Permission Denied- You're not an IRC operator`
    pub fn err_noprivileges(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOPRIVILEGES,
            vec![
                client.to_string(),
                "Permission Denied- You're not an IRC operator".to_string(),
            ],
        )
    }

    /// `482 ERR_CHANOPRIVSNEEDED`
    /// `<channel> :You're not channel operator`
    pub fn err_chanoprivsneeded(client: &str, channel: &str) -> Message {
        Self::error_msg(
            Response::ERR_CHANOPRIVSNEEDED,
            vec![
                client.to_string(),
                channel.to_string(),
                "You're not channel operator".to_string(),
            ],
        )
    }

    /// `483 ERR_CANTKILLSERVER`
    /// `:You can't kill a server!`
    pub fn err_cantkillserver(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_CANTKILLSERVER,
            vec![client.to_string(), "You can't kill a server!".to_string()],
        )
    }

    /// `484 ERR_RESTRICTED`
    /// `:Your connection is restricted!`
    pub fn err_restricted(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_RESTRICTED,
            vec![
                client.to_string(),
                "Your connection is restricted!".to_string(),
            ],
        )
    }

    /// `485 ERR_UNIQOPPRIVSNEEDED`
    /// `:You're not the original channel operator`
    pub fn err_uniqopprivsneeded(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_UNIQOPPRIVSNEEDED,
            vec![
                client.to_string(),
                "You're not the original channel operator".to_string(),
            ],
        )
    }

    /// `491 ERR_NOOPERHOST`
    /// `:No O-lines for your host`
    pub fn err_nooperhost(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_NOOPERHOST,
            vec![client.to_string(), "No O-lines for your host".to_string()],
        )
    }

    /// `501 ERR_UMODEUNKNOWNFLAG`
    /// `:Unknown MODE flag`
    pub fn err_umodeunknownflag(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_UMODEUNKNOWNFLAG,
            vec![client.to_string(), "Unknown MODE flag".to_string()],
        )
    }

    /// `502 ERR_USERSDONTMATCH`
    /// `:Cannot change mode for other users`
    pub fn err_usersdontmatch(client: &str) -> Message {
        Self::error_msg(
            Response::ERR_USERSDONTMATCH,
            vec![
                client.to_string(),
                "Cannot change mode for other users".to_string(),
            ],
        )
    }
}

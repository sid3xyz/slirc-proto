//! IRC command types.
//!
//! This module provides type-safe representations of IRC commands
//! as defined in RFC 2812 and extended by IRCv3 and modern IRC servers.
//!
//! # Reference
//! - RFC 2812: Internet Relay Chat: Client Protocol
//! - IRCv3 specifications: <https://ircv3.net/>

use crate::mode::{ChannelMode, Mode, UserMode};
use crate::response::Response;

use super::subcommands::{BatchSubCommand, CapSubCommand, ChatHistorySubCommand, MessageReference};

/// IRC command with its parameters.
///
/// This enum represents all known IRC commands with type-safe parameters.
/// Unknown commands are captured in the `Raw` variant.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Command {
    // === Connection Registration (RFC 2812 Section 3.1) ===
    /// `PASS password`
    PASS(String),
    /// `NICK nickname`
    NICK(String),
    /// `USER username mode realname`
    USER(String, String, String),
    /// `OPER name password`
    OPER(String, String),
    /// User MODE command: `MODE nickname [modes]`
    UserMODE(String, Vec<Mode<UserMode>>),
    /// `SERVICE nickname reserved distribution type reserved info`
    SERVICE(String, String, String, String, String, String),
    /// `QUIT [message]`
    QUIT(Option<String>),
    /// `SQUIT server comment`
    SQUIT(String, String),

    // === Channel Operations (RFC 2812 Section 3.2) ===
    /// `JOIN channels [keys] [realname]`
    JOIN(String, Option<String>, Option<String>),
    /// `PART channels [message]`
    PART(String, Option<String>),
    /// Channel MODE command: `MODE channel [modes]`
    ChannelMODE(String, Vec<Mode<ChannelMode>>),
    /// `TOPIC channel [topic]`
    TOPIC(String, Option<String>),
    /// `NAMES [channels] [target]`
    NAMES(Option<String>, Option<String>),
    /// `LIST [channels] [target]`
    LIST(Option<String>, Option<String>),
    /// `INVITE nickname channel`
    INVITE(String, String),
    /// `KICK channels users [comment]`
    KICK(String, String, Option<String>),

    // === Messaging (RFC 2812 Section 3.3) ===
    /// `PRIVMSG target text`
    PRIVMSG(String, String),
    /// `NOTICE target text`
    NOTICE(String, String),

    // === Server Queries (RFC 2812 Section 3.4) ===
    /// `MOTD [target]`
    MOTD(Option<String>),
    /// `LUSERS [mask] [target]`
    LUSERS(Option<String>, Option<String>),
    /// `VERSION [target]`
    VERSION(Option<String>),
    /// `STATS [query] [target]`
    STATS(Option<String>, Option<String>),
    /// `LINKS [[remote] mask]`
    LINKS(Option<String>, Option<String>),
    /// `TIME [target]`
    TIME(Option<String>),
    /// `CONNECT target port [remote]`
    CONNECT(String, String, Option<String>),
    /// `TRACE [target]`
    TRACE(Option<String>),
    /// `ADMIN [target]`
    ADMIN(Option<String>),
    /// `INFO [target]`
    INFO(Option<String>),

    // === Service Queries (RFC 2812 Section 3.5) ===
    /// `SERVLIST [mask] [type]`
    SERVLIST(Option<String>, Option<String>),
    /// `SQUERY servicename text`
    SQUERY(String, String),

    // === User Queries (RFC 2812 Section 3.6) ===
    /// `WHO [mask] [o]`
    WHO(Option<String>, Option<bool>),
    /// `WHOIS [target] nickmasks`
    WHOIS(Option<String>, String),
    /// `WHOWAS nickname [count] [target]`
    WHOWAS(String, Option<String>, Option<String>),

    // === Miscellaneous (RFC 2812 Section 3.7) ===
    /// `KILL nickname comment`
    KILL(String, String),
    /// `PING server1 [server2]`
    PING(String, Option<String>),
    /// `PONG server1 [server2]`
    PONG(String, Option<String>),
    /// `ERROR message`
    ERROR(String),

    // === Optional Features (RFC 2812 Section 4) ===
    /// `AWAY [message]`
    AWAY(Option<String>),
    /// `REHASH` (no parameters)
    REHASH,
    /// `DIE` (no parameters)
    DIE,
    /// `RESTART` (no parameters)
    RESTART,
    /// `SUMMON user [target] [channel]`
    SUMMON(String, Option<String>, Option<String>),
    /// `USERS [target]`
    USERS(Option<String>),
    /// `WALLOPS text`
    WALLOPS(String),
    /// `USERHOST nicknames...`
    USERHOST(Vec<String>),
    /// `ISON nicknames...`
    ISON(Vec<String>),

    // === Services Commands (common extensions) ===
    /// `SAJOIN nick channel`
    SAJOIN(String, String),
    /// `SAMODE target modes [params]`
    SAMODE(String, String, Option<String>),
    /// `SANICK oldnick newnick`
    SANICK(String, String),
    /// `SAPART nick channel`
    SAPART(String, String),
    /// `SAQUIT nick reason`
    SAQUIT(String, String),
    /// NickServ shorthand
    NICKSERV(Vec<String>),
    /// ChanServ shorthand
    CHANSERV(String),
    /// OperServ shorthand
    OPERSERV(String),
    /// BotServ shorthand
    BOTSERV(String),
    /// HostServ shorthand
    HOSTSERV(String),
    /// MemoServ shorthand
    MEMOSERV(String),

    // === IRCv3 Extensions ===
    /// `CAP [target] subcommand [params] [capabilities]`
    CAP(
        Option<String>,
        CapSubCommand,
        Option<String>,
        Option<String>,
    ),
    /// `AUTHENTICATE mechanism_or_data`
    AUTHENTICATE(String),
    /// `ACCOUNT accountname`
    ACCOUNT(String),
    /// `MONITOR +/-/C/L/S [targets]`
    MONITOR(String, Option<String>),
    /// `BATCH +/-reference [type] [params...]`
    BATCH(String, Option<BatchSubCommand>, Option<Vec<String>>),
    /// `CHGHOST user host`
    CHGHOST(String, String),
    /// `SETNAME realname`
    SETNAME(String),
    /// `TAGMSG target` - IRCv3 message-tags: message with only tags, no text
    TAGMSG(String),
    /// `WEBIRC password gateway hostname ip [:options]` - WebIRC/CGI:IRC identification
    WEBIRC(String, String, String, String, Option<String>),
    /// `CHATHISTORY subcommand target/params...` - IRCv3 chat history retrieval
    ///
    /// Variants:
    /// - `LATEST <target> <* | msgref> <limit>`
    /// - `BEFORE/AFTER/AROUND <target> <msgref> <limit>`
    /// - `BETWEEN <target> <msgref> <msgref> <limit>`
    /// - `TARGETS <timestamp> <timestamp> <limit>`
    CHATHISTORY {
        subcommand: ChatHistorySubCommand,
        target: String,
        msg_ref1: MessageReference,
        msg_ref2: Option<MessageReference>,
        limit: u32,
    },

    // === Standard Replies (IRCv3) ===
    /// `FAIL command code [context...] :description`
    FAIL(String, String, Vec<String>),
    /// `WARN command code [context...] :description`
    WARN(String, String, Vec<String>),
    /// `NOTE command code [context...] :description`
    NOTE(String, String, Vec<String>),

    // === Numeric Response ===
    /// Numeric response from server
    Response(Response, Vec<String>),

    // === Unknown/Raw Commands ===
    /// Unknown command captured as raw
    Raw(String, Vec<String>),
}

/// A borrowed reference to a command.
///
/// Used for zero-copy parsing of IRC messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandRef<'a> {
    /// Command name
    pub name: &'a str,
    /// Command arguments
    pub args: Vec<&'a str>,
}

impl<'a> CommandRef<'a> {
    /// Create a new command reference.
    pub fn new(name: &'a str, args: Vec<&'a str>) -> Self {
        Self { name, args }
    }

    /// Convert this reference to an owned raw command string.
    pub fn to_raw_string(&self) -> String {
        if self.args.is_empty() {
            self.name.to_string()
        } else {
            let capacity =
                self.name.len() + 1 + self.args.iter().map(|a| a.len() + 1).sum::<usize>();
            let mut s = String::with_capacity(capacity);
            s.push_str(self.name);
            s.push(' ');
            s.push_str(&self.args.join(" "));
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_ref_to_raw_string() {
        let cmd = CommandRef::new("PRIVMSG", vec!["#channel", "hello"]);
        assert_eq!(cmd.to_raw_string(), "PRIVMSG #channel hello");

        let cmd = CommandRef::new("PING", vec![]);
        assert_eq!(cmd.to_raw_string(), "PING");
    }

    #[test]
    fn test_command_equality() {
        let cmd1 = Command::NICK("test".to_string());
        let cmd2 = Command::NICK("test".to_string());
        assert_eq!(cmd1, cmd2);

        let cmd3 = Command::NICK("other".to_string());
        assert_ne!(cmd1, cmd3);
    }
}

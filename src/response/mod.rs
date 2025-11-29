//! IRC numeric response codes as defined in RFC 2812 and modern IRC specifications.
//!
//! This module provides an enumeration of IRC server response codes (numerics).
//! Response codes are three-digit numbers sent by servers to indicate the result
//! of commands or to provide information.
//!
//! # Reference
//! - RFC 2812: Internet Relay Chat: Client Protocol
//! - Modern IRC documentation: <https://modern.ircdocs.horse/>

#![allow(non_camel_case_types)]

use std::str::FromStr;

/// IRC server response code.
///
/// Response codes are categorized as:
/// - 001-099: Connection/registration
/// - 200-399: Command replies
/// - 400-599: Error replies
/// - 600-999: Extended/modern numerics
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum Response {
    // === Connection Registration (001-099) ===
    /// 001 - Welcome to the IRC network
    RPL_WELCOME = 1,
    /// 002 - Your host is running version
    RPL_YOURHOST = 2,
    /// 003 - Server creation date
    RPL_CREATED = 3,
    /// 004 - Server info (name, version, user modes, channel modes)
    RPL_MYINFO = 4,
    /// 005 - Server supported features (ISUPPORT)
    RPL_ISUPPORT = 5,
    /// 010 - Bounce to another server
    RPL_BOUNCE = 10,
    /// 042 - Your unique ID
    RPL_YOURID = 42,

    // === Command Responses (200-399) ===

    // Trace replies
    /// 200 - Trace link
    RPL_TRACELINK = 200,
    /// 201 - Trace connecting
    RPL_TRACECONNECTING = 201,
    /// 202 - Trace handshake
    RPL_TRACEHANDSHAKE = 202,
    /// 203 - Trace unknown
    RPL_TRACEUNKNOWN = 203,
    /// 204 - Trace operator
    RPL_TRACEOPERATOR = 204,
    /// 205 - Trace user
    RPL_TRACEUSER = 205,
    /// 206 - Trace server
    RPL_TRACESERVER = 206,
    /// 207 - Trace service
    RPL_TRACESERVICE = 207,
    /// 208 - Trace new type
    RPL_TRACENEWTYPE = 208,
    /// 209 - Trace class
    RPL_TRACECLASS = 209,
    /// 210 - Trace reconnect
    RPL_TRACERECONNECT = 210,

    // Stats replies
    /// 211 - Stats link info
    RPL_STATSLINKINFO = 211,
    /// 212 - Stats commands
    RPL_STATSCOMMANDS = 212,
    /// 216 - Stats K-line
    RPL_STATSKLINE = 216,
    /// 219 - End of stats
    RPL_ENDOFSTATS = 219,
    /// 220 - Stats D-line
    RPL_STATSDLINE = 220,
    /// 221 - User mode string
    RPL_UMODEIS = 221,
    /// 226 - Stats shun
    RPL_STATSSHUN = 226,
    /// 234 - Service list
    RPL_SERVLIST = 234,
    /// 235 - Service list end
    RPL_SERVLISTEND = 235,
    /// 242 - Stats uptime
    RPL_STATSUPTIME = 242,
    /// 243 - Stats O-line
    RPL_STATSOLINE = 243,

    // Luser replies
    /// 251 - Luser client count
    RPL_LUSERCLIENT = 251,
    /// 252 - Luser operator count
    RPL_LUSEROP = 252,
    /// 253 - Luser unknown connections
    RPL_LUSERUNKNOWN = 253,
    /// 254 - Luser channel count
    RPL_LUSERCHANNELS = 254,
    /// 255 - Luser local info
    RPL_LUSERME = 255,

    // Admin replies
    /// 256 - Admin info start
    RPL_ADMINME = 256,
    /// 257 - Admin location 1
    RPL_ADMINLOC1 = 257,
    /// 258 - Admin location 2
    RPL_ADMINLOC2 = 258,
    /// 259 - Admin email
    RPL_ADMINEMAIL = 259,

    // Trace/stats end
    /// 261 - Trace log
    RPL_TRACELOG = 261,
    /// 262 - Trace end
    RPL_TRACEEND = 262,
    /// 263 - Try again later
    RPL_TRYAGAIN = 263,

    // Local/global users
    /// 265 - Local users
    RPL_LOCALUSERS = 265,
    /// 266 - Global users
    RPL_GLOBALUSERS = 266,
    /// 276 - WHOIS certificate fingerprint
    RPL_WHOISCERTFP = 276,

    // Misc
    /// 300 - None (dummy placeholder)
    RPL_NONE = 300,
    /// 301 - User is away
    RPL_AWAY = 301,
    /// 302 - USERHOST reply
    RPL_USERHOST = 302,
    /// 303 - ISON reply
    RPL_ISON = 303,
    /// 305 - You are no longer marked as away
    RPL_UNAWAY = 305,
    /// 306 - You have been marked as away
    RPL_NOWAWAY = 306,

    // WHOIS replies
    /// 311 - WHOIS user info
    RPL_WHOISUSER = 311,
    /// 312 - WHOIS server
    RPL_WHOISSERVER = 312,
    /// 313 - WHOIS operator status
    RPL_WHOISOPERATOR = 313,
    /// 314 - WHOWAS user info
    RPL_WHOWASUSER = 314,
    /// 315 - End of WHO
    RPL_ENDOFWHO = 315,
    /// 317 - WHOIS idle time
    RPL_WHOISIDLE = 317,
    /// 318 - End of WHOIS
    RPL_ENDOFWHOIS = 318,
    /// 319 - WHOIS channels
    RPL_WHOISCHANNELS = 319,

    // Channel/list replies
    /// 321 - List start
    RPL_LISTSTART = 321,
    /// 322 - List entry
    RPL_LIST = 322,
    /// 323 - List end
    RPL_LISTEND = 323,
    /// 324 - Channel mode
    RPL_CHANNELMODEIS = 324,
    /// 325 - Channel unique operator
    RPL_UNIQOPIS = 325,
    /// 329 - Channel creation time
    RPL_CREATIONTIME = 329,
    /// 330 - WHOIS account name
    RPL_WHOISACCOUNT = 330,
    /// 331 - No topic set
    RPL_NOTOPIC = 331,
    /// 332 - Channel topic
    RPL_TOPIC = 332,
    /// 333 - Topic set by/time
    RPL_TOPICWHOTIME = 333,
    /// 335 - WHOIS bot flag
    RPL_WHOISBOT = 335,
    /// 338 - WHOIS actually (real host)
    RPL_WHOISACTUALLY = 338,
    /// 340 - USERIP reply
    RPL_USERIP = 340,
    /// 341 - Inviting user to channel
    RPL_INVITING = 341,
    /// 342 - Summoning user
    RPL_SUMMONING = 342,
    /// 346 - Invite list entry
    RPL_INVITELIST = 346,
    /// 347 - End of invite list
    RPL_ENDOFINVITELIST = 347,
    /// 348 - Exception list entry
    RPL_EXCEPTLIST = 348,
    /// 349 - End of exception list
    RPL_ENDOFEXCEPTLIST = 349,
    /// 351 - Server version
    RPL_VERSION = 351,
    /// 352 - WHO reply
    RPL_WHOREPLY = 352,
    /// 353 - NAMES reply
    RPL_NAMREPLY = 353,
    /// 354 - WHOX reply
    RPL_WHOSPCRPL = 354,

    // Links/info
    /// 364 - Links entry
    RPL_LINKS = 364,
    /// 365 - End of links
    RPL_ENDOFLINKS = 365,
    /// 366 - End of NAMES
    RPL_ENDOFNAMES = 366,
    /// 367 - Ban list entry
    RPL_BANLIST = 367,
    /// 368 - End of ban list
    RPL_ENDOFBANLIST = 368,
    /// 369 - End of WHOWAS
    RPL_ENDOFWHOWAS = 369,
    /// 371 - Info text
    RPL_INFO = 371,
    /// 372 - MOTD text
    RPL_MOTD = 372,
    /// 374 - End of info
    RPL_ENDOFINFO = 374,
    /// 375 - MOTD start
    RPL_MOTDSTART = 375,
    /// 376 - End of MOTD
    RPL_ENDOFMOTD = 376,
    /// 378 - WHOIS host
    RPL_WHOISHOST = 378,
    /// 379 - WHOIS modes
    RPL_WHOISMODES = 379,

    // Oper/rehash
    /// 381 - You are now an operator
    RPL_YOUREOPER = 381,
    /// 382 - Rehashing config
    RPL_REHASHING = 382,
    /// 383 - You are a service
    RPL_YOURESERVICE = 383,
    /// 391 - Server time
    RPL_TIME = 391,
    /// 392 - Users start
    RPL_USERSSTART = 392,
    /// 393 - Users entry
    RPL_USERS = 393,
    /// 394 - End of users
    RPL_ENDOFUSERS = 394,
    /// 395 - No users
    RPL_NOUSERS = 395,
    /// 396 - Host hidden
    RPL_HOSTHIDDEN = 396,

    // === Error Replies (400-599) ===
    /// 400 - Unknown error
    ERR_UNKNOWNERROR = 400,
    /// 401 - No such nick
    ERR_NOSUCHNICK = 401,
    /// 402 - No such server
    ERR_NOSUCHSERVER = 402,
    /// 403 - No such channel
    ERR_NOSUCHCHANNEL = 403,
    /// 404 - Cannot send to channel
    ERR_CANNOTSENDTOCHAN = 404,
    /// 405 - Too many channels
    ERR_TOOMANYCHANNELS = 405,
    /// 406 - Was no such nick
    ERR_WASNOSUCHNICK = 406,
    /// 407 - Too many targets
    ERR_TOOMANYTARGETS = 407,
    /// 408 - No such service
    ERR_NOSUCHSERVICE = 408,
    /// 409 - No origin
    ERR_NOORIGIN = 409,
    /// 411 - No recipient
    ERR_NORECIPIENT = 411,
    /// 412 - No text to send
    ERR_NOTEXTTOSEND = 412,
    /// 413 - No top level domain
    ERR_NOTOPLEVEL = 413,
    /// 414 - Wildcard in top level
    ERR_WILDTOPLEVEL = 414,
    /// 415 - Bad mask
    ERR_BADMASK = 415,
    /// 417 - Input too long
    ERR_INPUTTOOLONG = 417,
    /// 421 - Unknown command
    ERR_UNKNOWNCOMMAND = 421,
    /// 422 - No MOTD
    ERR_NOMOTD = 422,
    /// 423 - No admin info
    ERR_NOADMININFO = 423,
    /// 424 - File error
    ERR_FILEERROR = 424,
    /// 431 - No nickname given
    ERR_NONICKNAMEGIVEN = 431,
    /// 432 - Erroneous nickname
    ERR_ERRONEOUSNICKNAME = 432,
    /// 433 - Nickname in use
    ERR_NICKNAMEINUSE = 433,
    /// 436 - Nick collision
    ERR_NICKCOLLISION = 436,
    /// 437 - Resource unavailable
    ERR_UNAVAILRESOURCE = 437,
    /// 441 - User not in channel
    ERR_USERNOTINCHANNEL = 441,
    /// 442 - Not on channel
    ERR_NOTONCHANNEL = 442,
    /// 443 - User on channel
    ERR_USERONCHANNEL = 443,
    /// 444 - No login
    ERR_NOLOGIN = 444,
    /// 445 - Summon disabled
    ERR_SUMMONDISABLED = 445,
    /// 446 - Users disabled
    ERR_USERSDISABLED = 446,
    /// 451 - Not registered
    ERR_NOTREGISTERED = 451,
    /// 461 - Need more params
    ERR_NEEDMOREPARAMS = 461,
    /// 462 - Already registered
    ERR_ALREADYREGISTERED = 462,
    /// 463 - No permission for host
    ERR_NOPERMFORHOST = 463,
    /// 464 - Password mismatch
    ERR_PASSWDMISMATCH = 464,
    /// 465 - You are banned
    ERR_YOUREBANNEDCREEP = 465,
    /// 466 - You will be banned
    ERR_YOUWILLBEBANNED = 466,
    /// 467 - Key already set
    ERR_KEYSET = 467,
    /// 471 - Channel is full
    ERR_CHANNELISFULL = 471,
    /// 472 - Unknown mode
    ERR_UNKNOWNMODE = 472,
    /// 473 - Invite only channel
    ERR_INVITEONLYCHAN = 473,
    /// 474 - Banned from channel
    ERR_BANNEDFROMCHAN = 474,
    /// 475 - Bad channel key
    ERR_BADCHANNELKEY = 475,
    /// 476 - Bad channel mask
    ERR_BADCHANMASK = 476,
    /// 477 - Need registered nick
    ERR_NEEDREGGEDNICK = 477,
    /// 478 - Ban list full
    ERR_BANLISTFULL = 478,
    /// 479 - Bad channel name
    ERR_BADCHANNAME = 479,
    /// 481 - No privileges
    ERR_NOPRIVILEGES = 481,
    /// 482 - Channel op privileges needed
    ERR_CHANOPRIVSNEEDED = 482,
    /// 483 - Cannot kill server
    ERR_CANTKILLSERVER = 483,
    /// 484 - Restricted
    ERR_RESTRICTED = 484,
    /// 485 - Unique op privileges needed
    ERR_UNIQOPPRIVSNEEDED = 485,
    /// 489 - Secure only channel
    ERR_SECUREONLYCHAN = 489,
    /// 491 - No oper host
    ERR_NOOPERHOST = 491,
    /// 501 - Unknown mode flag
    ERR_UMODEUNKNOWNFLAG = 501,
    /// 502 - Users don't match
    ERR_USERSDONTMATCH = 502,
    /// 524 - Help not found
    ERR_HELPNOTFOUND = 524,

    // === Extended/Modern Numerics (600+) ===
    /// 606 - Map entry
    RPL_MAP = 606,
    /// 607 - End of map
    RPL_MAPEND = 607,
    /// 632 - Rules start
    RPL_RULESTART = 632,
    /// 633 - Rules text
    RPL_RULES = 633,
    /// 634 - End of rules
    RPL_ENDOFRULES = 634,
    /// 635 - No rules
    ERR_NORULES = 635,
    /// 646 - Stats P-line
    RPL_STATSPLINE = 646,
    /// 671 - WHOIS secure connection
    RPL_WHOISSECURE = 671,
    /// 704 - Help start
    RPL_HELPSTART = 704,
    /// 705 - Help text
    RPL_HELPTXT = 705,
    /// 706 - End of help
    RPL_ENDOFHELP = 706,
    /// 710 - Knock
    RPL_KNOCK = 710,
    /// 711 - Knock delivered
    RPL_KNOCKDLVR = 711,
    /// 712 - Too many knocks
    ERR_TOOMANYKNOCK = 712,
    /// 713 - Channel open
    ERR_CHANOPEN = 713,
    /// 714 - Knock on channel
    ERR_KNOCKONCHAN = 714,
    /// 723 - No privileges
    ERR_NOPRIVS = 723,
    /// 728 - Quiet list entry
    RPL_QUIETLIST = 728,
    /// 729 - End of quiet list
    RPL_ENDOFQUIETLIST = 729,

    // Monitor
    /// 730 - Monitor online
    RPL_MONONLINE = 730,
    /// 731 - Monitor offline
    RPL_MONOFFLINE = 731,
    /// 732 - Monitor list
    RPL_MONLIST = 732,
    /// 733 - End of monitor list
    RPL_ENDOFMONLIST = 733,
    /// 734 - Monitor list full
    ERR_MONLISTFULL = 734,

    // Metadata
    /// 760 - WHOIS key/value
    RPL_WHOISKEYVALUE = 760,
    /// 761 - Key/value
    RPL_KEYVALUE = 761,
    /// 765 - Target invalid
    ERR_TARGETINVALID = 765,
    /// 766 - No matching key
    ERR_NOMATCHINGKEY = 766,
    /// 767 - Key invalid
    ERR_KEYINVALID = 767,
    /// 768 - Key not set
    ERR_KEYNOTSET = 768,
    /// 769 - Key no permission
    ERR_KEYNOPERMISSION = 769,

    // SASL (IRCv3)
    /// 900 - Logged in
    RPL_LOGGEDIN = 900,
    /// 901 - Logged out
    RPL_LOGGEDOUT = 901,
    /// 902 - Nick locked
    ERR_NICKLOCKED = 902,
    /// 903 - SASL success
    RPL_SASLSUCCESS = 903,
    /// 904 - SASL fail
    ERR_SASLFAIL = 904,
    /// 905 - SASL too long
    ERR_SASLTOOLONG = 905,
    /// 906 - SASL aborted
    ERR_SASLABORT = 906,
    /// 907 - SASL already authenticated
    ERR_SASLALREADY = 907,
    /// 908 - SASL mechanisms
    RPL_SASLMECHS = 908,
}

/// Deprecated alias for [`Response::ERR_ALREADYREGISTERED`].
///
/// The original RFC 1459/2812 used the typo'd spelling "ALREADYREGISTRED".
/// Modern IRC documentation uses the correct spelling "ALREADYREGISTERED".
#[deprecated(since = "1.2.0", note = "use ERR_ALREADYREGISTERED (correct spelling)")]
pub const ERR_ALREADYREGISTRED: Response = Response::ERR_ALREADYREGISTERED;

impl Response {
    /// Returns the numeric code as u16
    #[inline]
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Creates a Response from a numeric code
    pub fn from_code(code: u16) -> Option<Response> {
        Some(match code {
            1 => Response::RPL_WELCOME,
            2 => Response::RPL_YOURHOST,
            3 => Response::RPL_CREATED,
            4 => Response::RPL_MYINFO,
            5 => Response::RPL_ISUPPORT,
            10 => Response::RPL_BOUNCE,
            42 => Response::RPL_YOURID,
            200 => Response::RPL_TRACELINK,
            201 => Response::RPL_TRACECONNECTING,
            202 => Response::RPL_TRACEHANDSHAKE,
            203 => Response::RPL_TRACEUNKNOWN,
            204 => Response::RPL_TRACEOPERATOR,
            205 => Response::RPL_TRACEUSER,
            206 => Response::RPL_TRACESERVER,
            207 => Response::RPL_TRACESERVICE,
            208 => Response::RPL_TRACENEWTYPE,
            209 => Response::RPL_TRACECLASS,
            210 => Response::RPL_TRACERECONNECT,
            211 => Response::RPL_STATSLINKINFO,
            212 => Response::RPL_STATSCOMMANDS,
            216 => Response::RPL_STATSKLINE,
            219 => Response::RPL_ENDOFSTATS,
            220 => Response::RPL_STATSDLINE,
            221 => Response::RPL_UMODEIS,
            226 => Response::RPL_STATSSHUN,
            234 => Response::RPL_SERVLIST,
            235 => Response::RPL_SERVLISTEND,
            242 => Response::RPL_STATSUPTIME,
            243 => Response::RPL_STATSOLINE,
            251 => Response::RPL_LUSERCLIENT,
            252 => Response::RPL_LUSEROP,
            253 => Response::RPL_LUSERUNKNOWN,
            254 => Response::RPL_LUSERCHANNELS,
            255 => Response::RPL_LUSERME,
            256 => Response::RPL_ADMINME,
            257 => Response::RPL_ADMINLOC1,
            258 => Response::RPL_ADMINLOC2,
            259 => Response::RPL_ADMINEMAIL,
            261 => Response::RPL_TRACELOG,
            262 => Response::RPL_TRACEEND,
            263 => Response::RPL_TRYAGAIN,
            265 => Response::RPL_LOCALUSERS,
            266 => Response::RPL_GLOBALUSERS,
            276 => Response::RPL_WHOISCERTFP,
            300 => Response::RPL_NONE,
            301 => Response::RPL_AWAY,
            302 => Response::RPL_USERHOST,
            303 => Response::RPL_ISON,
            305 => Response::RPL_UNAWAY,
            306 => Response::RPL_NOWAWAY,
            311 => Response::RPL_WHOISUSER,
            312 => Response::RPL_WHOISSERVER,
            313 => Response::RPL_WHOISOPERATOR,
            314 => Response::RPL_WHOWASUSER,
            315 => Response::RPL_ENDOFWHO,
            317 => Response::RPL_WHOISIDLE,
            318 => Response::RPL_ENDOFWHOIS,
            319 => Response::RPL_WHOISCHANNELS,
            321 => Response::RPL_LISTSTART,
            322 => Response::RPL_LIST,
            323 => Response::RPL_LISTEND,
            324 => Response::RPL_CHANNELMODEIS,
            325 => Response::RPL_UNIQOPIS,
            329 => Response::RPL_CREATIONTIME,
            330 => Response::RPL_WHOISACCOUNT,
            331 => Response::RPL_NOTOPIC,
            332 => Response::RPL_TOPIC,
            333 => Response::RPL_TOPICWHOTIME,
            335 => Response::RPL_WHOISBOT,
            338 => Response::RPL_WHOISACTUALLY,
            340 => Response::RPL_USERIP,
            341 => Response::RPL_INVITING,
            342 => Response::RPL_SUMMONING,
            346 => Response::RPL_INVITELIST,
            347 => Response::RPL_ENDOFINVITELIST,
            348 => Response::RPL_EXCEPTLIST,
            349 => Response::RPL_ENDOFEXCEPTLIST,
            351 => Response::RPL_VERSION,
            352 => Response::RPL_WHOREPLY,
            353 => Response::RPL_NAMREPLY,
            354 => Response::RPL_WHOSPCRPL,
            364 => Response::RPL_LINKS,
            365 => Response::RPL_ENDOFLINKS,
            366 => Response::RPL_ENDOFNAMES,
            367 => Response::RPL_BANLIST,
            368 => Response::RPL_ENDOFBANLIST,
            369 => Response::RPL_ENDOFWHOWAS,
            371 => Response::RPL_INFO,
            372 => Response::RPL_MOTD,
            374 => Response::RPL_ENDOFINFO,
            375 => Response::RPL_MOTDSTART,
            376 => Response::RPL_ENDOFMOTD,
            378 => Response::RPL_WHOISHOST,
            379 => Response::RPL_WHOISMODES,
            381 => Response::RPL_YOUREOPER,
            382 => Response::RPL_REHASHING,
            383 => Response::RPL_YOURESERVICE,
            391 => Response::RPL_TIME,
            392 => Response::RPL_USERSSTART,
            393 => Response::RPL_USERS,
            394 => Response::RPL_ENDOFUSERS,
            395 => Response::RPL_NOUSERS,
            396 => Response::RPL_HOSTHIDDEN,
            400 => Response::ERR_UNKNOWNERROR,
            401 => Response::ERR_NOSUCHNICK,
            402 => Response::ERR_NOSUCHSERVER,
            403 => Response::ERR_NOSUCHCHANNEL,
            404 => Response::ERR_CANNOTSENDTOCHAN,
            405 => Response::ERR_TOOMANYCHANNELS,
            406 => Response::ERR_WASNOSUCHNICK,
            407 => Response::ERR_TOOMANYTARGETS,
            408 => Response::ERR_NOSUCHSERVICE,
            409 => Response::ERR_NOORIGIN,
            411 => Response::ERR_NORECIPIENT,
            412 => Response::ERR_NOTEXTTOSEND,
            413 => Response::ERR_NOTOPLEVEL,
            414 => Response::ERR_WILDTOPLEVEL,
            415 => Response::ERR_BADMASK,
            417 => Response::ERR_INPUTTOOLONG,
            421 => Response::ERR_UNKNOWNCOMMAND,
            422 => Response::ERR_NOMOTD,
            423 => Response::ERR_NOADMININFO,
            424 => Response::ERR_FILEERROR,
            431 => Response::ERR_NONICKNAMEGIVEN,
            432 => Response::ERR_ERRONEOUSNICKNAME,
            433 => Response::ERR_NICKNAMEINUSE,
            436 => Response::ERR_NICKCOLLISION,
            437 => Response::ERR_UNAVAILRESOURCE,
            441 => Response::ERR_USERNOTINCHANNEL,
            442 => Response::ERR_NOTONCHANNEL,
            443 => Response::ERR_USERONCHANNEL,
            444 => Response::ERR_NOLOGIN,
            445 => Response::ERR_SUMMONDISABLED,
            446 => Response::ERR_USERSDISABLED,
            451 => Response::ERR_NOTREGISTERED,
            461 => Response::ERR_NEEDMOREPARAMS,
            462 => Response::ERR_ALREADYREGISTERED,
            463 => Response::ERR_NOPERMFORHOST,
            464 => Response::ERR_PASSWDMISMATCH,
            465 => Response::ERR_YOUREBANNEDCREEP,
            466 => Response::ERR_YOUWILLBEBANNED,
            467 => Response::ERR_KEYSET,
            471 => Response::ERR_CHANNELISFULL,
            472 => Response::ERR_UNKNOWNMODE,
            473 => Response::ERR_INVITEONLYCHAN,
            474 => Response::ERR_BANNEDFROMCHAN,
            475 => Response::ERR_BADCHANNELKEY,
            476 => Response::ERR_BADCHANMASK,
            477 => Response::ERR_NEEDREGGEDNICK,
            478 => Response::ERR_BANLISTFULL,
            479 => Response::ERR_BADCHANNAME,
            481 => Response::ERR_NOPRIVILEGES,
            482 => Response::ERR_CHANOPRIVSNEEDED,
            483 => Response::ERR_CANTKILLSERVER,
            484 => Response::ERR_RESTRICTED,
            485 => Response::ERR_UNIQOPPRIVSNEEDED,
            489 => Response::ERR_SECUREONLYCHAN,
            491 => Response::ERR_NOOPERHOST,
            501 => Response::ERR_UMODEUNKNOWNFLAG,
            502 => Response::ERR_USERSDONTMATCH,
            524 => Response::ERR_HELPNOTFOUND,
            606 => Response::RPL_MAP,
            607 => Response::RPL_MAPEND,
            632 => Response::RPL_RULESTART,
            633 => Response::RPL_RULES,
            634 => Response::RPL_ENDOFRULES,
            635 => Response::ERR_NORULES,
            646 => Response::RPL_STATSPLINE,
            671 => Response::RPL_WHOISSECURE,
            704 => Response::RPL_HELPSTART,
            705 => Response::RPL_HELPTXT,
            706 => Response::RPL_ENDOFHELP,
            710 => Response::RPL_KNOCK,
            711 => Response::RPL_KNOCKDLVR,
            712 => Response::ERR_TOOMANYKNOCK,
            713 => Response::ERR_CHANOPEN,
            714 => Response::ERR_KNOCKONCHAN,
            723 => Response::ERR_NOPRIVS,
            728 => Response::RPL_QUIETLIST,
            729 => Response::RPL_ENDOFQUIETLIST,
            730 => Response::RPL_MONONLINE,
            731 => Response::RPL_MONOFFLINE,
            732 => Response::RPL_MONLIST,
            733 => Response::RPL_ENDOFMONLIST,
            734 => Response::ERR_MONLISTFULL,
            760 => Response::RPL_WHOISKEYVALUE,
            761 => Response::RPL_KEYVALUE,
            765 => Response::ERR_TARGETINVALID,
            766 => Response::ERR_NOMATCHINGKEY,
            767 => Response::ERR_KEYINVALID,
            768 => Response::ERR_KEYNOTSET,
            769 => Response::ERR_KEYNOPERMISSION,
            900 => Response::RPL_LOGGEDIN,
            901 => Response::RPL_LOGGEDOUT,
            902 => Response::ERR_NICKLOCKED,
            903 => Response::RPL_SASLSUCCESS,
            904 => Response::ERR_SASLFAIL,
            905 => Response::ERR_SASLTOOLONG,
            906 => Response::ERR_SASLABORT,
            907 => Response::ERR_SASLALREADY,
            908 => Response::RPL_SASLMECHS,
            _ => return None,
        })
    }

    /// Check if this is an error response (4xx, 5xx, or specific error codes)
    #[inline]
    pub fn is_error(&self) -> bool {
        let code = self.code();
        (400..600).contains(&code)
            || code == 723
            || code == 734
            || (765..=769).contains(&code)
            || (902..=907).contains(&code)
    }

    /// Check if this is a success/informational response
    #[inline]
    pub fn is_success(&self) -> bool {
        !self.is_error()
    }

    /// Check if this is a connection registration response (001-099)
    #[inline]
    pub fn is_registration(&self) -> bool {
        self.code() < 100
    }

    /// Check if this is a command reply (200-399)
    #[inline]
    pub fn is_reply(&self) -> bool {
        let code = self.code();
        (200..400).contains(&code)
    }

    /// Check if this is a SASL-related response (900-908)
    #[inline]
    pub fn is_sasl(&self) -> bool {
        let code = self.code();
        (900..=908).contains(&code)
    }

    /// Check if this is a channel-related response
    #[inline]
    pub fn is_channel_related(&self) -> bool {
        matches!(
            self,
            Response::RPL_TOPIC
                | Response::RPL_NOTOPIC
                | Response::RPL_TOPICWHOTIME
                | Response::RPL_NAMREPLY
                | Response::RPL_ENDOFNAMES
                | Response::RPL_CHANNELMODEIS
                | Response::RPL_CREATIONTIME
                | Response::RPL_BANLIST
                | Response::RPL_ENDOFBANLIST
                | Response::RPL_EXCEPTLIST
                | Response::RPL_ENDOFEXCEPTLIST
                | Response::RPL_INVITELIST
                | Response::RPL_ENDOFINVITELIST
                | Response::RPL_QUIETLIST
                | Response::RPL_ENDOFQUIETLIST
                | Response::ERR_NOSUCHCHANNEL
                | Response::ERR_CANNOTSENDTOCHAN
                | Response::ERR_TOOMANYCHANNELS
                | Response::ERR_CHANNELISFULL
                | Response::ERR_INVITEONLYCHAN
                | Response::ERR_BANNEDFROMCHAN
                | Response::ERR_BADCHANNELKEY
                | Response::ERR_BADCHANMASK
                | Response::ERR_BADCHANNAME
                | Response::ERR_CHANOPRIVSNEEDED
                | Response::ERR_NOTONCHANNEL
                | Response::ERR_USERNOTINCHANNEL
                | Response::ERR_USERONCHANNEL
                | Response::ERR_NEEDREGGEDNICK
                | Response::ERR_BANLISTFULL
                | Response::ERR_SECUREONLYCHAN
        )
    }

    /// Check if this is a WHOIS/WHOWAS-related response
    #[inline]
    pub fn is_whois_related(&self) -> bool {
        matches!(
            self,
            Response::RPL_WHOISUSER
                | Response::RPL_WHOISSERVER
                | Response::RPL_WHOISOPERATOR
                | Response::RPL_WHOISIDLE
                | Response::RPL_ENDOFWHOIS
                | Response::RPL_WHOISCHANNELS
                | Response::RPL_WHOISACCOUNT
                | Response::RPL_WHOISBOT
                | Response::RPL_WHOISACTUALLY
                | Response::RPL_WHOISHOST
                | Response::RPL_WHOISMODES
                | Response::RPL_WHOISCERTFP
                | Response::RPL_WHOISSECURE
                | Response::RPL_WHOISKEYVALUE
                | Response::RPL_WHOWASUSER
                | Response::RPL_ENDOFWHOWAS
        )
    }

    /// Returns the RFC 2812 category name for this response
    pub fn category(&self) -> &'static str {
        let code = self.code();
        match code {
            1..=99 => "Connection Registration",
            200..=299 => "Command Replies (Trace/Stats)",
            300..=399 => "Command Replies (User/Channel)",
            400..=499 => "Error Replies",
            500..=599 => "Error Replies (Server)",
            600..=699 => "Extended Replies",
            700..=799 => "Extended Replies (IRCv3)",
            900..=999 => "SASL/Account",
            _ => "Unknown",
        }
    }
}

impl FromStr for Response {
    type Err = ParseResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code: u16 = s.parse().map_err(|_| ParseResponseError::InvalidFormat)?;
        Response::from_code(code).ok_or(ParseResponseError::UnknownCode(code))
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:03}", self.code())
    }
}

/// Error when parsing a response code
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseResponseError {
    /// The string was not a valid number
    InvalidFormat,
    /// The numeric code is not a known response
    UnknownCode(u16),
}

impl std::fmt::Display for ParseResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid response code format"),
            Self::UnknownCode(code) => write!(f, "unknown response code: {}", code),
        }
    }
}

impl std::error::Error for ParseResponseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_code() {
        assert_eq!(Response::RPL_WELCOME.code(), 1);
        assert_eq!(Response::ERR_NICKNAMEINUSE.code(), 433);
        assert_eq!(Response::RPL_ENDOFMOTD.code(), 376);
    }

    #[test]
    fn test_from_code() {
        assert_eq!(Response::from_code(1), Some(Response::RPL_WELCOME));
        assert_eq!(Response::from_code(433), Some(Response::ERR_NICKNAMEINUSE));
        assert_eq!(Response::from_code(9999), None);
    }

    #[test]
    fn test_is_error() {
        assert!(!Response::RPL_WELCOME.is_error());
        assert!(Response::ERR_NICKNAMEINUSE.is_error());
        assert!(Response::ERR_NOSUCHNICK.is_error());
    }

    #[test]
    fn test_parse() {
        assert_eq!("001".parse::<Response>().unwrap(), Response::RPL_WELCOME);
        assert_eq!(
            "433".parse::<Response>().unwrap(),
            Response::ERR_NICKNAMEINUSE
        );
        assert!("abc".parse::<Response>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Response::RPL_WELCOME), "001");
        assert_eq!(format!("{}", Response::ERR_NICKNAMEINUSE), "433");
    }

    #[test]
    fn test_is_reply() {
        assert!(Response::RPL_AWAY.is_reply());
        assert!(Response::RPL_TOPIC.is_reply());
        assert!(!Response::RPL_WELCOME.is_reply());
        assert!(!Response::ERR_NOSUCHNICK.is_reply());
    }

    #[test]
    fn test_is_sasl() {
        assert!(Response::RPL_LOGGEDIN.is_sasl());
        assert!(Response::RPL_SASLSUCCESS.is_sasl());
        assert!(Response::ERR_SASLFAIL.is_sasl());
        assert!(!Response::RPL_WELCOME.is_sasl());
    }

    #[test]
    fn test_is_channel_related() {
        assert!(Response::RPL_TOPIC.is_channel_related());
        assert!(Response::RPL_NAMREPLY.is_channel_related());
        assert!(Response::ERR_NOSUCHCHANNEL.is_channel_related());
        assert!(!Response::RPL_WELCOME.is_channel_related());
    }

    #[test]
    fn test_is_whois_related() {
        assert!(Response::RPL_WHOISUSER.is_whois_related());
        assert!(Response::RPL_ENDOFWHOIS.is_whois_related());
        assert!(!Response::RPL_WELCOME.is_whois_related());
    }

    #[test]
    fn test_category() {
        assert_eq!(Response::RPL_WELCOME.category(), "Connection Registration");
        assert_eq!(
            Response::RPL_TRACELINK.category(),
            "Command Replies (Trace/Stats)"
        );
        assert_eq!(
            Response::RPL_TOPIC.category(),
            "Command Replies (User/Channel)"
        );
        assert_eq!(Response::ERR_NOSUCHNICK.category(), "Error Replies");
        assert_eq!(Response::RPL_SASLSUCCESS.category(), "SASL/Account");
    }
}

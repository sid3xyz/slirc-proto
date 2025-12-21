//! Helper methods and trait implementations for IRC response codes.
//!
//! This module provides utility methods for Response enum including:
//! - Code conversion (from_code, code)
//! - Type checking (is_error, is_reply, etc.)
//! - Category classification
//! - Display/parsing traits

use super::Response;
use std::str::FromStr;

impl Response {
    /// Returns the numeric code as u16
    #[inline]
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Creates a Response from a numeric code
    pub fn from_code(code: u16) -> Option<Response> {
        // Try success codes first
        if let Some(resp) = Response::from_success_code(code) {
            return Some(resp);
        }
        // Try error codes
        if let Some(resp) = Response::from_error_code(code) {
            return Some(resp);
        }
        None
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

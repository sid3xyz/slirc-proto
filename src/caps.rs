//! IRCv3 capability negotiation support.
//!
//! This module provides types and utilities for IRCv3 capability negotiation,
//! allowing clients and servers to negotiate optional protocol extensions.
//!
//! # Reference
//! - IRCv3 Capability Negotiation: <https://ircv3.net/specs/extensions/capability-negotiation>
//! - Individual capability specifications: <https://ircv3.net/irc/>

use std::collections::HashSet;

/// Definition of a known IRCv3 capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDef {
    /// Capability name (e.g., "multi-prefix")
    pub name: &'static str,
    /// Minimum CAP version that supports this capability (301 or 302)
    pub version: u32,
    /// Default value for capabilities that take parameters
    pub value: Option<&'static str>,
    /// Human-readable description
    pub description: &'static str,
}

/// Known IRCv3 capability types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Show all user prefix modes in NAMES
    MultiPrefix,
    /// SASL authentication
    Sasl,
    /// Notify of account login/logout
    AccountNotify,
    /// Notify of away status changes
    AwayNotify,
    /// Extended JOIN with account and realname
    ExtendedJoin,
    /// MONITOR command for presence tracking
    Monitor,
    /// Add account tag to messages
    AccountTag,
    /// Message batching
    Batch,
    /// Notify of capability changes
    CapNotify,
    /// Notify of hostname changes
    ChgHost,
    /// Echo messages back to sender
    EchoMessage,
    /// Notify of channel invites
    InviteNotify,
    /// Server-time message tags
    ServerTime,
    /// Full nick!user@host in NAMES
    UserhostInNames,
    /// SETNAME command for changing realname
    SetName,
    /// Client message tags support
    MessageTags,
    /// Unique message IDs
    Msgid,
    /// Label request/response correlation
    LabeledResponse,
    /// FAIL/WARN/NOTE standard replies
    StandardReplies,
    /// Strict Transport Security
    Sts,
    /// Unknown/custom capability
    Custom(String),
}

impl AsRef<str> for Capability {
    fn as_ref(&self) -> &str {
        match self {
            Self::MultiPrefix => "multi-prefix",
            Self::Sasl => "sasl",
            Self::AccountNotify => "account-notify",
            Self::AwayNotify => "away-notify",
            Self::ExtendedJoin => "extended-join",
            Self::Monitor => "monitor",
            Self::AccountTag => "account-tag",
            Self::Batch => "batch",
            Self::CapNotify => "cap-notify",
            Self::ChgHost => "chghost",
            Self::EchoMessage => "echo-message",
            Self::InviteNotify => "invite-notify",
            Self::ServerTime => "server-time",
            Self::UserhostInNames => "userhost-in-names",
            Self::SetName => "setname",
            Self::MessageTags => "message-tags",
            Self::Msgid => "msgid",
            Self::LabeledResponse => "labeled-response",
            Self::StandardReplies => "standard-replies",
            Self::Sts => "sts",
            Self::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        match s {
            "multi-prefix" => Self::MultiPrefix,
            "sasl" => Self::Sasl,
            "account-notify" => Self::AccountNotify,
            "away-notify" => Self::AwayNotify,
            "extended-join" => Self::ExtendedJoin,
            "monitor" => Self::Monitor,
            "account-tag" => Self::AccountTag,
            "batch" => Self::Batch,
            "cap-notify" => Self::CapNotify,
            "chghost" => Self::ChgHost,
            "echo-message" => Self::EchoMessage,
            "invite-notify" => Self::InviteNotify,
            "server-time" => Self::ServerTime,
            "userhost-in-names" => Self::UserhostInNames,
            "setname" => Self::SetName,
            "message-tags" => Self::MessageTags,
            "msgid" => Self::Msgid,
            "labeled-response" => Self::LabeledResponse,
            "standard-replies" => Self::StandardReplies,
            "sts" => Self::Sts,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Static list of supported capabilities.
pub const CAPABILITIES: &[CapabilityDef] = &[
    // CAP 3.1 capabilities
    CapabilityDef {
        name: "multi-prefix",
        version: 301,
        value: None,
        description: "Show all user modes in NAMES (@+nick for op+voice)",
    },
    CapabilityDef {
        name: "userhost-in-names",
        version: 301,
        value: None,
        description: "Include full nick!user@host in NAMES replies",
    },
    CapabilityDef {
        name: "away-notify",
        version: 301,
        value: None,
        description: "Broadcast AWAY status changes to channel members",
    },
    CapabilityDef {
        name: "account-notify",
        version: 301,
        value: None,
        description: "Account tag on messages + ACCOUNT command for login/logout",
    },
    CapabilityDef {
        name: "extended-join",
        version: 301,
        value: None,
        description: "JOIN includes account + realname",
    },
    CapabilityDef {
        name: "sasl",
        version: 301,
        value: Some("PLAIN"),
        description: "SASL authentication (PLAIN mechanism)",
    },
    CapabilityDef {
        name: "monitor",
        version: 301,
        value: None,
        description: "Online/offline status tracking",
    },
    // CAP 3.2 capabilities
    CapabilityDef {
        name: "account-tag",
        version: 302,
        value: None,
        description: "Add account tag to messages from logged-in users",
    },
    CapabilityDef {
        name: "echo-message",
        version: 302,
        value: None,
        description: "Send copy of PRIVMSG/NOTICE back to sender",
    },
    CapabilityDef {
        name: "server-time",
        version: 302,
        value: None,
        description: "Add time tag to messages (ISO 8601)",
    },
    CapabilityDef {
        name: "message-tags",
        version: 302,
        value: None,
        description: "Parse client tags from incoming messages",
    },
    CapabilityDef {
        name: "msgid",
        version: 302,
        value: None,
        description: "Unique message IDs for deduplication",
    },
    CapabilityDef {
        name: "labeled-response",
        version: 302,
        value: None,
        description: "Echo label tag for request/response correlation",
    },
    CapabilityDef {
        name: "batch",
        version: 302,
        value: None,
        description: "Multi-line response grouping",
    },
    CapabilityDef {
        name: "cap-notify",
        version: 302,
        value: None,
        description: "Server notifies clients of capability changes (CAP NEW/DEL)",
    },
    CapabilityDef {
        name: "chghost",
        version: 302,
        value: None,
        description: "Notify when user's hostname changes (CHGHOST command)",
    },
    CapabilityDef {
        name: "invite-notify",
        version: 302,
        value: None,
        description: "Notify channel members when someone is invited",
    },
    CapabilityDef {
        name: "setname",
        version: 302,
        value: None,
        description: "Change realname with SETNAME command",
    },
    CapabilityDef {
        name: "standard-replies",
        version: 302,
        value: None,
        description: "Machine-parseable FAIL/WARN/NOTE responses",
    },
    CapabilityDef {
        name: "sts",
        version: 302,
        value: Some("port=6697,duration=2592000"),
        description: "Strict Transport Security - upgrade plaintext to TLS",
    },
];

/// CAP negotiation version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationVersion {
    /// CAP 3.1
    V301,
    /// CAP 3.2
    V302,
}

impl NegotiationVersion {
    /// Get the numeric version value.
    pub fn version(&self) -> u32 {
        match self {
            Self::V301 => 301,
            Self::V302 => 302,
        }
    }
}

/// Build a space-separated list of capabilities for CAP LS response.
///
/// # Arguments
/// * `version` - CAP version (301 or 302)
/// * `tls_port` - Optional TLS port for STS capability
pub fn get_cap_list(version: u32, tls_port: Option<u16>) -> String {
    let caps: Vec<String> = CAPABILITIES
        .iter()
        .filter(|cap| {
            // Only include STS if we have a TLS port
            if cap.name == "sts" && tls_port.is_none() {
                return false;
            }
            cap.version <= version
        })
        .map(|cap| {
            if version >= 302 && cap.value.is_some() {
                if cap.name == "sts" {
                    if let Some(port) = tls_port {
                        format!("{}=port={},duration=2592000", cap.name, port)
                    } else {
                        cap.name.to_string()
                    }
                } else {
                    format!("{}={}", cap.name, cap.value.unwrap())
                }
            } else {
                cap.name.to_string()
            }
        })
        .collect();
    
    caps.join(" ")
}

/// Check if a capability name is supported.
pub fn is_supported(name: &str) -> bool {
    CAPABILITIES.iter().any(|cap| cap.name == name)
}

/// Get all supported capability names.
pub fn get_all_names() -> Vec<&'static str> {
    CAPABILITIES.iter().map(|cap| cap.name).collect()
}

/// Parse a CAP REQ request and separate accepted from rejected capabilities.
///
/// Returns (accepted, rejected) capability lists.
pub fn parse_request(requested: &str) -> (Vec<String>, Vec<String>) {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    
    for cap in requested.split_whitespace() {
        // Check for removal prefix
        let (is_removal, cap_name) = if let Some(name) = cap.strip_prefix('-') {
            (true, name)
        } else {
            (false, cap)
        };
        
        // Strip any value suffix (cap=value)
        let cap_base = cap_name.split('=').next().unwrap_or(cap_name);
        
        if is_supported(cap_base) {
            accepted.push(if is_removal {
                format!("-{}", cap_base)
            } else {
                cap_base.to_string()
            });
        } else {
            rejected.push(cap_base.to_string());
        }
    }
    
    (accepted, rejected)
}

/// Apply capability changes to an active set.
///
/// Changes prefixed with '-' remove capabilities, others add them.
/// Returns true if any changes were made.
pub fn apply_changes(capabilities: &mut HashSet<String>, changes: &[String]) -> bool {
    let mut modified = false;
    
    for change in changes {
        if let Some(cap_name) = change.strip_prefix('-') {
            if capabilities.remove(cap_name) {
                modified = true;
            }
        } else if capabilities.insert(change.clone()) {
            modified = true;
        }
    }
    
    modified
}

/// Format a CAP NEW message for notifying clients of new capabilities.
pub fn format_cap_new(nickname: &str, server_name: &str, new_caps: &[&str]) -> String {
    format!(":{} CAP {} NEW :{}", server_name, nickname, new_caps.join(" "))
}

/// Format a CAP DEL message for notifying clients of removed capabilities.
pub fn format_cap_del(nickname: &str, server_name: &str, removed_caps: &[&str]) -> String {
    format!(":{} CAP {} DEL :{}", server_name, nickname, removed_caps.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_as_ref() {
        assert_eq!(Capability::MultiPrefix.as_ref(), "multi-prefix");
        assert_eq!(Capability::Sasl.as_ref(), "sasl");
    }
    
    #[test]
    fn test_capability_from_str() {
        assert_eq!(Capability::from("multi-prefix"), Capability::MultiPrefix);
        assert_eq!(Capability::from("sasl"), Capability::Sasl);
        assert_eq!(
            Capability::from("unknown-cap"),
            Capability::Custom("unknown-cap".to_string())
        );
    }
    
    #[test]
    fn test_is_supported() {
        assert!(is_supported("multi-prefix"));
        assert!(is_supported("sasl"));
        assert!(!is_supported("unknown-capability"));
    }
    
    #[test]
    fn test_parse_request() {
        let (accepted, rejected) = parse_request("multi-prefix sasl unknown-cap");
        assert!(accepted.contains(&"multi-prefix".to_string()));
        assert!(accepted.contains(&"sasl".to_string()));
        assert!(rejected.contains(&"unknown-cap".to_string()));
    }
    
    #[test]
    fn test_parse_request_removal() {
        let (accepted, _) = parse_request("-multi-prefix");
        assert!(accepted.contains(&"-multi-prefix".to_string()));
    }
    
    #[test]
    fn test_apply_changes() {
        let mut caps = HashSet::new();
        
        let changes = vec!["multi-prefix".to_string(), "sasl".to_string()];
        assert!(apply_changes(&mut caps, &changes));
        assert!(caps.contains("multi-prefix"));
        assert!(caps.contains("sasl"));
        
        let removal = vec!["-sasl".to_string()];
        assert!(apply_changes(&mut caps, &removal));
        assert!(!caps.contains("sasl"));
    }
    
    #[test]
    fn test_cap_list_v301() {
        let list = get_cap_list(301, None);
        assert!(list.contains("multi-prefix"));
        assert!(!list.contains("echo-message")); // v302 only
        assert!(!list.contains("sts")); // needs TLS port
    }
    
    #[test]
    fn test_cap_list_v302() {
        let list = get_cap_list(302, Some(6697));
        assert!(list.contains("multi-prefix"));
        assert!(list.contains("echo-message"));
        assert!(list.contains("sts=port=6697"));
    }
}


//! Comprehensive RFC 1459/2812 and IRCv3 compliance tests.
//!
//! This module tests specific edge cases and requirements from:
//! - RFC 1459: Internet Relay Chat Protocol
//! - RFC 2812: Internet Relay Chat: Client Protocol
//! - IRCv3 Message Tags: https://ircv3.net/specs/extensions/message-tags
//!
//! Run with: `cargo test --test rfc_ircv3_compliance`

use slirc_proto::message::tags::{escape_tag_value, unescape_tag_value};
use slirc_proto::{Command, Message, MessageRef};

// =============================================================================
// IRCv3 MESSAGE TAGS ESCAPING (https://ircv3.net/specs/extensions/message-tags)
// =============================================================================

mod tag_escaping {
    use super::*;

    /// IRCv3 specifies these escape sequences:
    /// - `\:` → `;` (semicolon)
    /// - `\s` → ` ` (space)
    /// - `\\` → `\` (backslash)  
    /// - `\r` → CR (carriage return)
    /// - `\n` → LF (line feed)
    #[test]
    fn test_unescape_semicolon() {
        assert_eq!(unescape_tag_value("a\\:b"), "a;b");
    }

    #[test]
    fn test_unescape_space() {
        assert_eq!(unescape_tag_value("hello\\sworld"), "hello world");
    }

    #[test]
    fn test_unescape_backslash() {
        assert_eq!(unescape_tag_value("path\\\\file"), "path\\file");
    }

    #[test]
    fn test_unescape_carriage_return() {
        assert_eq!(unescape_tag_value("line\\rend"), "line\rend");
    }

    #[test]
    fn test_unescape_line_feed() {
        assert_eq!(unescape_tag_value("line\\nend"), "line\nend");
    }

    #[test]
    fn test_unescape_combined() {
        // All escape sequences together
        let input = "a\\:b\\sc\\\\d\\re\\nf";
        let expected = "a;b c\\d\re\nf";
        assert_eq!(unescape_tag_value(input), expected);
    }

    #[test]
    fn test_unescape_trailing_backslash() {
        // Trailing backslash with no following char should be dropped per IRCv3
        assert_eq!(unescape_tag_value("test\\"), "test");
    }

    #[test]
    fn test_unescape_unknown_escape() {
        // Unknown escape sequences: \x becomes x (backslash dropped)
        assert_eq!(unescape_tag_value("a\\xb"), "axb");
    }

    #[test]
    fn test_escape_roundtrip() {
        let test_values = vec![
            "simple",
            "with space",
            "with;semicolon",
            "with\\backslash",
            "with\nnewline",
            "with\rcarriage",
            "complex; \\ \n \r all",
        ];

        for original in test_values {
            let mut escaped = String::new();
            escape_tag_value(&mut escaped, original).unwrap();
            let unescaped = unescape_tag_value(&escaped);
            assert_eq!(
                unescaped, original,
                "Roundtrip failed: '{}' -> '{}' -> '{}'",
                original, escaped, unescaped
            );
        }
    }
}

// =============================================================================
// IRCv3 TAG PARSING IN MESSAGES
// =============================================================================

mod tag_parsing {
    use super::*;

    #[test]
    fn test_tag_with_escaped_semicolon() {
        let raw = "@key=value\\:with\\:semicolons :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        let owned = msg.to_owned();

        // The tag value should have actual semicolons after unescaping
        let value = owned.tag_value("key");
        assert_eq!(value, Some("value;with;semicolons"));
    }

    #[test]
    fn test_tag_with_escaped_spaces() {
        let raw = "@key=hello\\sworld :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        let owned = msg.to_owned();

        assert_eq!(owned.tag_value("key"), Some("hello world"));
    }

    #[test]
    fn test_tag_without_value() {
        // IRCv3 allows tags without values (flag-style)
        let raw = "@+typing :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");

        assert!(msg.has_tag("+typing"));
        // Value should be empty string for flag tags
        assert_eq!(msg.tag_value("+typing"), Some(""));
    }

    #[test]
    fn test_multiple_tags_mixed() {
        let raw = "@+typing;time=2023-01-01T00:00:00Z;msgid=abc :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");

        assert!(msg.has_tag("+typing"));
        assert_eq!(msg.tag_value("time"), Some("2023-01-01T00:00:00Z"));
        assert_eq!(msg.tag_value("msgid"), Some("abc"));
    }

    #[test]
    fn test_client_only_tag_prefix() {
        // Client-only tags start with +
        let raw = "@+example.com/custom=value :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.tag_value("+example.com/custom"), Some("value"));
    }

    #[test]
    fn test_vendor_prefixed_tag() {
        // Vendor-prefixed tags
        let raw = "@example.com/foo=bar :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.tag_value("example.com/foo"), Some("bar"));
    }
}

// =============================================================================
// RFC 1459/2812 MESSAGE FORMAT
// =============================================================================

mod message_format {
    use super::*;

    #[test]
    fn test_max_line_length_512() {
        // RFC 1459/2812: Maximum message length is 512 bytes including CRLF
        let long_text = "a".repeat(500);
        let raw = format!("PRIVMSG #ch :{}\r\n", long_text);

        // Should parse (but compliance check would flag it)
        let msg: Message = raw.parse().expect("Should parse");
        match &msg.command {
            Command::PRIVMSG(_, text) => assert_eq!(text.len(), 500),
            _ => panic!("Expected PRIVMSG"),
        }
    }

    #[test]
    fn test_crlf_line_ending() {
        let raw = "PING :server\r\n";
        let msg = MessageRef::parse(raw).expect("Should parse with CRLF");
        assert_eq!(msg.command_name(), "PING");
    }

    #[test]
    fn test_lf_only_line_ending() {
        // Many servers accept LF-only
        let raw = "PING :server\n";
        let msg = MessageRef::parse(raw).expect("Should parse with LF only");
        assert_eq!(msg.command_name(), "PING");
    }

    #[test]
    fn test_no_line_ending() {
        // Parser should handle messages without line ending
        let raw = "PING :server";
        let msg = MessageRef::parse(raw).expect("Should parse without line ending");
        assert_eq!(msg.command_name(), "PING");
    }

    #[test]
    fn test_empty_trailing_parameter() {
        // Empty trailing is valid: "PRIVMSG #ch :" means empty message
        let raw = "PRIVMSG #channel :";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.args(), &["#channel", ""]);
    }

    #[test]
    fn test_trailing_with_spaces() {
        let raw = ":nick PRIVMSG #ch :hello world with spaces";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.arg(1), Some("hello world with spaces"));
    }

    #[test]
    fn test_trailing_preserves_leading_colon() {
        // Double colon at start of trailing: the second colon is literal
        let raw = "PRIVMSG #ch ::starts with colon";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.arg(1), Some(":starts with colon"));
    }

    #[test]
    fn test_numeric_command() {
        // Numeric responses are 3 digits
        let raw = ":server 001 nick :Welcome to the network";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert!(msg.is_numeric());
        assert_eq!(msg.numeric_code(), Some(1));
    }

    #[test]
    fn test_max_params_15() {
        // RFC allows up to 15 parameters (14 middle + 1 trailing)
        let raw = "CMD 1 2 3 4 5 6 7 8 9 10 11 12 13 14 :15th trailing";
        let msg = MessageRef::parse(raw).expect("Should parse 15 params");
        assert_eq!(msg.args().len(), 15);
        assert_eq!(msg.arg(14), Some("15th trailing"));
    }
}

// =============================================================================
// PREFIX PARSING (RFC 2812 Section 2.3.1)
// =============================================================================

mod prefix_parsing {
    use super::*;

    #[test]
    fn test_full_user_prefix() {
        // nick!user@host format
        let raw = ":nick!user@host.example.com PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.source_nickname(), Some("nick"));
        assert_eq!(msg.source_user(), Some("user"));
        assert_eq!(msg.source_host(), Some("host.example.com"));
    }

    #[test]
    fn test_nick_at_host_prefix() {
        // Some servers send nick@host (no user)
        let raw = ":nick@host.example.com PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.source_nickname(), Some("nick"));
        // User may or may not be present depending on parser behavior
    }

    #[test]
    fn test_nick_only_prefix() {
        // Just nickname
        let raw = ":nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.source_nickname(), Some("nick"));
    }

    #[test]
    fn test_server_prefix() {
        // Server names contain dots
        let raw = ":irc.example.com 001 nick :Welcome";
        let msg = MessageRef::parse(raw).expect("Should parse");
        // Server prefix should be detected
        assert!(msg.prefix.is_some());
    }

    #[test]
    fn test_ipv6_host() {
        // IPv6 in host
        let raw = ":nick!user@2001:db8::1 PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse IPv6 host");
        assert_eq!(msg.source_nickname(), Some("nick"));
    }

    #[test]
    fn test_cloaked_host() {
        // Cloaked/masked hosts
        let raw = ":nick!user@user/nick/cloaked PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse cloaked host");
        assert_eq!(msg.source_host(), Some("user/nick/cloaked"));
    }
}

// =============================================================================
// CHANNEL NAMES (RFC 2812 Section 1.3)
// =============================================================================

mod channel_names {
    use super::*;

    #[test]
    fn test_standard_channel() {
        let raw = "JOIN #channel";
        let msg: Message = raw.parse().expect("Should parse");
        match msg.command {
            Command::JOIN(ch, _, _) => assert_eq!(ch, "#channel"),
            _ => panic!("Expected JOIN"),
        }
    }

    #[test]
    fn test_local_channel() {
        // & prefix is local channel
        let raw = "JOIN &localchan";
        let msg: Message = raw.parse().expect("Should parse");
        match msg.command {
            Command::JOIN(ch, _, _) => assert_eq!(ch, "&localchan"),
            _ => panic!("Expected JOIN"),
        }
    }

    #[test]
    fn test_channel_with_special_chars() {
        // Channels can contain special characters (except space, bell, comma)
        let raw = "JOIN #foo-bar_baz";
        let msg: Message = raw.parse().expect("Should parse");
        match msg.command {
            Command::JOIN(ch, _, _) => assert_eq!(ch, "#foo-bar_baz"),
            _ => panic!("Expected JOIN"),
        }
    }

    #[test]
    fn test_multiple_channels_join() {
        let raw = "JOIN #chan1,#chan2,#chan3";
        let msg: Message = raw.parse().expect("Should parse");
        match msg.command {
            Command::JOIN(ch, _, _) => assert_eq!(ch, "#chan1,#chan2,#chan3"),
            _ => panic!("Expected JOIN"),
        }
    }
}

// =============================================================================
// UTF-8 HANDLING (IRCv3 implies UTF-8)
// =============================================================================

mod utf8_handling {
    use super::*;

    #[test]
    fn test_utf8_in_message() {
        let raw = ":nick PRIVMSG #ch :Hello 世界 🌍";
        let msg = MessageRef::parse(raw).expect("Should parse UTF-8");
        assert_eq!(msg.arg(1), Some("Hello 世界 🌍"));
    }

    #[test]
    fn test_utf8_in_nick() {
        // Some servers allow UTF-8 nicks
        let raw = ":Ñoño!user@host PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse UTF-8 nick");
        assert_eq!(msg.source_nickname(), Some("Ñoño"));
    }

    #[test]
    fn test_utf8_in_tag_value() {
        let raw = "@label=föö :nick PRIVMSG #ch :hi";
        let msg = MessageRef::parse(raw).expect("Should parse UTF-8 in tag");
        assert_eq!(msg.tag_value("label"), Some("föö"));
    }

    #[test]
    fn test_emoji_in_message() {
        let raw = ":nick PRIVMSG #ch :🎉🎊🎈";
        let msg = MessageRef::parse(raw).expect("Should parse emoji");
        assert_eq!(msg.arg(1), Some("🎉🎊🎈"));
    }
}

// =============================================================================
// ROUND-TRIP COMPLIANCE
// =============================================================================

mod roundtrip {
    use super::*;

    fn assert_roundtrip(raw: &str) {
        let msg: Message = raw.parse().expect("Should parse");
        let serialized = msg.to_string();
        let reparsed: Message = serialized.parse().expect("Should reparse");
        assert_eq!(msg, reparsed, "Roundtrip failed for: {}", raw);
    }

    #[test]
    fn test_roundtrip_simple() {
        assert_roundtrip("PING :server");
    }

    #[test]
    fn test_roundtrip_with_prefix() {
        assert_roundtrip(":nick!user@host PRIVMSG #channel :Hello world");
    }

    #[test]
    fn test_roundtrip_with_tags() {
        assert_roundtrip("@time=2023-01-01T00:00:00Z;msgid=abc :nick PRIVMSG #ch :Tagged");
    }

    #[test]
    fn test_roundtrip_empty_trailing() {
        assert_roundtrip("PRIVMSG #channel :");
    }

    #[test]
    fn test_roundtrip_numeric() {
        assert_roundtrip(":server 001 nick :Welcome to the network");
    }

    #[test]
    fn test_roundtrip_with_escaped_tags() {
        // This tests that tags with special characters survive roundtrip
        let original = Message {
            tags: Some(vec![slirc_proto::Tag::new("key", Some("value;with;semicolons".to_string()))]),
            prefix: None,
            command: Command::PING("test".to_string(), None),
        };

        let serialized = original.to_string();
        let reparsed: Message = serialized.parse().expect("Should reparse");
        assert_eq!(original, reparsed);
        assert_eq!(reparsed.tag_value("key"), Some("value;with;semicolons"));
    }
}

// =============================================================================
// COMMAND-SPECIFIC TESTS
// =============================================================================

mod commands {
    use super::*;

    #[test]
    fn test_privmsg_requires_target_and_text() {
        let msg: Message = "PRIVMSG #channel :Hello".parse().unwrap();
        match msg.command {
            Command::PRIVMSG(target, text) => {
                assert_eq!(target, "#channel");
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected PRIVMSG"),
        }
    }

    #[test]
    fn test_notice_similar_to_privmsg() {
        let msg: Message = "NOTICE #channel :Hello".parse().unwrap();
        match msg.command {
            Command::NOTICE(target, text) => {
                assert_eq!(target, "#channel");
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected NOTICE"),
        }
    }

    #[test]
    fn test_join_with_key() {
        let msg: Message = "JOIN #channel secretkey".parse().unwrap();
        match msg.command {
            Command::JOIN(chan, key, _) => {
                assert_eq!(chan, "#channel");
                assert_eq!(key, Some("secretkey".to_string()));
            }
            _ => panic!("Expected JOIN"),
        }
    }

    #[test]
    fn test_part_with_message() {
        let msg: Message = "PART #channel :Goodbye!".parse().unwrap();
        match msg.command {
            Command::PART(chan, reason) => {
                assert_eq!(chan, "#channel");
                assert_eq!(reason, Some("Goodbye!".to_string()));
            }
            _ => panic!("Expected PART"),
        }
    }

    #[test]
    fn test_quit_with_message() {
        let msg: Message = "QUIT :Gone fishing".parse().unwrap();
        match msg.command {
            Command::QUIT(reason) => {
                assert_eq!(reason, Some("Gone fishing".to_string()));
            }
            _ => panic!("Expected QUIT"),
        }
    }

    #[test]
    fn test_mode_channel() {
        let msg: Message = "MODE #channel +o nick".parse().unwrap();
        // Verify it parses - channel MODE uses ChannelMODE variant
        assert!(matches!(msg.command, Command::ChannelMODE(_, _)));
    }

    #[test]
    fn test_kick_with_reason() {
        let msg: Message = "KICK #channel nick :Bad behavior".parse().unwrap();
        match msg.command {
            Command::KICK(chan, target, reason) => {
                assert_eq!(chan, "#channel");
                assert_eq!(target, "nick");
                assert_eq!(reason, Some("Bad behavior".to_string()));
            }
            _ => panic!("Expected KICK"),
        }
    }
}

// =============================================================================
// EDGE CASES AND ERROR HANDLING
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_message_fails() {
        let result = MessageRef::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_fails() {
        let result = MessageRef::parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_consecutive_spaces() {
        // Extra spaces between parts should be handled
        let raw = ":nick  PRIVMSG  #ch  :hello";
        // This might fail strict parsing but should not panic
        let _ = MessageRef::parse(raw);
    }

    #[test]
    fn test_very_long_nick() {
        // Extremely long nickname (non-compliant but shouldn't crash)
        let long_nick = "a".repeat(100);
        let raw = format!(":{}!user@host PRIVMSG #ch :hi", long_nick);
        let msg = MessageRef::parse(&raw).expect("Should handle long nick");
        assert_eq!(msg.source_nickname(), Some(long_nick.as_str()));
    }

    #[test]
    fn test_trailing_only_colon() {
        // Message with just a colon as trailing should work
        let raw = "PRIVMSG #ch ::";
        let msg = MessageRef::parse(raw).expect("Should parse");
        assert_eq!(msg.arg(1), Some(":"));
    }
}

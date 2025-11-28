//! Integration tests for message parsing and serialization
//!
//! These tests verify that messages can be parsed from strings and then
//! serialized back to equivalent strings, ensuring round-trip compatibility.

use slirc_proto::{Command, Message, Prefix, Tag};

#[test]
fn test_message_round_trip_simple() {
    let original = "PING :irc.example.com";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    // Parse the serialized version back
    let reparsed: Message = serialized.parse().expect("Failed to reparse message");

    // Should be equivalent
    assert_eq!(message, reparsed);
}

#[test]
fn test_message_round_trip_with_prefix() {
    let original = ":nick!user@host PRIVMSG #channel :Hello, world!";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_message_round_trip_with_tags() {
    let original = "@time=2023-01-01T00:00:00.000Z;msgid=abc123 :nick!user@host PRIVMSG #channel :Tagged message";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_message_round_trip_numeric_response() {
    let original = ":server 001 nickname :Welcome to the IRC Network";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_message_round_trip_complex_tags() {
    let original = "@batch=abc123;msgid=def456;time=2023-01-01T12:00:00Z;+custom=value :nick BATCH +abc123 chathistory #channel";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_message_construction_and_parsing() {
    // Construct a message programmatically
    let message = Message {
        tags: Some(vec![
            Tag("time".into(), Some("2023-01-01T00:00:00Z".into())),
            Tag("msgid".into(), Some("test123".into())),
        ]),
        prefix: Some(Prefix::new_from_str("testbot!test@example.com")),
        command: Command::PRIVMSG("#test".to_string(), "Integration test message".to_string()),
    };

    // Serialize to string
    let serialized = message.to_string();

    // Parse back
    let parsed: Message = serialized
        .parse()
        .expect("Failed to parse constructed message");

    // Should be equivalent
    assert_eq!(message, parsed);
}

#[test]
fn test_empty_trailing_parameter() {
    let original = "PRIVMSG #channel :";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);

    // Verify the empty trailing parameter is preserved
    match &reparsed.command {
        Command::PRIVMSG(_, text) => assert_eq!(text, ""),
        _ => panic!("Expected PRIVMSG command"),
    }
}

#[test]
fn test_special_characters_in_message() {
    let original = ":nick!user@host PRIVMSG #channel :Message with üñíçødé and émøjí 🎉";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_mode_command_round_trip() {
    let original = ":server MODE #channel +o nick";
    let message: Message = original.parse().expect("Failed to parse message");
    let serialized = message.to_string();

    let reparsed: Message = serialized.parse().expect("Failed to reparse message");
    assert_eq!(message, reparsed);
}

#[test]
fn test_join_command_variations() {
    let test_cases = vec![
        "JOIN #channel",
        "JOIN #channel key",
        ":nick!user@host JOIN #channel",
        "JOIN #channel1,#channel2 key1,key2",
    ];

    for original in test_cases {
        let message: Message = original
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", original, e));
        let serialized = message.to_string();

        let reparsed: Message = serialized
            .parse()
            .unwrap_or_else(|e| panic!("Failed to reparse '{}': {}", serialized, e));
        assert_eq!(message, reparsed, "Round-trip failed for '{}'", original);
    }
}

#[test]
fn test_batch_messages() {
    let test_cases = vec![
        "BATCH +abc123 chathistory #channel",
        "BATCH -abc123",
        "@batch=abc123 :server PRIVMSG #channel :Batched message",
    ];

    for original in test_cases {
        let message: Message = original
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", original, e));
        let serialized = message.to_string();

        let reparsed: Message = serialized
            .parse()
            .unwrap_or_else(|e| panic!("Failed to reparse '{}': {}", serialized, e));
        assert_eq!(message, reparsed, "Round-trip failed for '{}'", original);
    }
}

#[test]
fn test_operator_ban_commands_round_trip() {
    let test_cases = vec![
        // KLINE with time
        "KLINE 60 *@badhost.com :Spamming",
        // KLINE without time
        "KLINE user@host.com :No reason given",
        // DLINE with time
        "DLINE 3600 192.168.1.0/24 :Network abuse",
        // DLINE without time
        "DLINE 10.0.0.1 :Suspicious activity",
        // UNKLINE
        "UNKLINE user@host.com",
        // UNDLINE
        "UNDLINE 192.168.1.0/24",
    ];

    for original in test_cases {
        let message: Message = original
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", original, e));
        let serialized = message.to_string();

        let reparsed: Message = serialized
            .parse()
            .unwrap_or_else(|e| panic!("Failed to reparse '{}': {}", serialized, e));
        assert_eq!(message, reparsed, "Round-trip failed for '{}'", original);
    }
}

#[test]
fn test_knock_command_round_trip() {
    let test_cases = vec![
        // KNOCK without message
        "KNOCK #channel",
        // KNOCK with message
        "KNOCK #secretroom :Please let me in!",
    ];

    for original in test_cases {
        let message: Message = original
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", original, e));
        let serialized = message.to_string();

        let reparsed: Message = serialized
            .parse()
            .unwrap_or_else(|e| panic!("Failed to reparse '{}': {}", serialized, e));
        assert_eq!(message, reparsed, "Round-trip failed for '{}'", original);
    }
}

#[test]
fn test_operator_commands_typed_variants() {
    use slirc_proto::Command;

    // Verify KLINE with time
    let msg: Message = "KLINE 60 user@host :reason".parse().unwrap();
    match msg.command {
        Command::KLINE(Some(time), mask, reason) => {
            assert_eq!(time, "60");
            assert_eq!(mask, "user@host");
            assert_eq!(reason, "reason");
        }
        other => panic!("Expected KLINE with time, got {:?}", other),
    }

    // Verify KLINE without time
    let msg: Message = "KLINE user@host :reason".parse().unwrap();
    match msg.command {
        Command::KLINE(None, mask, reason) => {
            assert_eq!(mask, "user@host");
            assert_eq!(reason, "reason");
        }
        other => panic!("Expected KLINE without time, got {:?}", other),
    }

    // Verify DLINE with time
    let msg: Message = "DLINE 3600 192.168.0.1 :banned".parse().unwrap();
    match msg.command {
        Command::DLINE(Some(time), host, reason) => {
            assert_eq!(time, "3600");
            assert_eq!(host, "192.168.0.1");
            assert_eq!(reason, "banned");
        }
        other => panic!("Expected DLINE with time, got {:?}", other),
    }

    // Verify UNKLINE
    let msg: Message = "UNKLINE user@host".parse().unwrap();
    match msg.command {
        Command::UNKLINE(mask) => assert_eq!(mask, "user@host"),
        other => panic!("Expected UNKLINE, got {:?}", other),
    }

    // Verify UNDLINE
    let msg: Message = "UNDLINE 10.0.0.0/8".parse().unwrap();
    match msg.command {
        Command::UNDLINE(host) => assert_eq!(host, "10.0.0.0/8"),
        other => panic!("Expected UNDLINE, got {:?}", other),
    }

    // Verify KNOCK with message
    let msg: Message = "KNOCK #channel :let me in".parse().unwrap();
    match msg.command {
        Command::KNOCK(channel, Some(message)) => {
            assert_eq!(channel, "#channel");
            assert_eq!(message, "let me in");
        }
        other => panic!("Expected KNOCK with message, got {:?}", other),
    }

    // Verify KNOCK without message
    let msg: Message = "KNOCK #channel".parse().unwrap();
    match msg.command {
        Command::KNOCK(channel, None) => {
            assert_eq!(channel, "#channel");
        }
        other => panic!("Expected KNOCK without message, got {:?}", other),
    }
}

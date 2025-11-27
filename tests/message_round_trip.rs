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

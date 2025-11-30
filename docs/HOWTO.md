# slirc-proto: The Definitive Guide

This guide provides a deep dive into using `slirc-proto`, a high-performance Rust library for parsing and serializing IRC protocol messages with full IRCv3 support.

## Table of Contents

1. [Core Concepts](#1-core-concepts)
2. [Parsing Messages](#2-parsing-messages)
3. [Working with Commands](#3-working-with-commands)
4. [Constructing & Serializing](#4-constructing--serializing)
5. [Handling Modes](#5-handling-modes)
6. [Transport & Async](#6-transport--async-tokio)
7. [Zero-Copy Transport](#7-zero-copy-transport-high-performance)
8. [IRCv3 Capabilities](#8-ircv3-capabilities)
9. [SASL Authentication](#9-sasl-authentication)
10. [CTCP Messages](#10-ctcp-messages)
11. [ISUPPORT Parsing](#11-isupport-parsing)
12. [Compliance Checking](#12-compliance-checking)
13. [Best Practices](#13-best-practices)
14. [Feature Flags](#14-feature-flags)

---

## 1. Core Concepts

`slirc-proto` is designed around a few key types:

| Type | Description | Use Case |
|------|-------------|----------|
| `Message` | Owned, heap-allocated IRC message | Store messages, pass between threads |
| `MessageRef<'a>` | Zero-copy, borrowed view | High-performance parsing in hot loops |
| `Command` | Strongly-typed enum for all IRC commands | Type-safe command handling |
| `Prefix` | Source of a message (`nick!user@host`) | Identify message origin |
| `Tag` | IRCv3 message tags (key-value pairs) | Metadata like timestamps, msgid |

### Choosing Between `Message` and `MessageRef`

```rust
// Use Message when:
// - You need to store the message
// - You're passing messages between threads
// - You need to modify the message
let msg: Message = raw.parse()?;

// Use MessageRef when:
// - Processing messages in a hot loop
// - You only need to inspect message contents
// - Performance is critical
let msg_ref = MessageRef::parse(raw)?;
```

---

## 2. Parsing Messages

### Owned Parsing (Easiest)

Use this when you need to keep the message around or pass it between threads.

```rust
use slirc_proto::{Message, Command};

let raw = "@time=12345 :nick!user@host PRIVMSG #channel :Hello world!";
let msg: Message = raw.parse().expect("Failed to parse");

if let Command::PRIVMSG(target, text) = msg.command {
    println!("{} says to {}: {}", msg.source_nickname().unwrap(), target, text);
}
```

### Zero-Copy Parsing (Fastest)

Use this in hot loops (like a server) where you process and discard messages immediately.

```rust
use slirc_proto::MessageRef;

let raw = "@time=12345 :nick!user@host PRIVMSG #channel :Hello world!";
let msg = MessageRef::parse(raw).expect("Failed to parse");

// Access without allocation
assert_eq!(msg.command_name(), "PRIVMSG");
assert_eq!(msg.source_nickname(), Some("nick"));
assert_eq!(msg.arg(0), Some("#channel"));
assert_eq!(msg.arg(1), Some("Hello world!"));
```

### Accessing Tags

```rust
// Direct access by key
if let Some(time) = msg.tag_value("time") {
    println!("Server time: {}", time);
}

// Check if tag exists
if msg.has_tag("msgid") {
    println!("Message has an ID");
}

// Iterate all tags
for (key, value) in msg.tags_iter() {
    println!("Tag: {}={}", key, value);
}
```

---

## 3. Working with Commands

The `Command` enum is the heart of the library. It covers standard RFC 1459/2812 commands, IRCv3 extensions, and common server-side operations.

### Pattern Matching

```rust
use slirc_proto::Command;

match msg.command {
    Command::PRIVMSG(target, text) => {
        println!("Message to {}: {}", target, text);
    }
    Command::JOIN(channel, key, _) => {
        println!("Joining {} with key {:?}", channel, key);
    }
    Command::KICK(channel, user, reason) => {
        println!("Kicked {} from {}: {:?}", user, channel, reason);
    }
    Command::Response(code, args) => {
        println!("Numeric {}: {:?}", code.code(), args);
    }
    Command::Raw(cmd, args) => {
        // Fallback for unknown commands
        println!("Unknown command: {} {:?}", cmd, args);
    }
    _ => {}
}
```

### Operator Commands

We support typed variants for administrative actions:

```rust
match msg.command {
    // K-Line with optional duration
    Command::KLINE(Some(duration), mask, reason) => {
        println!("Banning {} for {} seconds: {}", mask, duration, reason);
    }
    Command::KLINE(None, mask, reason) => {
        println!("Permanent ban on {}: {}", mask, reason);
    }
    
    // D-Line (IP bans)
    Command::DLINE(duration, host, reason) => { /* ... */ }
    
    // Remove bans
    Command::UNKLINE(mask) => { /* ... */ }
    Command::UNDLINE(host) => { /* ... */ }
    
    // Channel management
    Command::KNOCK(channel, message) => { /* ... */ }
    Command::CHGHOST(nick, host) => { /* ... */ }
    
    _ => {}
}
```

---

## 4. Constructing & Serializing

### Using Builder Pattern (Recommended)

```rust
use slirc_proto::{Message, Prefix};

let msg = Message::privmsg("#rust", "Hello!")
    .with_prefix(Prefix::new_from_str("mybot!bot@example.com"))
    .with_tag("time", Some("2023-11-28T12:00:00Z"))
    .with_tag("msgid", Some("abc123"));

println!("{}", msg); // Serializes to wire format with CRLF
```

### Convenience Constructors

```rust
use slirc_proto::Message;

// Channel operations
let join = Message::join("#channel");
let join_key = Message::join_with_key("#secret", "password");
let part = Message::part("#channel");
let part_msg = Message::part_with_message("#channel", "Goodbye!");

// Messaging
let privmsg = Message::privmsg("#channel", "Hello!");
let notice = Message::notice("nick", "Alert!");

// User operations
let nick = Message::nick("newnick");
let quit = Message::quit_with_message("Going offline");
let away = Message::away_with_message("AFK");

// Server commands
let ping = Message::ping("server.name");
let pong = Message::pong("server.name");
```

### Manual Construction

```rust
use slirc_proto::{Message, Command};

let msg = Message {
    tags: None,
    prefix: None,
    command: Command::KICK(
        "#channel".to_string(),
        "baduser".to_string(),
        Some("Spamming".to_string()),
    ),
};
```

---

## 5. Handling Modes

Modes are complex because they can be user modes (`+i`) or channel modes (`+o nick`). `slirc-proto` provides typed handling.

### The `Mode<T>` Type

```rust
use slirc_proto::{Mode, ChannelMode, UserMode, Command};

// Channel mode changes
let modes = vec![
    Mode::Plus(ChannelMode::Op, Some("nick".to_string())),      // +o nick
    Mode::Minus(ChannelMode::Secret, None),                      // -s
    Mode::Plus(ChannelMode::Ban, Some("*!*@bad.host".to_string())), // +b mask
];

let cmd = Command::ChannelMODE("#channel".to_string(), modes);

// User mode changes  
let user_modes = vec![
    Mode::Plus(UserMode::Invisible, None),  // +i
    Mode::Minus(UserMode::Wallops, None),   // -w
];

let cmd = Command::UserMODE("mynick".to_string(), user_modes);
```

### Parsing Modes from Strings

```rust
use slirc_proto::{Mode, ChannelMode};

// Parse mode string
let raw = "MODE #channel +ov nick1 nick2";
let msg: Message = raw.parse()?;

if let Command::ChannelMODE(channel, modes) = msg.command {
    for mode in modes {
        match mode {
            Mode::Plus(ChannelMode::Op, Some(nick)) => {
                println!("Opped {} in {}", nick, channel);
            }
            Mode::Plus(ChannelMode::Voice, Some(nick)) => {
                println!("Voiced {} in {}", nick, channel);
            }
            _ => {}
        }
    }
}
```

---

## 6. Transport & Async (Tokio)

The `tokio` feature (enabled by default) provides async transport utilities.

### Basic Transport

```rust
use tokio::net::TcpStream;
use slirc_proto::{Transport, Message, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("irc.libera.chat:6667").await?;
    let mut transport = Transport::tcp(stream)?;

    // Send messages
    transport.write_message(&Message::nick("mybot")).await?;
    transport.write_message(&Message::user("mybot", "My Bot")).await?;

    // Read messages
    while let Ok(Some(message)) = transport.read_message().await {
        println!("← {}", message);
        
        // Handle PING/PONG
        if let Command::PING(server, _) = &message.command {
            transport.write_message(&Message::pong(server)).await?;
        }
    }

    Ok(())
}
```

### TLS Transport (Client-Side)

```rust
use slirc_proto::transport::Transport;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

// Establish TLS connection first, then wrap with Transport
let stream = TcpStream::connect("irc.libera.chat:6697").await?;
let connector: TlsConnector = /* configure rustls */;
let tls_stream = connector.connect(server_name, stream).await?;
let transport = Transport::client_tls(tls_stream)?;  // client_tls for outgoing connections
```

### Using IrcCodec with Framed

```rust
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use slirc_proto::IrcCodec;
use slirc_proto::Message;

let stream = TcpStream::connect("irc.example.com:6667").await?;
let mut framed = Framed::new(stream, IrcCodec::new("utf-8")?);

// Send
framed.send(Message::nick("mybot")).await?;

// Receive
while let Some(Ok(msg)) = framed.next().await {
    println!("Received: {}", msg);
}
```

---

## 7. Zero-Copy Transport (High Performance)

For high-throughput servers, upgrade from `Transport` to `ZeroCopyTransport` after handshake.

```rust
use slirc_proto::{Transport, ZeroCopyTransportEnum, MessageRef};

// Phase 1: Use Transport for handshake
let mut transport = Transport::tcp(stream)?;
// ... perform CAP negotiation, SASL auth, etc ...

// Phase 2: Upgrade to zero-copy for the hot loop
let mut zc: ZeroCopyTransportEnum = transport.into();

while let Some(result) = zc.next().await {
    let msg_ref: MessageRef<'_> = result?;
    
    // Zero-allocation access
    match msg_ref.command_name() {
        "PING" => {
            // Note: You'll need to allocate for the response
            let pong = format!("PONG :{}\r\n", msg_ref.arg(0).unwrap_or(""));
            // ... send pong ...
        }
        "PRIVMSG" => {
            let target = msg_ref.arg(0).unwrap_or("");
            let text = msg_ref.arg(1).unwrap_or("");
            // Process without heap allocation
        }
        _ => {}
    }
}
```

### When to Use Zero-Copy

| Scenario | Recommended |
|----------|-------------|
| IRC server handling 1000+ connections | ✅ ZeroCopyTransport |
| High-throughput bridge/relay | ✅ ZeroCopyTransport |
| Simple bot with few connections | ❌ Use regular Transport |
| Need to store/queue messages | ❌ Use regular Transport |

---

## 8. IRCv3 Capabilities

The library has first-class support for `CAP` negotiation.

### Capability Negotiation

```rust
use slirc_proto::{Command, CapSubCommand};

// Request capabilities
let cap_req = Command::CAP(
    None, 
    CapSubCommand::REQ, 
    None, 
    Some("server-time message-tags sasl".to_string())
);

// End capability negotiation
let cap_end = Command::CAP(None, CapSubCommand::END, None, None);

// Handle CAP responses
match msg.command {
    Command::CAP(_, CapSubCommand::ACK, _, Some(caps)) => {
        println!("Server acknowledged: {}", caps);
    }
    Command::CAP(_, CapSubCommand::NAK, _, Some(caps)) => {
        println!("Server rejected: {}", caps);
    }
    Command::CAP(_, CapSubCommand::LS, _, Some(caps)) => {
        println!("Server supports: {}", caps);
    }
    _ => {}
}
```

### Working with Message Tags

```rust
use slirc_proto::Message;

// Create a message with tags
let msg = Message::privmsg("#channel", "Hello!")
    .with_tag("time", Some("2023-11-28T12:00:00.000Z"))
    .with_tag("msgid", Some("abc123"))
    .with_tag("+typing", None);  // Client-only tag (no value)

// Access tags
if let Some(time) = msg.tag_value("time") {
    println!("Message sent at: {}", time);
}
```

---

## 9. SASL Authentication

### PLAIN Authentication

```rust
use slirc_proto::sasl::encode_plain;
use slirc_proto::Command;

// Start SASL negotiation
let authenticate = Command::AUTHENTICATE("PLAIN".to_string());

// Encode credentials (returns base64)
let encoded = encode_plain("username", "password");

// Send the encoded response
// Note: IRC limits SASL responses to 400 bytes per message.
// For most passwords, this is sufficient.
let auth_msg = Command::AUTHENTICATE(encoded);
// Send auth_msg...
```

### EXTERNAL Authentication (Client Certificates)

```rust
use slirc_proto::sasl::encode_external;

// For client cert auth, usually empty or just authzid
let encoded = encode_external(None);
```

### Handling SASL Responses

```rust
use slirc_proto::Command;

match msg.command {
    Command::Response(code, _) => match code.code() {
        900 => println!("Logged in!"),
        903 => println!("SASL successful!"),
        904 => println!("SASL failed"),
        905 => println!("SASL too long"),
        906 => println!("SASL aborted"),
        907 => println!("Already authenticated"),
        908 => println!("Available mechanisms listed"),
        _ => {}
    },
    _ => {}
}
```

---

## 10. CTCP Messages

### Parsing CTCP

```rust
use slirc_proto::ctcp::{Ctcp, CtcpKind};

// Parse from a PRIVMSG text
let text = "\x01VERSION\x01";
if let Some(ctcp) = Ctcp::parse(text) {
    match ctcp.kind {
        CtcpKind::Version => {
            // Respond with version info
        }
        CtcpKind::Ping => {
            // Echo back the argument
            println!("PING: {:?}", ctcp.argument);
        }
        CtcpKind::Action => {
            println!("* {} {}", nick, ctcp.argument.unwrap_or(""));
        }
        CtcpKind::Time => {
            // Respond with current time
        }
        _ => {}
    }
}
```

### Creating CTCP Messages

```rust
use slirc_proto::ctcp::Ctcp;

// Create an ACTION (/me)
let action = Ctcp::action("waves hello");
let privmsg = Message::privmsg("#channel", &action.to_string());

// Create a VERSION reply
let version = Ctcp::new(CtcpKind::Version, Some("slirc-proto 1.0"));
```

---

## 11. ISUPPORT Parsing

Parse `RPL_ISUPPORT` (005) to understand server capabilities.

### Basic Parsing

```rust
use slirc_proto::isupport::{Isupport, PrefixSpec};

// Parse ISUPPORT tokens from 005 response
let tokens = "NETWORK=Libera.Chat PREFIX=(qaohv)~&@%+ CHANMODES=beI,kLf,lH,psmntirzMQNRTOVKDdGPZSCc";
let isupport = Isupport::parse(tokens);

// Get specific values
if let Some(network) = isupport.get("NETWORK") {
    println!("Connected to: {}", network);
}
```

### Mode Disambiguation with PrefixSpec

Some mode characters (like `q`) have different meanings on different networks:

```rust
use slirc_proto::isupport::PrefixSpec;

// Parse the server's PREFIX token
let spec = PrefixSpec::parse("(qaohv)~&@%+").unwrap();

// Check if 'q' is a prefix mode (founder) or something else (quiet)
if spec.is_prefix_mode('q') {
    println!("'q' is founder mode (~)");
} else {
    println!("'q' is quiet mode (list mode)");
}

// Get prefix symbols for modes
assert_eq!(spec.prefix_for_mode('o'), Some('@'));  // operator
assert_eq!(spec.prefix_for_mode('v'), Some('+'));  // voice
assert_eq!(spec.mode_for_prefix('~'), Some('q'));  // founder
```

---

## 12. Compliance Checking

Validate messages against RFC 1459/2812 specifications.

```rust
use slirc_proto::compliance::{check_compliance, ComplianceConfig, ComplianceError};
use slirc_proto::MessageRef;

let raw = ":nick!user@host PRIVMSG #channel :Hello!";
let msg = MessageRef::parse(raw)?;

let config = ComplianceConfig {
    strict_channel_names: true,
    strict_nicknames: true,
};

match check_compliance(&msg, Some(raw.len()), &config) {
    Ok(_) => println!("Message is RFC compliant"),
    Err(errors) => {
        for err in errors {
            match err {
                ComplianceError::LineTooLong(len) => {
                    println!("Line too long: {} bytes (max 512)", len);
                }
                ComplianceError::InvalidChannelName(name) => {
                    println!("Invalid channel: {}", name);
                }
                ComplianceError::InvalidNickname(nick) => {
                    println!("Invalid nickname: {}", nick);
                }
                _ => println!("Error: {}", err),
            }
        }
    }
}
```

---

## 13. Best Practices

### Performance

1. **Use `MessageRef` for servers**: Avoid allocation overhead in hot loops
2. **Upgrade to `ZeroCopyTransport`**: After handshake, switch for maximum throughput
3. **Batch operations**: Collect multiple writes before flushing

### Safety

1. **Use typed commands**: Prefer `Command::PRIVMSG` over `Command::Raw`
2. **Validate user input**: Sanitize before constructing messages
3. **Check ISUPPORT**: Parse server capabilities before making assumptions about modes

### Code Quality

1. **Handle all command variants**: Use `_ => {}` or `#[non_exhaustive]` patterns
2. **Propagate errors**: Use `?` operator, avoid `unwrap()` in library code
3. **Log appropriately**: Use `tracing` for structured logging

---

## 14. Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `tokio` | ✅ | Async transport (TCP, TLS, WebSocket) via Tokio |
| `encoding` | ❌ | Character encoding support via `encoding_rs` |
| `proptest` | ❌ | Property-based testing utilities |

### Minimal Build (No Async)

```toml
[dependencies]
slirc-proto = { version = "1.2", default-features = false }
```

### With All Features

```toml
[dependencies]
slirc-proto = { version = "1.2", features = ["tokio", "encoding"] }
```

---

## Further Reading

- [README.md](README.md) — Quick start and overview
- [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [API Documentation](https://docs.rs/slirc-proto) — Full API reference
- [`examples/`](examples/) — Working code examples

## Protocol References

- [RFC 1459](https://tools.ietf.org/html/rfc1459) — Internet Relay Chat Protocol
- [RFC 2812](https://tools.ietf.org/html/rfc2812) — IRC Client Protocol
- [IRCv3 Specifications](https://ircv3.net/) — Modern extensions
- [Modern IRC Documentation](https://modern.ircdocs.horse/) — Practical reference

# slirc-proto

> **Straylight IRC Protocol Library**
>
> A robust, zero-copy IRCv3 parsing and serialization library for Rust.

[![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)](https://crates.io/crates/slirc-proto)
[![Documentation](https://docs.rs/slirc-proto/badge.svg)](https://docs.rs/slirc-proto)
[![License](https://img.shields.io/badge/license-Unlicense-blue)](LICENSE)

`slirc-proto` provides a type-safe foundation for IRC applications in Rust. It prioritizes correctness and performance, using strongly typed enums to make invalid IRC states unrepresentable.

## Features

### Zero-Copy Parsing

`MessageRef<'a>` borrows directly from the input buffer, avoiding heap allocations in hot loops:

```rust
use slirc_proto::MessageRef;

let raw = "@time=2023-11-29T12:00:00Z :nick!user@host PRIVMSG #channel :Hello!";
let msg = MessageRef::parse(raw)?;

assert_eq!(msg.command_name(), "PRIVMSG");
assert_eq!(msg.tag_value("time"), Some("2023-11-29T12:00:00Z"));
// No heap allocations!
```

### Type-Safe Commands

The `Command` enum covers RFC 1459/2812 and IRCv3:

```rust
use slirc_proto::{Message, Command};

let msg: Message = ":nick!user@host PRIVMSG #rust :Hello!".parse()?;

match &msg.command {
    Command::PRIVMSG(target, text) => println!("{}: {}", target, text),
    Command::JOIN(channel, _, _) => println!("Joined {}", channel),
    Command::Response(code, args) => println!("Numeric {}", code.code()),
    _ => {}
}
```

### Zero-Copy Encoding

Write directly to sockets without intermediate `String` allocations:

```rust
use slirc_proto::{Message, encode::IrcEncode};

let msg = Message::privmsg("#channel", "Hello!");
let mut buf = Vec::new();
msg.encode(&mut buf)?;  // Writes: "PRIVMSG #channel :Hello!\r\n"
```

### Async Transport (Tokio)

TCP, TLS, and WebSocket support with the `tokio` feature:

```rust
use slirc_proto::transport::{Transport, ZeroCopyTransportEnum};

// Use Transport during handshake
let transport = Transport::tcp(stream);

// Upgrade to zero-copy for the hot loop
let mut zero_copy: ZeroCopyTransportEnum = transport.try_into()?;
while let Some(msg) = zero_copy.next().await {
    let msg_ref = msg?;
    // Process without allocations
}
```

### Sans-IO State Machine

Runtime-agnostic connection handling:

```rust
use slirc_proto::state::{HandshakeMachine, HandshakeConfig};

let config = HandshakeConfig {
    nickname: "bot".to_string(),
    username: "bot".to_string(),
    realname: "My Bot".to_string(),
    password: None,
    request_caps: vec!["multi-prefix".to_string()],
    sasl_credentials: None,
};

let mut machine = HandshakeMachine::new(config);
for action in machine.start() {
    // Send action.message() to server
}
```

## Installation

```toml
[dependencies]
slirc-proto = "1.3"
```

### Feature Flags

| Feature    | Default | Description                           |
| ---------- | ------- | ------------------------------------- |
| `tokio`    | ✅       | Async transport (TCP, TLS, WebSocket) |
| `serde`    | ❌       | Serialize/Deserialize support         |
| `scram`    | ❌       | SCRAM-SHA-256 SASL authentication     |
| `proptest` | ❌       | Property-based testing strategies     |

## Core Types

| Type                   | Description                     | Use Case                |
| ---------------------- | ------------------------------- | ----------------------- |
| `Message`              | Owned IRC message               | Storage, cross-thread   |
| `MessageRef<'a>`       | Zero-copy borrowed message      | Hot loops, parsing      |
| `Command`              | Strongly-typed command enum     | Type-safe handling      |
| `Prefix` / `PrefixRef` | Message source (nick!user@host) | Sender identification   |
| `Tag`                  | IRCv3 message tags              | Timestamps, msgid, etc. |
| `Response`             | Numeric reply codes             | Server responses        |

## Command Coverage

### RFC 1459/2812

- Connection: `PASS`, `NICK`, `USER`, `OPER`, `QUIT`
- Channels: `JOIN`, `PART`, `TOPIC`, `NAMES`, `LIST`, `INVITE`, `KICK`
- Messaging: `PRIVMSG`, `NOTICE`
- Modes: `MODE` (user and channel)
- Queries: `WHO`, `WHOIS`, `WHOWAS`, `MOTD`, `LUSERS`, `VERSION`, `STATS`
- Server: `PING`, `PONG`, `KILL`, `AWAY`, `WALLOPS`

### IRCv3 Extensions

- `CAP` (LS, REQ, ACK, NAK, END, NEW, DEL)
- `AUTHENTICATE` (SASL)
- `BATCH` (with subcommands)
- `CHATHISTORY` (LATEST, BEFORE, AFTER, BETWEEN, AROUND)
- `TAGMSG`
- `MONITOR` (+, -, C, L, S)
- `SETNAME`
- `FAIL`, `WARN`, `NOTE` (standard replies)

### Operator Commands

- `KLINE`, `UNKLINE`, `DLINE`, `UNDLINE`
- `GLINE`, `UNGLINE`, `ZLINE`, `UNZLINE`
- `RLINE`, `UNRLINE`, `SHUN`, `UNSHUN`
- `SAJOIN`, `SAPART`, `SAMODE`, `SANICK`
- `CHGHOST`, `DIE`, `REHASH`, `RESTART`

## IRCv3 Support

### Capabilities

Full capability negotiation (CAP 301 and 302):

```rust
use slirc_proto::caps::{Capability, NegotiationVersion};

// Parse capability list
let caps = Capability::parse_list("multi-prefix userhost-in-names sasl");
```

### SASL Authentication

PLAIN, EXTERNAL, and SCRAM-SHA-256:

```rust
use slirc_proto::sasl::{encode_plain, ScramClient};

// PLAIN
let auth = encode_plain("account", "password");

// SCRAM-SHA-256 (requires `scram` feature)
let client = ScramClient::new("account", "password")?;
```

### Message Tags

First-class support for IRCv3 tags:

```rust
let msg = Message::privmsg("#dev", "Hello")
    .with_tag("time", Some("2023-01-01T12:00:00Z"))
    .with_tag("msgid", Some("abc123"));
```

### CHATHISTORY

```rust
use slirc_proto::{ChatHistorySubCommand, MessageReference};

// Request last 50 messages
let cmd = Command::CHATHISTORY(
    ChatHistorySubCommand::LATEST,
    "#channel".to_string(),
    MessageReference::Timestamp("*".to_string()),
    50,
);
```

## ISUPPORT Parsing

Parse server capabilities from `RPL_ISUPPORT` (005):

```rust
use slirc_proto::Isupport;

let isupport = Isupport::from_message(&msg)?;

assert_eq!(isupport.network(), Some("Libera.Chat"));
assert_eq!(isupport.chantypes(), Some("#&"));

if let Some(prefix) = isupport.prefix_spec() {
    println!("Modes: {:?}, Prefixes: {:?}", prefix.modes, prefix.prefixes);
}
```

## CTCP Handling

Parse and create CTCP messages:

```rust
use slirc_proto::ctcp::{Ctcp, CtcpKind};

// Parse
let ctcp = Ctcp::parse("\x01ACTION waves\x01")?;
assert_eq!(ctcp.kind, CtcpKind::Action);

// Create
let action = Ctcp::action("waves");
println!("{}", action);  // "\x01ACTION waves\x01"
```

## Compliance Checking

Validate messages against RFC specifications:

```rust
use slirc_proto::compliance::{check_compliance, ComplianceConfig};

let config = ComplianceConfig::default();
match check_compliance(&msg, Some(raw.len()), &config) {
    Ok(_) => println!("RFC compliant"),
    Err(errors) => println!("Issues: {:?}", errors),
}
```

## Modes

Type-safe user and channel modes:

```rust
use slirc_proto::mode::{UserMode, ChannelMode, Mode};

// Parse mode changes
let modes = Mode::<ChannelMode>::parse_many("+ov nick1 nick2");

// User modes
let invisible = Mode::Plus(UserMode::Invisible);
let oper = Mode::Plus(UserMode::Oper);
```

## Project Structure

```
slirc-proto/
├── src/
│   ├── lib.rs            # Public API exports
│   ├── message/          # Message and MessageRef types
│   │   ├── borrowed.rs   # Zero-copy MessageRef
│   │   └── types.rs      # Owned Message
│   ├── command/          # Command enum and parsing
│   ├── prefix/           # Prefix and PrefixRef
│   ├── response/         # Numeric reply codes
│   ├── mode/             # User and channel modes
│   ├── encode/           # Zero-copy encoding
│   ├── transport/        # Async I/O (tokio feature)
│   ├── state/            # Sans-IO state machine
│   ├── sasl/             # SASL authentication
│   ├── caps/             # Capability negotiation
│   ├── isupport/         # ISUPPORT parsing
│   ├── ircv3/            # IRCv3 utilities
│   ├── ctcp.rs           # CTCP message handling
│   ├── compliance/       # RFC compliance checking
│   └── casemap.rs        # IRC case mapping
├── examples/             # Usage examples
├── benches/              # Benchmarks
├── fuzz/                 # Fuzz testing
└── docs/
    └── HOWTO.md          # Detailed guide
```

## Examples

See `examples/` for complete working code:

- `simple_client.rs` - Basic IRC client
- `bot.rs` - Simple bot implementation
- `zero_copy_server.rs` - High-performance server pattern
- `sasl_auth.rs` - SASL authentication flow
- `websocket_server.rs` - WebSocket IRC gateway
- `ctcp_handler.rs` - CTCP message handling
- `isupport_parser.rs` - ISUPPORT parsing
- `compliance_check.rs` - RFC compliance validation

## Performance

Key optimizations:

- **Zero-copy parsing**: `MessageRef` borrows from input buffer
- **Zero-copy encoding**: `IrcEncode` writes directly to sockets
- **Minimal allocations**: Hot paths avoid heap allocation
- **Efficient mode handling**: Batch mode parsing and serialization

## License

This project is released into the public domain under the [Unlicense](LICENSE).

## Acknowledgments

Inspired by [Aaron Weiss's irc crate](https://github.com/aatxe/irc).

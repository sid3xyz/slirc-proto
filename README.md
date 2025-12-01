# slirc-proto

> **Straylight IRC Protocol Library**
>
> A robust, zero-copy IRCv3 parsing and serialization library for Rust, designed for high-performance servers, clients, and bots.

[![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)](https://crates.io/crates/slirc-proto)
[![Documentation](https://docs.rs/slirc-proto/badge.svg)](https://docs.rs/slirc-proto)
[![License](https://img.shields.io/badge/license-Unlicense-blue)](LICENSE)

`slirc-proto` provides a type-safe foundation for the Straylight IRC ecosystem. It prioritizes correctness and performance, using strongly typed enums to make invalid IRC states unrepresentable.

## 🌟 Key Features

### 🚀 High Performance
- **Zero-Copy Parsing**: `MessageRef<'a>` borrows directly from the input buffer, avoiding heap allocations in hot loops.
- **Zero-Copy Encoding**: `IrcEncode` trait writes directly to `impl Write` without intermediate `String` allocations.
- **Efficient Serialization**: Optimized handling of mode changes and command parameters.

### 🛡️ Type Safety
- **Typed Commands**: `Command` enum covers standard RFC 1459/2812 commands (`PRIVMSG`, `JOIN`, etc.) and IRCv3 extensions.
- **Typed Modes**: `UserMode` and `ChannelMode` enums (including `+r`, `+S`, etc.).
- **Typed Responses**: `Response` enum for all numeric replies.
- **Builder Pattern**: Ergonomic builders for `Message` and `Prefix`.

### 🔌 Modern IRCv3 Support
- **Capabilities**: Full `CAP LS/REQ/ACK` negotiation flow.
- **Tags**: First-class support for message tags (`@time`, `@account`, `@batch`, etc.).
- **Extensions**: Helpers for `BATCH`, `CHATHISTORY`, `MONITOR`, `SASL` (PLAIN/EXTERNAL/SCRAM-SHA-256).

### 🌐 Flexible Transport
- **Async I/O**: Built on `tokio` for scalable networking.
- **Transports**:
  - **TCP**: Plain-text connections.
  - **TLS**: Server-side and Client-side TLS support.
  - **WebSocket**: Native IRC-over-WebSocket support.
- **Sans-IO State Machine**: `HandshakeMachine` for runtime-agnostic connection handling (New in v1.3.0).

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = "1.3"
```

### Feature Flags

| Feature    | Default | Description                                       |
| ---------- | ------- | ------------------------------------------------- |
| `tokio`    | ✅       | Async transport (TCP, TLS, WebSocket) via Tokio   |
| `serde`    | ❌       | Serialize/Deserialize support for data structures |
| `encoding` | ❌       | Character encoding support via `encoding_rs`      |
| `proptest` | ❌       | Property-based testing strategies                 |

## 📖 Usage Examples

### Zero-Copy Parsing (Recommended for Hot Paths)

```rust
use slirc_proto::MessageRef;

let raw = "@time=2023-11-29T12:00:00Z :nick!user@host PRIVMSG #channel :Hello world!";
let msg = MessageRef::parse(raw).expect("Valid IRC message");

assert_eq!(msg.command_name(), "PRIVMSG");
assert_eq!(msg.tag_value("time"), Some("2023-11-29T12:00:00Z"));
// No heap allocations occurred!
```

### Constructing Messages

```rust
use slirc_proto::{Message, Prefix};

let msg = Message::privmsg("#rust", "Hello from slirc-proto!")
    .with_prefix(Prefix::new("mybot", "bot", "example.com"))
    .with_tag("time", Some("2023-11-29T12:00:00Z"));

println!("{}", msg);
// Output: @time=... :mybot!bot@example.com PRIVMSG #rust :Hello from slirc-proto!
```

### Using the Handshake Machine (Sans-IO)

Manage the complexity of connection registration and capability negotiation without being tied to a specific runtime.

```rust
use slirc_proto::state::{HandshakeMachine, HandshakeConfig, HandshakeAction};

let config = HandshakeConfig {
    nickname: "straylight".to_string(),
    username: "straylight".to_string(),
    realname: "Straylight Bot".to_string(),
    password: None,
};

let mut machine = HandshakeMachine::new(config);

// Get initial actions (CAP LS, NICK, USER)
for action in machine.start() {
    // Send action.to_string() to the server
}

// Process incoming messages
// let msg = MessageRef::parse(input)?;
// for action in machine.feed(&msg) { ... }
```

## 🏗️ Architecture

- **`src/message/`**: Core message types (`Message`, `MessageRef`) and parsing logic.
- **`src/command/`**: Strongly typed `Command` enum and subcommands.
- **`src/transport/`**: Async transport implementations (TCP, TLS, WebSocket).
- **`src/state/`**: Connection state machines.
- **`src/ircv3/`**: IRCv3 extension support.

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

This project is licensed under the [Unlicense](LICENSE).

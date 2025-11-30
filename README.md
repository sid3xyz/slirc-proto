# slirc-proto

> **Straylight IRC Protocol Library**
> A robust, zero-copy IRCv3 parsing and serialization library for Rust.

[![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)](https://crates.io/crates/slirc-proto)
[![Documentation](https://docs.rs/slirc-proto/badge.svg)](https://docs.rs/slirc-proto)
[![License](https://img.shields.io/badge/license-Unlicense-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![Status](https://img.shields.io/badge/status-stable-green)]()

`slirc-proto` is the backbone of the Straylight IRC ecosystem. It provides a high-performance, type-safe foundation for building IRC clients, servers, and bots. It prioritizes correctness, utilizing strongly typed enums for all protocol primitives to make invalid states unrepresentable.

## 🚀 Key Features

### Performance First

* **Zero-Copy Parsing:** `MessageRef<'a>` borrows directly from the input buffer, avoiding heap allocations in hot loops.
* **Zero-Copy Encoding:** `IrcEncode` trait writes directly to `impl Write` without intermediate `String` allocations.
* **Zero-Copy Transport:** Specialized `ZeroCopyTransport` for high-throughput server implementations.
* **Optimized Serialization:** Mode changes and command parameters are serialized efficiently.

### Modern IRCv3 Support

* **Full Capability Negotiation:** `CAP LS/REQ/ACK` flow with version negotiation (301/302).
* **Tags:** First-class support for message tags (`@time`, `@account`, `@batch`).
* **Extensions:** Helpers for `BATCH`, `CHATHISTORY`, `MONITOR`, and `SASL` (PLAIN/EXTERNAL/SCRAM-SHA-256).
* **Sans-IO State Machine:** `HandshakeMachine` for runtime-agnostic connection handling.

### Transport Options

* **TCP:** Plain-text connections via `Transport::tcp()`.
* **TLS (Server):** Server-side TLS via `Transport::tls()` for IRC daemons.
* **TLS (Client):** Client-side TLS via `Transport::client_tls()` for connecting to port 6697.
* **WebSocket:** IRC-over-WebSocket via `Transport::websocket()` and `Transport::websocket_tls()`.

### Developer Experience

* **Typed Commands:** `Command::PRIVMSG`, `Command::JOIN`, etc., instead of error-prone string arrays.
* **Typed Modes:** `UserMode` and `ChannelMode` enums (including `+r` Registered, `+S` Service).
* **Typed Responses:** `Response` enum covering all RFC 2812 numerics with helper methods.
* **Builders:** Ergonomic constructors for Messages and Prefixes.
* **Optional Serde:** Enable `serde` feature for serialization support.

---

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = "1.3"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `tokio` | ✓ | Async transport (TCP, TLS, WebSocket) via Tokio |
| `serde` | | Serialize/Deserialize derives for Message, Command, etc. |
| `encoding` | | Character encoding support via `encoding_rs` |
| `proptest` | | Property-based testing strategies |

---

## ⚡ Quick Start

### 1. Parsing Messages (Zero-Copy)

Ideal for servers and high-frequency bots.

```rust
use slirc_proto::MessageRef;

let raw = "@time=2023-11-29T12:00:00Z :nick!user@host PRIVMSG #channel :Hello world!";
let msg = MessageRef::parse(raw).expect("Valid IRC message");

assert_eq!(msg.command_name(), "PRIVMSG");
assert_eq!(msg.tag_value("time"), Some("2023-11-29T12:00:00Z"));
// No heap allocations occurred up to this point!
```

### 2. Constructing Messages

Using the fluent builder API (v1.2.0+).

```rust
use slirc_proto::{Message, Prefix};

let msg = Message::privmsg("#rust", "Hello from slirc-proto!")
    .with_prefix(Prefix::new("mybot", "bot", "example.com"))
    .with_tag("time", Some("2023-11-29T12:00:00Z"));

println!("{}", msg);
// Output: @time=... :mybot!bot@example.com PRIVMSG #rust :Hello from slirc-proto!
```

### 3. Async Transport

Connect and handshake in just a few lines.

```rust
use slirc_proto::{Transport, Message};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("irc.libera.chat:6667").await?;
    let mut transport = Transport::tcp(stream)?;

    transport.write_message(&Message::nick("slirc_bot")).await?;
    transport.write_message(&Message::user("slirc", "Slirc Bot")).await?;

    while let Ok(Some(msg)) = transport.read_message().await {
        println!("Received: {}", msg);
    }
    Ok(())
}
```

---

## 🧩 Architecture

The library is structured into specialized modules:

### Core Protocol

* **`message`**: `Message` and `MessageRef<'a>` types for owned/borrowed parsing.
* **`command`**: Strongly typed `Command` enum covering RFC 2812 + IRCv3.
* **`prefix`**: `Prefix` and `PrefixRef<'a>` for source (nick!user@host) parsing.
* **`response`**: `Response` enum for all IRC numeric replies (001-999).
* **`mode`**: Type-safe `UserMode` and `ChannelMode` parsing/serialization.
* **`encode`**: `IrcEncode` trait for zero-copy message serialization.
* **`state`**: Sans-IO `HandshakeMachine` for connection lifecycle management.

### IRCv3 Extensions

* **`caps`**: Capability negotiation (`Capability` enum, version 301/302).
* **`ircv3`**: Helpers for `BATCH`, server-time, message IDs.
* **`sasl`**: PLAIN/EXTERNAL/SCRAM-SHA-256 authentication mechanics.
* **`ctcp`**: CTCP message parsing (ACTION, VERSION, PING, etc.).

### Transport (requires `tokio` feature)

* **`transport`**: `Transport` enum with TCP/TLS/WebSocket variants.
* **`transport::ZeroCopyTransport`**: Zero-allocation message streaming.
* **`websocket`**: WebSocket handshake helpers.

### Utilities

* **`isupport`**: Parse `RPL_ISUPPORT` (005) server capability tokens.
* **`casemap`**: IRC-compliant case-insensitive string comparison.
* **`colors`**: IRC color/formatting code helpers via `FormattedStringExt`.
* **`compliance`**: Message validation against protocol rules.
* **`scanner`**: Protocol detection (IRC vs HTTP/TLS/SSH).

---

## 📚 Documentation

* **[API Documentation](https://docs.rs/slirc-proto)** — Full rustdoc reference
* **[Crates.io](https://crates.io/crates/slirc-proto)** — Package registry
* **[GitHub](https://github.com/sid3xyz/slirc-proto)** — Source repository

---

## 📄 License

This project is released into the public domain under [The Unlicense](LICENSE).

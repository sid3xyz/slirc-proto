# slirc-proto

> **Straylight IRC Protocol Library**
> A robust, zero-copy IRCv3 parsing and serialization library for Rust.

![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)
![License](https://img.shields.io/badge/license-Unlicense-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Status](https://img.shields.io/badge/status-stable-green)

`slirc-proto` is the backbone of the Straylight IRC ecosystem. It provides a high-performance, type-safe foundation for building IRC clients, servers, and bots. It prioritizes correctness, utilizing strongly typed enums for all protocol primitives to make invalid states unrepresentable.

## 🚀 Key Features

### Performance First
* **Zero-Copy Parsing:** `MessageRef<'a>` borrows directly from the input buffer, avoiding heap allocations in hot loops.
* **Zero-Copy Transport:** Specialized `ZeroCopyTransport` for high-throughput server implementations.
* **Optimized Serialization:** Mode changes and command parameters are serialized efficiently without intermediate strings.

### Modern IRCv3 Support
* **Full Capability Negotiation:** `CAP LS/REQ/ACK` flow with version negotiation (301/302).
* **Tags:** First-class support for message tags (`@time`, `@account`, `@batch`).
* **Extensions:** Helpers for `BATCH`, `CHATHISTORY`, `MONITOR`, and `SASL` (PLAIN/EXTERNAL).

### Developer Experience
* **Typed Commands:** `Command::PRIVMSG`, `Command::JOIN`, etc., instead of error-prone string arrays.
* **Typed Modes:** `UserMode` and `ChannelMode` enums (including `+r` Registered, `+S` Service).
* **Builders:** Ergonomic constructors for Messages and Prefixes.

---

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = "1.2"
```

#### Feature Flags

* `tokio` (Default): Enables async transport and codecs.
* `encoding`: Adds support for non-UTF8 text encodings.
* `proptest`: Enables property-based testing strategies.

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

* **`command`**: Strongly typed `Command` enum covering RFC 2812 + IRCv3.
* **`mode`**: Type-safe Mode string parsing/serialization.
* **`sasl`**: Mechanics for PLAIN/EXTERNAL authentication.
* **`transport`**: Tokio-based framing (owned) and zero-copy streams.
* **`isupport`**: Helpers for parsing server capability bursts (005).

---

## 📄 License

This project is released into the public domain under [The Unlicense](LICENSE).

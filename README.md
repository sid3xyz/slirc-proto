# slirc-proto

![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)

> **⚠️ AI RESEARCH EXPERIMENT: This software is a proof-of-concept developed using AI agents. It is NEVER production ready. Do not deploy, do not use for any real network.**

`slirc-proto` is a robust, high-performance Rust library for parsing and serializing IRC protocol messages. It provides full support for modern IRCv3 extensions, including message tags, capability negotiation, and SASL.

## Why IRC?

We chose IRC as the test domain because:

- **40+ years of open development** provides rich training data and clear specifications (RFCs 1459, 2812, IRCv3)
- **Goldilocks complexity**: Complex enough to be interesting (async networking, state machines, protocol parsing) but not so complex it requires domain expertise AI lacks
- **Testable compliance**: The `irctest` suite provides objective metrics for code quality
- **No users, no consequences**: We can experiment freely without breaking production systems

## Features

- **Dual API**:
  - **Owned types** (`Message`, `Command`) for ease of use and modification.
  - **Zero-copy types** (`MessageRef`, `CommandRef`) for high-performance parsing without allocations.
- **IRCv3 Support**: Native handling of message tags, capabilities, and batch commands.
- **Async Networking**: Optional `tokio` integration providing `IrcCodec` for seamless framing.
- **Compliance**: Built-in compliance checking against IRC specifications.
- **Extensive Parsing**: Handles complex prefixes, user/channel modes, and `ISUPPORT` parameters.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = "1.3"
```

To enable async networking support with Tokio:

```toml
[dependencies]
slirc-proto = { version = "1.3", features = ["tokio"] }
```

## Usage

### Parsing Messages

You can parse raw IRC lines into strongly-typed structures.

```rust
use slirc_proto::Message;

let raw = "@time=2023-04-12T12:00:00Z :nick!user@host PRIVMSG #channel :Hello world!";
let msg: Message = raw.parse().expect("Failed to parse message");

assert_eq!(msg.command.as_str(), "PRIVMSG");
if let Some(tags) = msg.tags {
    println!("Message time: {:?}", tags.get("time"));
}
```

### Constructing Messages

Build messages fluently using the builder pattern.

```rust
use slirc_proto::{Message, prefix::Prefix};

let msg = Message::privmsg("#rust", "Hello from slirc-proto!")
    .with_tag("intent", Some("greeting"))
    .with_prefix(Prefix::new_from_str("bot!service@example.com"));

println!("{}", msg);
// Output: @intent=greeting :bot!service@example.com PRIVMSG #rust :Hello from slirc-proto!
```

### Zero-Copy Parsing

For performance-critical applications, use `MessageRef` to parse without copying the underlying string data.

```rust
use slirc_proto::MessageRef;

let raw = ":server.example.com 001 user :Welcome";
let msg = MessageRef::parse(raw).expect("Valid message");

// msg.command, msg.prefix, etc. are all &str references to `raw`
assert_eq!(msg.command.as_str(), "001");
```

### Async Codec (Tokio)

With the `tokio` feature enabled, use `IrcCodec` to handle framing over a TCP stream.

```rust,no_run
use futures_util::StreamExt;
use slirc_proto::IrcCodec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("irc.example.com:6667").await?;
    let mut framed = Framed::new(stream, IrcCodec::new());

    while let Some(msg) = framed.next().await {
        match msg {
            Ok(message) => println!("Received: {}", message),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}
```

## License

Unlicense.

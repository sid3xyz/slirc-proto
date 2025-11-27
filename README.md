# slirc-proto

[![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)](https://crates.io/crates/slirc-proto)
[![Documentation](https://docs.rs/slirc-proto/badge.svg)](https://docs.rs/slirc-proto)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

A Rust library for parsing and serializing IRC protocol messages, with full IRCv3 support.

## Part of the STRAYLIGHT Project

This library is a core component of **STRAYLIGHT**, an experimental IRC ecosystem. For more information about the STRAYLIGHT project and its related components, visit [sid3.xyz](https://sid3.xyz).

## Features

- **Message Parsing & Serialization** — Parse and serialize IRC messages with tags, prefixes, commands, and parameters
- **IRCv3 Extensions** — Capability negotiation, message tags, batch processing, server-time, and message IDs
- **Zero-Copy Parsing** — Efficient `MessageRef` and `CommandRef` types for borrowing without allocation
- **Async Transport** — TCP, TLS (via rustls), and WebSocket connections with Tokio
- **SASL Authentication** — PLAIN and EXTERNAL mechanism support with chunked encoding
- **CTCP Handling** — Parse and construct CTCP messages (ACTION, VERSION, PING, etc.)
- **Mode Parsing** — User and channel mode parsing and serialization
- **ISUPPORT** — Parse `RPL_ISUPPORT` (005) server capability tokens
- **Case Mapping** — IRC-compliant case-insensitive string comparison
- **Color Formatting** — IRC color code helpers via `FormattedStringExt`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = "1.0"
```

### Feature Flags

| Feature    | Default | Description                                      |
|------------|---------|--------------------------------------------------|
| `tokio`    | ✓       | Async transport (TCP, TLS, WebSocket) via Tokio  |
| `encoding` |         | Character encoding support via `encoding_rs`     |

To use without async transport:

```toml
[dependencies]
slirc-proto = { version = "1.0", default-features = false }
```

## Usage

### Parsing Messages

```rust
use slirc_proto::Message;

let raw = "@time=2023-01-01T12:00:00Z :nick!user@host PRIVMSG #channel :Hello!";
let message: Message = raw.parse().expect("Valid IRC message");

if let Some(tags) = &message.tags {
    println!("Message has {} tags", tags.len());
}
```

### Creating Messages

Use convenience constructors for common message types:

```rust
use slirc_proto::{Message, prefix::Prefix};

// Simple messages
let privmsg = Message::privmsg("#rust", "Hello, world!");
let notice = Message::notice("nick", "Server notice");
let join = Message::join("#channel");

// With IRCv3 tags and prefix
let tagged = Message::privmsg("#dev", "Tagged message")
    .with_tag("time", Some("2023-01-01T12:00:00Z"))
    .with_tag("msgid", Some("abc123"))
    .with_prefix(Prefix::new_from_str("bot!bot@example.com"));

println!("{}", tagged); // Serializes to IRC wire format
```

### Async Client with Transport

```rust
use slirc_proto::{Command, Message, Transport};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("irc.libera.chat:6667").await?;
    let mut transport = Transport::tcp(stream);

    // Send registration
    transport.write_message(&Message::from(Command::NICK("mybot".into())).to_string()).await?;
    transport.write_message(&Message::from(Command::USER("bot".into(), "0".into(), "My Bot".into())).to_string()).await?;

    // Read messages
    while let Ok(Some(line)) = transport.read_message().await {
        let msg: Message = line.parse()?;
        println!("← {}", msg);
    }

    Ok(())
}
```

### SASL Authentication

```rust
use slirc_proto::sasl::{encode_plain, SaslMechanism, SaslState};

// Encode PLAIN credentials (username + password)
let encoded = encode_plain("myuser", "mypassword");

// Parse mechanism from server
let mech = SaslMechanism::parse("PLAIN");
assert_eq!(mech, SaslMechanism::Plain);
```

### CTCP Messages

```rust
use slirc_proto::ctcp::{Ctcp, CtcpKind};

// Parse CTCP from a PRIVMSG
let ctcp = Ctcp::parse("\x01VERSION\x01");
assert_eq!(ctcp.kind, CtcpKind::Version);

// Create CTCP ACTION
let action = Ctcp::action("waves hello");
```

### Mode Parsing

```rust
use slirc_proto::mode::{Mode, ChannelMode, UserMode};

// Parse channel modes
let modes: Vec<Mode<ChannelMode>> = "+ov nick1 nick2".parse().unwrap();

// Parse user modes
let user_modes: Vec<Mode<UserMode>> = "+iw".parse().unwrap();
```

## Examples

See the [`examples/`](examples/) directory:

- `simple_client.rs` — Basic IRC client connection
- `bot.rs` — Full-featured bot with command handling
- `sasl_auth.rs` — SASL PLAIN authentication flow
- `ctcp_handler.rs` — CTCP VERSION/PING responses

Run an example:

```bash
cargo run --example bot
```

## Protocol References

- [RFC 1459 — Internet Relay Chat Protocol](https://tools.ietf.org/html/rfc1459)
- [RFC 2812 — IRC Client Protocol](https://tools.ietf.org/html/rfc2812)
- [IRCv3 Specifications](https://ircv3.net/)
- [Modern IRC Documentation](https://modern.ircdocs.horse/)

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

This project was inspired by the architectural patterns established by
[Aaron Weiss (aatxe)](https://github.com/aatxe) in the [irc](https://github.com/aatxe/irc) crate.

## Contributing

Contributions are welcome! Please open issues and pull requests on [GitHub](https://github.com/sid3xyz/slirc-proto).

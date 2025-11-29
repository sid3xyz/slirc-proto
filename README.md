# slirc-proto

[![License: Unlicense](https://img.shields.io/badge/License-Unlicense-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![Crates.io](https://img.shields.io/crates/v/slirc-proto.svg)](https://crates.io/crates/slirc-proto)
[![Documentation](https://docs.rs/slirc-proto/badge.svg)](https://docs.rs/slirc-proto)

A high-performance Rust library for parsing and serializing IRC protocol messages with full IRCv3 support.

> **🚀 First Public Release** — This is the first component of the STRAYLIGHT IRC ecosystem to go public. Over the next few weeks, we'll be releasing the full IRCd and related tools as we clean things up and make them professionally presentable — at least enough for the Gonzo Admin.

## Part of the STRAYLIGHT Project

This library is a core component of **STRAYLIGHT**, an experimental IRC ecosystem. For more information about the STRAYLIGHT project and its related components, visit [sid3.xyz](https://sid3.xyz).

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Examples](#examples)
- [Protocol References](#protocol-references)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Message Parsing & Serialization** — Parse and serialize IRC messages with tags, prefixes, commands, and parameters
- **IRCv3 Extensions** — Capability negotiation, message tags, batch processing, server-time, and message IDs
- **Zero-Copy Parsing** — Efficient `MessageRef` and `CommandRef` types for borrowing without allocation
- **Zero-Copy Transport** — `ZeroCopyTransport` yields `MessageRef<'_>` with zero heap allocations in hot loops
- **Async Transport** — TCP, TLS (server and client), and WebSocket connections with Tokio
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
use slirc_proto::transport::Transport;
use slirc_proto::Message;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("irc.libera.chat:6667").await?;
    let mut transport = Transport::tcp(stream)?;

    // Send registration
    transport.write_message(&Message::nick("mybot")).await?;
    transport.write_message(&Message::user("bot", "My Bot")).await?;

    // Read messages
    while let Ok(Some(msg)) = transport.read_message().await {
        println!("← {}", msg);
    }

    Ok(())
}
```

### TLS Client Connection

For secure connections to IRC servers (port 6697):

```rust
use slirc_proto::transport::Transport;
use slirc_proto::Message;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up TLS with system root certificates
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    
    // Connect with TLS
    let tcp_stream = TcpStream::connect("irc.libera.chat:6697").await?;
    let server_name = "irc.libera.chat".try_into()?;
    let tls_stream = connector.connect(server_name, tcp_stream).await?;
    
    // Create client TLS transport
    let mut transport = Transport::client_tls(tls_stream)?;
    
    // Use like any other transport
    transport.write_message(&Message::nick("mybot")).await?;
    transport.write_message(&Message::user("bot", "My Bot")).await?;
    
    while let Ok(Some(msg)) = transport.read_message().await {
        println!("← {}", msg);
    }
    
    Ok(())
}
```

### Zero-Copy Transport (High Performance)

For high-throughput servers, upgrade from `Transport` to `ZeroCopyTransport` after handshake:

```rust
use slirc_proto::transport::{Transport, ZeroCopyTransportEnum};

// Use Transport for handshake
let transport = Transport::tcp(stream)?;
// ... perform CAP negotiation ...

// Upgrade to zero-copy for the hot loop (no allocations!)
let mut zc: ZeroCopyTransportEnum = transport.try_into().expect("TCP/TLS only");
while let Some(result) = zc.next().await {
    let msg_ref = result?;  // MessageRef<'_> borrows from buffer
    println!("Command: {}", msg_ref.command_name());
    // Process without allocating
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

// Parse channel modes from command arguments
let modes = Mode::as_channel_modes(&["+ov", "nick1", "nick2"]).unwrap();

// Parse user modes
let user_modes = Mode::as_user_modes(&["+iw"]).unwrap();
```

### ISUPPORT and Mode Disambiguation

Some mode characters (like `q`) have different meanings on different IRC networks.
Use `PrefixSpec` from ISUPPORT to determine server-specific mode semantics:

```rust
use slirc_proto::isupport::PrefixSpec;

// Parse the server's PREFIX token (from RPL_ISUPPORT 005)
let spec = PrefixSpec::parse("(qaohv)~&@%+").unwrap();

// Check if 'q' is a prefix mode (founder) or something else (quiet)
if spec.is_prefix_mode('q') {
    println!("This server uses 'q' for founder (~)");
} else {
    println!("This server uses 'q' for quiet (list mode)");
}

// Get prefix symbols for modes
assert_eq!(spec.prefix_for_mode('o'), Some('@'));  // operator
assert_eq!(spec.mode_for_prefix('~'), Some('q'));  // founder
```

## Examples

See the [`examples/`](examples/) directory for practical examples of common IRC tasks:

| Example | Description |
|---------|-------------|
| [`simple_client.rs`](examples/simple_client.rs) | Basic IRC client connection |
| [`bot.rs`](examples/bot.rs) | Full-featured bot with command handling |
| [`sasl_auth.rs`](examples/sasl_auth.rs) | SASL PLAIN authentication flow |
| [`ctcp_handler.rs`](examples/ctcp_handler.rs) | CTCP VERSION/PING responses |
| [`isupport_parser.rs`](examples/isupport_parser.rs) | Parsing ISUPPORT (005) server capabilities |
| [`zero_copy_server.rs`](examples/zero_copy_server.rs) | High-performance server using `ZeroCopyTransport` |
| [`compliance_check.rs`](examples/compliance_check.rs) | Message compliance validation |

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

This project is released into the public domain under [The Unlicense](LICENSE).

## Acknowledgments

This project was inspired by the architectural patterns established by
[Aaron Weiss (aatxe)](https://github.com/aatxe) in the [irc](https://github.com/aatxe/irc) crate.

## Contributing

Contributions are welcome! Please open issues and pull requests on [GitHub](https://github.com/sid3xyz/slirc-proto).

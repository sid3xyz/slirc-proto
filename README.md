# slirc-proto

A robust IRC protocol library for parsing and encoding IRC messages with full IRCv3 support.

## Features

- **IRC Message Parsing**: Efficient parsing of IRC messages with tags, prefixes, commands, and parameters.
- **IRCv3 Support**: Full support for IRCv3 capability negotiation and message tags.
- **Zero-Copy Parsing**: Uses `Cow` and borrowed types where possible to minimize allocations.
- **Tokio Integration**: Optional `tokio` feature for async networking support (`IrcCodec`).
- **Mode Parsing**: Support for user and channel modes.
- **ISUPPORT**: Parsing of `RPL_ISUPPORT` tokens.
- **Builder Pattern**: Convenient message construction.

## Usage

### Creating IRC Messages

```rust
use slirc_proto::{Message, prefix::Prefix};

// Basic message construction
let privmsg = Message::privmsg("#rust", "Hello, world!");
let notice = Message::notice("nick", "Server notice");
let join = Message::join("#channel");

// Messages with IRCv3 tags and prefixes
let tagged_msg = Message::privmsg("#dev", "Tagged message")
    .with_tag("time", Some("2023-01-01T12:00:00Z"))
    .with_tag("msgid", Some("abc123"))
    .with_prefix(Prefix::new_from_str("bot!bot@example.com"));

println!("{}", tagged_msg); // Serializes to IRC protocol format
```

### Parsing IRC Messages

```rust
use slirc_proto::Message;

let raw = "@time=2023-01-01T12:00:00Z :nick!user@host PRIVMSG #channel :Hello!";
let message: Message = raw.parse().expect("Valid IRC message");

if let Some(tags) = &message.tags {
    println!("Message has {} tags", tags.len());
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slirc-proto = { git = "https://github.com/sid3xyz/slirc-proto" }
```

## License

Unlicense

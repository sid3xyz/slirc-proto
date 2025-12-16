# slirc-proto

IRC protocol library for parsing and encoding IRC messages with IRCv3 support.

**Status**: Research prototype. Not production ready.

## Features

Verified from `src/` and `Cargo.toml`:

- **Zero-copy parsing**: `MessageRef<'a>` in `src/message/borrowed.rs` borrows from input string
- **Owned messages**: `Message` in `src/message/mod.rs` for storage and modification
- **RFC compliance**: RFC 1459, RFC 2812, IRCv3 extensions
- **Semantic errors**: 58 `Response::err_*()` constructors in `src/response/constructors.rs`
- **IRCv3 support**: Message tags, capabilities, SASL, batch, server-time
- **Mode parsing**: Channel and user modes in `src/mode/`
- **ISUPPORT**: Server parameter parsing in `src/isupport/`
- **Optional tokio**: Async codec in `src/irc.rs`, transport layer in `src/transport/`
- **WebSocket**: IRC-over-WebSocket in `src/websocket.rs`
- **Utilities**: Wildcard matching, hostmask matching, case mapping

## Usage

```rust
use slirc_proto::{Message, MessageRef};

// Zero-copy parsing
let raw = ":nick!user@host PRIVMSG #channel :Hello!";
let msg = MessageRef::parse(raw).unwrap();
assert_eq!(msg.command_name(), "PRIVMSG");

// Owned message construction
let privmsg = Message::privmsg("#rust", "Hello, world!");
println!("{}", privmsg); // Serializes to IRC protocol
```

## Build

```bash
cargo build --all-features
cargo test --all-features
```

## License

Unlicense

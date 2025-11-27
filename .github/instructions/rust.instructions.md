---
applyTo: "**/*.rs"
---

# Rust Code Guidelines for slirc-proto

## Zero-Copy Patterns

- Prefer `MessageRef<'a>` over `Message` when data doesn't need to outlive the current scope
- Use `Cow<'a, str>` for strings that are usually borrowed but occasionally owned
- Avoid `String::clone()` — use references or `Cow` instead
- For small collections (1-2 items typical), consider `SmallVec` over `Vec`

## Parsing with nom

- Use simple `nom::error::Error` type, NOT `VerboseError` (performance critical)
- Parser functions return `IResult<&str, T>` or `IResult<&[u8], T>`
- Compose parsers with combinators: `preceded`, `delimited`, `separated_list0`, etc.
- Handle the full IRC grammar including edge cases (empty params, CRLF variations)

## Error Handling

- Use crate error types: `ProtocolError`, `MessageParseError`
- Never use `unwrap()` or `expect()` in library code
- Propagate errors with `?` operator
- Provide context in error messages (e.g., "failed to parse prefix" not just "parse error")

## Enum Guidelines

- Public enums that may grow use `#[non_exhaustive]`
- Match arms should handle `_` wildcard for non-exhaustive enums
- Command and Response enums follow this pattern

## Async Code (transport.rs)

- Use `tokio_util::codec::Framed` for line-based protocols
- Ensure cancellation safety in async operations
- Handle connection drops and reconnection gracefully
- Use `tracing` for structured logging, not `println!`

## Style Requirements

- All code must pass `cargo clippy -- -D warnings`
- Use `cargo fmt` formatting
- Prefer explicit type annotations in public APIs
- Document public items with `///` doc comments

## Testing Patterns

```rust
// Round-trip test example
#[test]
fn test_command_round_trip() {
    let raw = ":nick!user@host PRIVMSG #channel :Hello world\r\n";
    let msg = Message::parse(raw).unwrap();
    assert_eq!(msg.to_string(), raw);
}
```

## Common Imports

```rust
use crate::error::{MessageParseError, ProtocolError};
use crate::message::{Message, MessageRef};
use std::borrow::Cow;
```

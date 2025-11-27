---
applyTo: "**/*.rs"
---

# Rust Code Style for slirc-proto

## Zero-Copy Patterns

- Use `MessageRef<'a>` when data doesn't need to outlive the scope
- Use `Cow<'a, str>` for strings that are usually borrowed
- Use `write_to(&mut impl fmt::Write)` for serialization (no intermediate String)
- Avoid `String::clone()` — use references or Cow

## nom Parsing

- Use `nom::error::Error`, NOT `VerboseError` (performance critical)
- Return `IResult<&str, T>` or `IResult<&[u8], T>`
- Handle edge cases: empty params, CRLF variations, IRCv3 tag escaping

## Error Handling

- Use `ProtocolError` for transport, `MessageParseError` for parsing
- Never `unwrap()` or `expect()` in library code
- Propagate with `?`, provide context in messages

## Enums

- Use `#[non_exhaustive]` on public enums that may grow
- Handle `_` wildcard in match arms for non-exhaustive types

## Async (transport.rs)

- Uses `Framed<T, IrcCodec>` for line-based I/O
- Ensure cancellation safety
- Use `tracing` for logging, not `println!`

## Style

- Must pass `cargo clippy --all-features -- -D warnings`
- Use `cargo fmt` formatting
- Document public items with `///`

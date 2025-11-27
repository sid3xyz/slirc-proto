# Copilot Instructions for slirc-proto

This is a high-performance Rust library for parsing and encoding IRC protocol messages with full IRCv3 support.

## Build & Test Commands

```bash
# Build (default features include tokio async transport)
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Lint (must pass with zero warnings)
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt -- --check
```

## Project Requirements

- **MSRV**: Rust 1.70 minimum
- **Edition**: 2021
- **Linting**: `#![deny(clippy::all)]` is enforced—all clippy warnings are errors
- **No unwrap()**: Library code must propagate errors with `?`, never `unwrap()` or `expect()`

## Feature Flags

| Feature | Description |
|---------|-------------|
| `tokio` (default) | Async transport with Tokio, TLS, WebSocket support |
| `proptest` | Property-based testing support |
| `encoding` | Character encoding support via encoding_rs |

## Architecture Overview

- **Zero-copy parsing**: Use `MessageRef<'a>` for hot paths, convert to owned `Message` only when needed
- **Parser**: nom combinators with simple `Error` type (not `VerboseError`)
- **Error types**: `ProtocolError` for transport, `MessageParseError` for parsing
- **Enums**: Key enums use `#[non_exhaustive]` for API stability

## Key Modules

| Module | Purpose |
|--------|---------|
| `message/` | Core message types, parsing, serialization |
| `command/` | IRC command enum and parsing |
| `prefix/` | Nick!user@host prefix handling |
| `mode/` | Channel and user mode parsing |
| `ircv3/` | IRCv3 extensions (batch, msgid, server-time) |
| `transport.rs` | Async Tokio codec and connection handling |

## Testing Requirements

- All new commands need round-trip tests (parse → serialize → parse)
- Use `proptest` for fuzzing parser inputs when adding new parsing logic
- Benchmarks in `benches/parsing.rs` for performance-critical changes

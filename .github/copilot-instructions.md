# Copilot Instructions for slirc-proto

High-performance Rust library for IRC protocol parsing with full IRCv3 support. Released to the public domain under [The Unlicense](../LICENSE).

## Quick Reference

```bash
cargo build --all-features    # Build with all features
cargo test --all-features     # Run all tests
cargo clippy --all-features -- -D warnings  # Lint (must pass)
cargo fmt -- --check          # Format check
cargo bench                   # Run benchmarks
```

## Project Constraints

| Constraint | Requirement |
|------------|-------------|
| MSRV | Rust 1.70+ |
| Linting | `#![deny(clippy::all)]` — zero warnings allowed |
| Error handling | Use `?` propagation, never `unwrap()` or `expect()` in lib code |
| API stability | `#[non_exhaustive]` on public enums that may grow |

## Feature Flags

- `tokio` (default) — Async transport with TLS, WebSocket
- `proptest` — Property-based testing
- `encoding` — Character encoding via encoding_rs

## Architecture

| Component | Pattern |
|-----------|---------|
| Parsing | `MessageRef<'a>` zero-copy, nom combinators with simple `Error` |
| Serialization | `write_to(&mut impl fmt::Write)` to avoid allocations |
| Transport (owned) | `Framed<T, IrcCodec>` for handshake, returns `Message` |
| Transport (zero-copy) | `ZeroCopyTransport<S>` for hot loop, yields `MessageRef<'_>` |
| Errors | `ProtocolError` (transport), `MessageParseError` (parsing) |

## Testing Requirements

- Round-trip tests for all commands (parse → serialize → parse)
- Property tests with `proptest` for parser fuzzing
- Benchmarks in `benches/parsing.rs` for perf-critical changes

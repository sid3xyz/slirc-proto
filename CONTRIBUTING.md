# Contributing to slirc-proto

Thank you for your interest in contributing to slirc-proto! This document provides
guidelines and information for contributors.

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a
welcoming environment for all contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/slirc-proto.git`
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test --all-features`
6. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
7. Format code: `cargo fmt`
8. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later (MSRV)
- Cargo

### Building

```bash
# Build with default features (tokio)
cargo build

# Build with all features
cargo build --all-features

# Build without default features
cargo build --no-default-features
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench parsing
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

## Guidelines

### Code Style

- Follow Rust idioms and conventions
- Use `cargo fmt` for formatting
- All public items must have documentation
- Add examples to doc comments where appropriate
- Keep functions focused and small

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in imperative mood (e.g., "Add", "Fix", "Update")
- Reference issues when applicable (e.g., "Fix #123")

### Pull Requests

- Keep PRs focused on a single change
- Update documentation for API changes
- Add tests for new functionality
- Ensure all CI checks pass
- Update CHANGELOG.md for notable changes

### Adding New Commands

When adding new IRC commands:

1. Add the variant to `Command` enum in `src/command/types.rs`
2. Implement parsing in `src/command/parse/`
3. Implement serialization in `src/command/serialize.rs`
4. Add tests for both parsing and serialization
5. Document the command with RFC references

### Adding New Response Codes

When adding new numeric response codes:

1. Add the variant to `Response` enum in `src/response/mod.rs`
2. Add the code mapping in `from_code()` method
3. Add a doc comment with the code number and description
4. Reference the relevant RFC or specification

## RFC Compliance

This library aims for compliance with:

- [RFC 1459](https://tools.ietf.org/html/rfc1459) - Internet Relay Chat Protocol
- [RFC 2812](https://tools.ietf.org/html/rfc2812) - Internet Relay Chat: Client Protocol
- [IRCv3 Specifications](https://ircv3.net/)
- [Modern IRC Documentation](https://modern.ircdocs.horse/)

When implementing new features, please reference the appropriate specification
and include links in documentation.

## Questions?

If you have questions, feel free to:

- Open an issue for discussion
- Check existing issues for similar questions

## License

By contributing to slirc-proto, you agree that your contributions will be
released into the public domain under The Unlicense.

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed (Internal)

- Refactored `parse_message` in `src/message/nom_parser.rs`: extracted parameter parsing into `parse_params` helper
- Refactored `parse_modes` in `src/mode/parse.rs`: extracted argument resolution into `resolve_mode_arg` helper
- Simplified `src/command/parse/channel.rs`: replaced repetitive `if/else` chains with `match` expressions and `arg_opt` helper
- Extracted `parse_mode_command` helper in `src/command/parse/mod.rs` for cleaner dispatcher

### Note for Server Team

These are **internal refactorings only**. No public API changes. All functions modified are `pub(crate)`, `pub(super)`, or private.

## [1.1.0] - 2025-11-28

### Changed
- Improved `Clone` implementation for `MessageParseError::ParseContext` to preserve source error message instead of discarding it

### Fixed
- `ParseContext` clone no longer loses source error information (now stores error message string)

## [1.0.0] - 2025-11-27

### 🎉 Stable Release

This marks the first stable release of slirc-proto. The API is now considered stable
and follows semantic versioning guarantees.

### Added
- `#[non_exhaustive]` attribute on `Command`, `Response`, `ProtocolError`, `MessageParseError`, and `ModeParseError` enums for future extensibility
- Comprehensive RFC compliance helper methods on `Response`:
  - `is_reply()` - Check if response is a command reply (200-399)
  - `is_sasl()` - Check if response is SASL-related (900-908)
  - `is_channel_related()` - Check for channel-specific responses
  - `is_whois_related()` - Check for WHOIS/WHOWAS responses
  - `category()` - Get RFC 2812 category name
- Criterion-based benchmarks for parsing and serialization performance
- `#![deny(clippy::all)]` enforcement for code quality
- `#![warn(missing_docs)]` for documentation coverage
- MSRV badge in README (Rust 1.70)

### Changed
- Version bump to 1.0.0 (stable API)
- Improved crate description and keywords
- Added `parser-implementations` category

### Fixed
- All clippy warnings resolved
- Example code updated for latest API

## [0.3.0] - 2025-11-26

### Changed
- Updated all examples to use current API
- Fixed Transport API usage in examples

### Fixed
- Example compilation errors from API changes

## [0.2.0] - 2025-01-01

### Changed
- Complete clean-room rewrite of all protocol types
- Implemented IRC numeric response codes from RFC 2812 and modern IRC docs
- Implemented Command enum with all RFC 2812 commands and IRCv3 extensions
- Implemented UserMode/ChannelMode/Mode types with full mode support
- Implemented Prefix parsing with validation
- Implemented IRCv3 capability negotiation
- Implemented channel name validation (ChannelExt trait)
- Implemented IRC color/format code stripping
- Implemented error types for protocol parsing
- Implemented IrcCodec and LineCodec for tokio

### Added
- WebSocket transport support with TLS
- ISUPPORT parsing for server capability advertisements
- Zero-copy message reference types (`MessageRef`, `CommandRef`, `PrefixRef`)
- IRCv3 message ID generation and batch reference utilities
- Server time formatting helpers
- Non-IRC protocol detection (scanner module)
- Comprehensive unit tests for all modules

### Fixed
- Memory efficiency improvements with borrowed types

## [0.1.0] - Initial Release

- IRC protocol parsing and serialization
- IRCv3 capability negotiation
- Tokio-based async transport

---

## Acknowledgments

This project was inspired by the excellent [irc](https://github.com/aatxe/irc) crate
created by [Aaron Weiss (aatxe)](https://github.com/aatxe). Aaron's work on IRC
protocol handling in Rust provided valuable architectural insights that informed
the design of this library. We extend our sincere thanks for his contributions
to the Rust IRC ecosystem.

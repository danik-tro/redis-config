# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `redis-config`, a Rust crate that provides an asynchronous Redis source for the `config-rs` configuration library. It allows applications to load configuration data from Redis using either Hash (HGETALL) or String (GET) operations, integrating seamlessly with the config-rs ecosystem.

## Commands

### Building and Checking
- `cargo check --all-features` - Check compilation without producing executables
- `cargo build` - Build the project
- `cargo build --all-features` - Build with all features enabled

### Testing
- `cargo nextest run --workspace --all-targets` - Run tests using nextest (preferred test runner)
- `cargo test` - Run tests using standard test runner
- Set `TEST_REDIS_URL=redis://127.0.0.1:6379` environment variable for integration tests

### Code Quality
- `cargo fmt --all -- --check` - Check code formatting (fails CI if not formatted)
- `cargo fmt` - Format code
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::all -W clippy::pedantic` - Run Clippy lints (strict settings used in CI)

### Examples
- `cargo check --examples` - Check that examples compile
- `cargo run --example redis_key_source` - Run the key source example

## Architecture

### Core Components

**RedisSource<SK, S>** (`src/lib.rs`): The main configuration source that implements the `AsyncSource` trait from config-rs. It supports two states:
- `states::PlainString`: Retrieves configuration from Redis strings using GET
- `states::Hash`: Retrieves configuration from Redis hashes using HGETALL
- `states::Json`: Retrieves configuration from Redis JSON using JSON.GET [key] $ command

**States** (`src/states.rs`): Type-level state markers that determine how RedisSource interacts with Redis:
- `PlainString`: For JSON strings stored as Redis values
- `Hash`: For key-value pairs stored as Redis hash fields
- `Json`: For JSON objects stored as Redis JSON values

**RedisCell<I>** (`src/redis_cell.rs`): A smart pointer-like wrapper that can refresh its contents from Redis, useful for configuration that needs periodic updates.

**Error Handling** (`src/errors.rs`): Comprehensive error types covering Redis connection issues, serialization problems, and missing keys.

### Key Design Patterns

- **Type-level state pattern**: Uses phantom types to enforce compile-time correctness about Redis operation types
- **Async/await throughout**: Built for async runtimes (tokio by default)
- **Flexible key types**: Accepts any type implementing `redis::ToRedisArgs` as source keys
- **Optional vs Required sources**: Sources can be marked as optional to handle missing keys gracefully

## Features and Runtime Support

- `tokio-comp` (default): Tokio runtime support
- `ahash`: Performance optimization using ahash
- TLS features: Various combinations of native-tls and rustls for both runtimes

## Testing Requirements

Integration tests require a running Redis server. The CI uses `redis://127.0.0.1:6379` by default. Tests use the `TEST_REDIS_URL` environment variable if set.

## MSRV and Compatibility

- Minimum supported Rust version: 1.70.0
- Uses redis-rs 0.25.2 with MultiplexedConnection for improved performance
- Compatible with config-rs 0.14.0
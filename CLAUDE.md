# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

raps-mock is a Rust-based mock server for Autodesk Platform Services (APS) APIs. It auto-generates routes from OpenAPI 3.0 specifications and can run in either stateless (fixed responses) or stateful (in-memory storage) mode. It's designed as both a library and standalone CLI server.

## Build and Development Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Testing
cargo test                     # Run unit and integration tests
cargo test --all-features      # Run all tests including feature-gated ones
cargo test <test_name>         # Run a single test

# Run specific integration test
cargo test --test basic_test   # Run tests/integration/basic_test.rs

# APS OpenAPI smoke test (requires aps_ci feature and APS_OPENAPI_DIR env var)
APS_OPENAPI_DIR=./aps-sdk-openapi cargo test --features aps_ci --test aps_repo_smoke

# Code Quality
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting (CI mode)
cargo clippy --all-targets --all-features -- -D warnings  # Lint with warnings as errors

# Run the server
cargo run -- --port 3000 --mode stateful
cargo run -- --openapi-dir ../aps-sdk-openapi --port 3000 --verbose

# Run example
cargo run --example basic-usage
```

## Architecture

### Core Flow
`main.rs` → CLI parsing (clap) → `MockServer::new()` → OpenAPI parsing → Router building → axum server

### Key Modules

- **`server.rs`**: `MockServer` struct that orchestrates startup. Parses OpenAPI specs, creates `StateManager` (if stateful mode), builds the axum router via `server/router.rs`.

- **`openapi/`**: OpenAPI 3.0 spec handling
  - `parser.rs`: Recursively parses YAML/JSON specs from a directory, converts OpenAPI path params (`{param}`) to axum format (`:param`)
  - `types.rs`: Serde structs for OpenAPI schema elements

- **`handlers/`**: Request handlers
  - `generic.rs`: `GenericHandler` extracts example responses from OpenAPI specs (checks `example`, `examples`, schema example)
  - `stateful.rs`: Handlers with state mutations
  - `custom.rs`: `CustomHandlerRegistry` for user-defined endpoint overrides

- **`state/`**: In-memory storage for stateful mode
  - `manager.rs`: `StateManager` holds `Arc` references to all state modules
  - Individual modules (`auth.rs`, `buckets.rs`, `objects.rs`, `projects.rs`, `translations.rs`, `issues.rs`, `webhooks.rs`) each manage specific APS resource types using `dashmap`

- **`middleware/`**: axum middleware for auth, CORS, error handling

### Operation Modes

- **Stateless**: Returns fixed example responses from OpenAPI specs
- **Stateful**: Maintains in-memory state, supports CRUD operations on mocked resources

## Testing

Integration tests in `tests/integration/` require the `aps-sdk-openapi` directory to be present (default location: `../aps-sdk-openapi`). The `aps_repo_smoke.rs` test is feature-gated behind `aps_ci` and runs in CI against the real APS OpenAPI repo.

## Dependencies

- Rust 1.88+
- axum 0.7 (HTTP server)
- tokio (async runtime)
- dashmap (concurrent state storage)
- serde_yaml (OpenAPI spec parsing)

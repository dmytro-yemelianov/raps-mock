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
cargo run -- --db /tmp/mock.db --port 3000  # Persistent SQLite storage

# Run example
cargo run --example basic-usage
```

## Architecture

### Core Flow
`main.rs` → CLI parsing (clap) → `MockServer::new()` → OpenAPI parsing → Router building → axum server

### Key Modules

- **`server.rs`**: `MockServer` struct that orchestrates startup. Parses OpenAPI specs, creates `StateManager` (if stateful mode), builds the axum router via `server/router.rs`.

- **`server/router.rs`**: Builds the axum router. Routes are registered in two phases: (1) dynamic routes from OpenAPI specs, (2) hardcoded fallback routes. Duplicate detection ensures OpenAPI routes take precedence over hardcoded ones.

- **`openapi/`**: OpenAPI 3.0 spec handling
  - `parser.rs`: Recursively parses YAML/JSON specs from a directory, converts OpenAPI path params (`{param}`) to axum format (`:param`)
  - `types.rs`: Serde structs for OpenAPI schema elements (`RouteDefinition`, `HttpMethod`)

- **`handlers/`**: Request handlers
  - `generic.rs`: `GenericHandler` extracts example responses from OpenAPI specs (checks `example`, `examples`, schema example)
  - `custom.rs`: `CustomHandlerRegistry` for user-defined endpoint overrides

- **`state/`**: SQLite-backed storage for stateful mode
  - `db.rs`: `Db` wrapper around `Mutex<Connection>` — in-memory by default, file-backed with `--db`
  - `manager.rs`: `StateManager` holds `Arc<T>` references to all state modules; `new()` for in-memory, `with_db(path)` for persistent
  - `auth.rs`: OAuth token generation and validation
  - `buckets.rs`: OSS bucket CRUD
  - `objects.rs`: OSS object metadata (composite PK: bucket_key + object_key)
  - `projects.rs`: Data Management hubs and projects
  - `translations.rs`: Model Derivative job tracking with status progression
  - `issues.rs`: ACC Issues + Comments CRUD
  - `webhooks.rs`: Webhook subscription management
  - `da.rs`: Design Automation (app bundles, activities, work items)
  - `reality.rs`: Reality Capture photoscenes
  - `acc.rs`: ACC RFIs, Assets, Submittals, Checklists

- **`middleware/`**: axum layers for auth validation and CORS

- **`testing.rs`**: `TestServer` helper that starts a mock server on a random port for integration tests. Auto-cleans up on drop.

### State Persistence

State is stored in SQLite (17 tables). Two modes:

- **In-memory** (default): `StateManager::new()` — fast, no persistence, same as original DashMap behavior
- **File-backed**: `StateManager::with_db(path)` via `--db <PATH>` — state survives restarts, WAL mode for performance

Pre-seeded demo data uses `INSERT OR IGNORE` — idempotent on persistent databases.

### Operation Modes

- **Stateless**: Returns fixed example responses from OpenAPI specs
- **Stateful**: Maintains SQLite state, supports CRUD operations on mocked resources

### Library Usage

The crate exports `MockServer`, `MockServerConfig`, `MockMode`, and `TestServer` from `lib.rs`. Use `TestServer::start_default()` for quick test setup:

```rust
let server = TestServer::start_default().await.unwrap();
// server.url contains "http://127.0.0.1:<random_port>"
```

## Testing

Integration tests in `tests/integration/` require the `aps-sdk-openapi` directory to be present (default location: `../aps-sdk-openapi`). The `aps_repo_smoke.rs` test is feature-gated behind `aps_ci` and runs in CI against the real APS OpenAPI repo.

CI runs two jobs: `build-and-test` (format, clippy, build, test) and `aps-openapi-smoke` (clones the APS OpenAPI repo and runs smoke tests).

## Dependencies

- Rust 1.88+
- axum 0.7 (HTTP server)
- tokio (async runtime)
- rusqlite 0.32 bundled (SQLite state storage)
- serde_yaml (OpenAPI spec parsing)

# Repository Guidelines

## Project Structure & Module Organization
- Source code lives in `src/` with feature-focused modules: `handlers/`, `middleware/`, `openapi/`, `state/`, and server entry points in `server.rs`, `lib.rs`, `main.rs`.
- Integration tests in `tests/integration/` and fixtures in `tests/fixtures/` (e.g., `token_response.json`).
- Examples in `examples/` (e.g., `basic-usage.rs`).
- Package metadata in `Cargo.toml`; build artifacts in `target/`.

## Build, Test, and Development Commands
- Build: `cargo build` (debug) or `cargo build --release`.
- Run CLI: `cargo run -- --port 3000 --mode stateful --openapi-dir ../aps-sdk-openapi`.
- Run example: `cargo run --example basic-usage`.
- Tests: `cargo test` (runs unit + integration tests).
- Lint/format: `cargo fmt --all` and `cargo clippy --all-targets --all-features -D warnings`.

## Coding Style & Naming Conventions
- Rust 2024 edition; prefer idiomatic Rust and small, composable modules.
- Indentation: 4 spaces; keep lines readable (<100 chars where practical).
- Naming: modules/files `snake_case`, types/traits `PascalCase`, functions/vars `snake_case`, constants `SCREAMING_SNAKE_CASE`.
- Use `thiserror` for error types and `anyhow::Result` for application-level results where appropriate.
- Logging via `tracing`; keep messages actionable and structured.

## Testing Guidelines
- Integration tests live under `tests/integration/*.rs`; name tests descriptively (e.g., `dynamic_test.rs`).
- Use `#[tokio::test]` for async tests; prefer realistic inputs (see `tests/fixtures/`).
- Tests should pass without external services; use temporary dirs and local OpenAPI samples.
- Run: `cargo test` or filter with `cargo test test_stateless_mode`.

## Commit & Pull Request Guidelines
- Commits: use clear, imperative messages; group related changes. Conventional prefixes encouraged: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- PRs: include a summary, rationale, and scope; link issues (e.g., `Closes #123`). Add CLI output or screenshots when behavior changes.
- Keep diffs focused; add/update docs (README, examples, comments) when user-facing behavior changes.

## Security & Configuration Tips
- No secrets required; the server mocks APS APIs from OpenAPI specs.
- Key flags: `--mode {stateless|stateful}`, `--openapi-dir <path>`, `--state-file <path>`, `--host`, `--port`.
- Prefer `stateful` for workflows needing in-memory persistence; use `--state-file` to persist between runs.

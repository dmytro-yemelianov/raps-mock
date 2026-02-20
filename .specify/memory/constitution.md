<!--
SYNC IMPACT REPORT
==================
Version change: N/A → 1.0.0 (initial ratification)

Modified principles: N/A (initial version)

Added sections:
- Core Principles (5 principles)
- Technical Standards
- Development Workflow
- Governance

Removed sections: N/A (initial version)

Templates requiring updates:
- .specify/templates/plan-template.md: ✅ Compatible (Constitution Check section exists)
- .specify/templates/spec-template.md: ✅ Compatible (requirements structure aligns)
- .specify/templates/tasks-template.md: ✅ Compatible (phase structure supports principles)

Follow-up TODOs: None
-->

# raps-mock Constitution

## Core Principles

### I. OpenAPI Fidelity

Every mock response MUST faithfully represent the Autodesk Platform Services API contracts as defined in OpenAPI specifications.

- Route paths, HTTP methods, and parameter names MUST match OpenAPI specs exactly
- Response schemas MUST conform to OpenAPI-defined structures
- Example responses MUST be valid instances of their defined schemas
- When OpenAPI specs are updated, mock behavior MUST be updated to match
- Deviations from OpenAPI specs require explicit documentation and justification

**Rationale**: The mock server's value depends entirely on its accuracy. Tests passing against an inaccurate mock provide false confidence.

### II. Test-First Development

All new functionality MUST be developed using test-driven development (TDD). Tests define the contract before implementation.

- Write failing tests before implementing features
- Integration tests are mandatory for all stateful endpoints
- Contract tests MUST verify OpenAPI compliance
- `cargo test` MUST pass before any PR merge
- Test coverage for new code SHOULD exceed 80%

**Rationale**: A mock server exists to enable testing. A mock server that isn't itself well-tested cannot be trusted.

### III. Library-First Architecture

raps-mock MUST be usable as a Rust library first; CLI is a thin wrapper.

- Core functionality lives in `lib.rs` and submodules, not `main.rs`
- `MockServer`, `MockServerConfig`, `MockMode`, and `TestServer` MUST be public exports
- Internal state management MUST NOT leak into the public API
- Library users MUST be able to start a mock server with <10 lines of code
- Breaking changes to public API require MAJOR version bump

**Rationale**: The primary use case is embedding in test suites. Library ergonomics take priority over CLI features.

### IV. Stateful Consistency

In stateful mode, all state operations MUST be consistent, isolated, and predictable.

- Concurrent access MUST be safe (use `DashMap` or equivalent)
- Each `TestServer` instance MUST have isolated state
- State mutations MUST be atomic at the operation level
- Query results MUST reflect all prior mutations immediately
- State MUST NOT persist between server restarts unless explicitly configured

**Rationale**: Flaky tests from race conditions or state leakage undermine the mock server's purpose.

### V. Developer Experience

Integration MUST be frictionless. Errors MUST be actionable.

- Zero configuration required for basic usage (`TestServer::start_default()`)
- Error messages MUST include: what failed, why, and how to fix
- Verbose mode MUST log requests/responses for debugging
- Documentation MUST include working code examples
- Startup time SHOULD be under 100ms for test scenarios

**Rationale**: Developers adopt tools that reduce friction. Confusing errors waste developer time.

## Technical Standards

### Language and Tooling

- **Rust Version**: 1.88+ (as specified in Cargo.toml)
- **HTTP Framework**: axum 0.7
- **Async Runtime**: tokio
- **State Storage**: dashmap for concurrent access
- **Serialization**: serde with serde_json and serde_yaml

### Code Quality Gates

- `cargo fmt -- --check` MUST pass (formatting)
- `cargo clippy --all-targets --all-features -- -D warnings` MUST pass (linting)
- `cargo test` MUST pass (unit and integration tests)
- No `unsafe` code without explicit justification and review

### API Versioning

- Follow Semantic Versioning (SemVer)
- MAJOR: Breaking changes to public API or mock behavior changes that would break existing tests
- MINOR: New API coverage, new features, backward-compatible additions
- PATCH: Bug fixes, documentation, internal refactoring

## Development Workflow

### Feature Development

1. Create feature branch from `main`
2. Write/update specification in `.specify/specs/` if applicable
3. Write failing tests that define expected behavior
4. Implement until tests pass
5. Run full test suite (`cargo test --all-features`)
6. Update documentation if public API changed
7. Create PR with clear description of changes

### Code Review Requirements

- All PRs require at least one approval
- CI MUST pass (format, lint, test)
- Breaking changes require explicit acknowledgment in PR description
- New public API items require documentation

### Commit Standards

- Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- Reference issues where applicable
- Keep commits atomic and focused

## Governance

### Constitution Authority

This constitution supersedes all other development practices for raps-mock. When in conflict, the constitution takes precedence.

### Amendment Process

1. Propose amendment via PR to `.specify/memory/constitution.md`
2. Include rationale for change and impact assessment
3. Update version according to SemVer:
   - MAJOR: Principle removal, fundamental governance change
   - MINOR: New principle, significant expansion of existing principle
   - PATCH: Clarification, typo fix, non-semantic refinement
4. Update dependent templates if affected
5. Merge requires maintainer approval

### Compliance

- All PRs SHOULD be checked against constitution principles
- Violations MUST be justified in PR description if unavoidable
- Repeated violations indicate need for constitution review

### Guidance Files

- `CLAUDE.md`: AI assistant guidance for development tasks
- `README.md`: User-facing documentation and quick start
- `ROADMAP.md`: Feature evolution plan

**Version**: 1.0.0 | **Ratified**: 2026-01-15 | **Last Amended**: 2026-01-15

# Implementation Plan: ACC Submittals Mock Support

**Branch**: `001-acc-submittals` | **Date**: 2026-01-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-acc-submittals/spec.md`

## Summary

Add stateful mock support for ACC Submittals API, enabling developers to test submittal CRUD operations without connecting to the real APS API. The implementation follows the existing state manager pattern (like `issues.rs`, `buckets.rs`) with a new `submittals.rs` module. Endpoints will support listing, creating, reading, updating, and deleting submittals scoped to project IDs.

## Technical Context

**Language/Version**: Rust 1.88+ (as specified in Cargo.toml)
**Primary Dependencies**: axum 0.7, tokio, dashmap (for concurrent state), serde/serde_json, uuid, chrono
**Storage**: In-memory (DashMap), following existing state manager pattern
**Testing**: cargo test with integration tests using TestServer
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Single project (Rust library + CLI)
**Performance Goals**: <50ms response time for typical operations
**Constraints**: State isolated per server instance, no persistence (in-memory only)
**Scale/Scope**: Typical test scenarios with 10-100 submittals per project

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. OpenAPI Fidelity** | ✅ PASS | Response schemas match ACC Submittals API structure |
| **II. Test-First Development** | ✅ PASS | Integration tests written before implementation |
| **III. Library-First Architecture** | ✅ PASS | State manager is internal; no new public API required |
| **IV. Stateful Consistency** | ✅ PASS | Uses DashMap for thread-safe concurrent access |
| **V. Developer Experience** | ✅ PASS | Follows existing patterns; no new configuration needed |

**Gate Result**: All principles satisfied. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-acc-submittals/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── config.rs            # No changes needed
├── error.rs             # No changes needed
├── lib.rs               # No changes needed (state managers are internal)
├── main.rs              # No changes needed
├── server.rs            # No changes needed
├── state/
│   ├── mod.rs           # Add submittals module export
│   ├── manager.rs       # Add SubmittalsState to StateManager
│   ├── submittals.rs    # NEW: Submittals state manager
│   ├── auth.rs
│   ├── buckets.rs
│   ├── issues.rs        # Pattern to follow
│   ├── objects.rs
│   ├── projects.rs
│   ├── translations.rs
│   └── webhooks.rs
├── handlers/
│   ├── mod.rs           # Register submittal handlers
│   └── submittals.rs    # NEW: HTTP handlers for submittals endpoints
├── middleware/
├── openapi/
└── testing.rs           # No changes needed

tests/
└── integration/
    └── submittals_test.rs  # NEW: Integration tests for submittals
```

**Structure Decision**: Single project structure maintained. New files follow existing patterns established by `issues.rs` state manager and handlers.

## Complexity Tracking

> No constitution violations requiring justification.

N/A - Implementation follows established patterns with minimal complexity.

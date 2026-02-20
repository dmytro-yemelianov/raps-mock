# Implementation Plan: State Persistence and Fixtures

**Branch**: `002-state-persistence` | **Date**: 2026-01-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-state-persistence/spec.md`

## Summary

Add state persistence and fixture capabilities to raps-mock, enabling developers to save and restore server state to/from files. This feature introduces a fixture format (YAML/JSON) for defining pre-configured test scenarios, a CLI option to load fixtures at startup, and a programmatic API for loading fixtures in tests. The StateManager will be extended with serialization capabilities.

## Technical Context

**Language/Version**: Rust 1.88+ (as specified in Cargo.toml)
**Primary Dependencies**: axum 0.7, tokio, serde/serde_json/serde_yaml, dashmap
**Storage**: Local filesystem (JSON/YAML fixture files)
**Testing**: cargo test with integration tests using TestServer
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Single project (Rust library + CLI)
**Performance Goals**: <1s to load fixtures with 100 resources
**Constraints**: Fixtures must be human-editable, portable across versions
**Scale/Scope**: Typical fixtures with 10-500 resources

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. OpenAPI Fidelity** | ✅ PASS | Fixtures store data matching API response schemas |
| **II. Test-First Development** | ✅ PASS | Integration tests written before implementation |
| **III. Library-First Architecture** | ✅ PASS | `load_fixture()` exposed via MockServer; CLI wraps library |
| **IV. Stateful Consistency** | ✅ PASS | Loaded fixtures immediately available; atomic replacement |
| **V. Developer Experience** | ✅ PASS | Simple `--fixture` flag; <5 lines for programmatic loading |

**Gate Result**: All principles satisfied. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/002-state-persistence/
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
├── config.rs            # Add fixture_path config option
├── error.rs             # Add fixture-related errors
├── lib.rs               # Re-export Fixture type
├── main.rs              # Add --fixture CLI flag
├── server.rs            # Load fixture on startup
├── fixture/             # NEW: Fixture subsystem
│   ├── mod.rs           # Module exports
│   ├── types.rs         # Fixture, FixtureMetadata structs
│   ├── loader.rs        # File loading and parsing
│   └── exporter.rs      # State export to fixture
├── state/
│   ├── mod.rs           # No changes
│   ├── manager.rs       # Add load_fixture(), export_fixture() methods
│   └── [other files]    # Add Serialize derive to state structs
├── handlers/
├── middleware/
├── openapi/
└── testing.rs           # Add start_with_fixture() helper

tests/
└── integration/
    └── fixtures_test.rs    # NEW: Fixture loading tests

fixtures/                   # NEW: Sample fixtures directory
├── empty.yaml
├── basic-buckets.yaml
└── full-project.yaml
```

**Structure Decision**: Single project structure maintained. New `src/fixture/` module contains fixture I/O logic. StateManager extended with serialization methods. Sample fixtures added at repo root.

## Complexity Tracking

> No constitution violations requiring justification.

N/A - Implementation follows established patterns.

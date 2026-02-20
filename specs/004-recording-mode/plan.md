# Implementation Plan: Request Recording and Playback

**Branch**: `004-recording-mode` | **Date**: 2026-01-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-recording-mode/spec.md`

## Summary

Add request recording and playback capabilities to raps-mock, enabling developers to capture real APS API interactions and replay them offline. The feature introduces two new server modes: **Record Mode** (proxy requests to real backend while saving interactions) and **Playback Mode** (serve recorded responses without network access). This addresses the gap between fully mocked responses and live API testing.

## Technical Context

**Language/Version**: Rust 1.88+ (as specified in Cargo.toml)
**Primary Dependencies**: axum 0.7, tokio, reqwest (for proxying), serde/serde_json/serde_yaml (for recording serialization)
**Storage**: Local filesystem (JSON/YAML files for recordings)
**Testing**: cargo test with integration tests using TestServer
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Single project (Rust library + CLI)
**Performance Goals**: <50ms recording overhead, <500ms playback startup for 50 recordings
**Constraints**: Recordings stored locally only, configurable body size limits
**Scale/Scope**: Typical recording sessions of 50-200 interactions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. OpenAPI Fidelity** | ✅ PASS | Recordings capture actual APS API responses, inherently matching OpenAPI contracts |
| **II. Test-First Development** | ✅ PASS | Integration tests will be written before implementation; TDD approach required |
| **III. Library-First Architecture** | ✅ PASS | Recording/playback APIs exposed via `MockServer` and `MockServerConfig`; CLI is thin wrapper |
| **IV. Stateful Consistency** | ✅ PASS | Recordings are immutable once written; playback state is isolated per session |
| **V. Developer Experience** | ✅ PASS | Simple CLI flags (`--record`, `--playback`); programmatic API with clear error messages |

**Gate Result**: All principles satisfied. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/004-recording-mode/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal APIs)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── config.rs            # Extended with RecordingConfig, PlaybackConfig
├── error.rs             # Extended with recording-specific errors
├── lib.rs               # Re-export recording types
├── main.rs              # CLI flags for record/playback modes
├── server.rs            # Extended server initialization
├── recording/           # NEW: Recording subsystem
│   ├── mod.rs           # Module exports
│   ├── types.rs         # Recording, RecordingSession, MatchingRules structs
│   ├── recorder.rs      # Recording middleware and file writing
│   ├── player.rs        # Playback handler and request matching
│   ├── anonymizer.rs    # Token/credential redaction
│   └── storage.rs       # File I/O for recordings (JSON/YAML)
├── middleware/
│   └── recording.rs     # NEW: Recording middleware layer
├── handlers/
├── openapi/
├── state/
└── testing.rs           # Extended with recording test helpers

tests/
├── integration/
│   ├── recording_test.rs    # NEW: Recording mode tests
│   └── playback_test.rs     # NEW: Playback mode tests
└── unit/
    └── matching_test.rs     # NEW: Request matching tests
```

**Structure Decision**: Single project structure maintained. New `src/recording/` module follows existing patterns (like `src/state/`, `src/openapi/`). Recording functionality integrates with existing `MockServer` via extended `MockServerConfig`.

## Complexity Tracking

> No constitution violations requiring justification.

N/A - Implementation follows established patterns.

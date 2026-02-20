# Tasks: Request Recording and Playback

**Input**: Design documents from `/specs/004-recording-mode/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/internal-api.md

**Tests**: Not explicitly requested - tests are optional but recommended.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create recording module structure and define core types

- [ ] T001 Create src/recording/ directory structure with mod.rs
- [ ] T002 [P] Create src/recording/types.rs with Recording, RecordedRequest, RecordedResponse, RecordedBody structs
- [ ] T003 [P] Create src/recording/recorder.rs with Recorder struct skeleton
- [ ] T004 [P] Create src/recording/player.rs with Player struct skeleton
- [ ] T005 [P] Create src/recording/anonymizer.rs with Anonymizer struct skeleton
- [ ] T006 [P] Create src/recording/storage.rs with file I/O function stubs
- [ ] T007 Add recording module export in src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement RecordingConfig struct with output_dir, backend_url, anonymize, max_body_size, format in src/recording/types.rs
- [ ] T009 [P] Implement PlaybackConfig struct with recordings_dir, matching_mode, ignored_headers, sequential in src/recording/types.rs
- [ ] T010 [P] Implement RecordingMetadata struct with backend_url, anonymized, recorded_by, tags in src/recording/types.rs
- [ ] T011 [P] Implement RecordingSession struct in src/recording/types.rs
- [ ] T012 Add RecordingError enum in src/error.rs with InvalidOutputDir, InvalidBackendUrl, IoError, SerializationError, ProxyError variants
- [ ] T013 [P] Add PlaybackError enum in src/error.rs with InvalidRecordingsDir, NoRecordingsFound, InvalidRecording, NoMatch, IoError variants
- [ ] T014 Add recording_config and playback_config options to MockServerConfig in src/config.rs
- [ ] T015 Create src/middleware/recording.rs with recording_layer() and playback_layer() stubs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Record API Interactions (Priority: P1)

**Goal**: Capture request-response pairs while proxying to real APS API

**Independent Test**: Start in record mode, make API call, verify recording file created with request and response data

### Implementation for User Story 1

- [ ] T016 [US1] Implement Recorder::new(config: RecordingConfig) constructor in src/recording/recorder.rs
- [ ] T017 [US1] Implement Recorder::session_id() and recording_count() methods
- [ ] T018 [US1] Implement proxy_request() function using reqwest to forward requests in src/recording/recorder.rs
- [ ] T019 [US1] Capture request method, path, query, headers, body into RecordedRequest
- [ ] T020 [US1] Capture response status, headers, body, duration_ms into RecordedResponse
- [ ] T021 [US1] Implement Recorder::record() async method to save Recording to file
- [ ] T022 [US1] Implement write_recording() function with atomic write in src/recording/storage.rs
- [ ] T023 [US1] Implement recording_layer() middleware that intercepts, proxies, and records
- [ ] T024 [US1] Add --record CLI argument with backend_url in src/main.rs
- [ ] T025 [US1] Add --output-dir CLI argument for recording output in src/main.rs
- [ ] T026 [US1] Implement Recorder::finalize() to write session manifest
- [ ] T027 [US1] Generate sequential IDs for recordings (0, 1, 2...)

**Checkpoint**: User Story 1 complete - can record API interactions to files

---

## Phase 4: User Story 2 - Replay Recorded Sessions (Priority: P1)

**Goal**: Serve recorded responses without connecting to real API

**Independent Test**: Load recordings directory, make request matching recording, verify recorded response returned

### Implementation for User Story 2

- [ ] T028 [US2] Implement read_recording() function in src/recording/storage.rs
- [ ] T029 [US2] Implement read_recordings_dir() to load all recordings from directory
- [ ] T030 [US2] Implement Player::new(config: PlaybackConfig) constructor in src/recording/player.rs
- [ ] T031 [US2] Build index of recordings by (method, path) for fast lookup
- [ ] T032 [US2] Implement Player::find_match() using method + path matching
- [ ] T033 [US2] Implement playback_layer() middleware that matches and returns recorded responses
- [ ] T034 [US2] Return 404 with "No matching recording" message when no match found
- [ ] T035 [US2] Add --playback CLI argument with recordings_dir in src/main.rs
- [ ] T036 [US2] Support multiple recordings for same path (return first match)

**Checkpoint**: User Stories 1 AND 2 complete - can record and playback

---

## Phase 5: User Story 3 - Anonymize Sensitive Data (Priority: P2)

**Goal**: Redact credentials and tokens from recordings for safe sharing

**Independent Test**: Record with anonymization, inspect file, verify Authorization header is "[REDACTED]"

### Implementation for User Story 3

- [ ] T037 [US3] Implement AnonymizePattern struct with target, pattern, replacement in src/recording/types.rs
- [ ] T038 [US3] Implement AnonymizeTarget enum (Header, QueryParam, JsonPath)
- [ ] T039 [US3] Implement default_anonymize_patterns() returning Authorization, Cookie, access_token, refresh_token patterns
- [ ] T040 [US3] Implement Anonymizer::new() and Anonymizer::default() in src/recording/anonymizer.rs
- [ ] T041 [US3] Implement Anonymizer::anonymize_request() method
- [ ] T042 [US3] Implement Anonymizer::anonymize_response() method
- [ ] T043 [US3] Integrate Anonymizer into Recorder::record() when anonymize=true
- [ ] T044 [US3] Add --no-anonymize CLI flag to disable anonymization in src/main.rs

**Checkpoint**: User Story 3 complete - recordings can be safely shared

---

## Phase 6: User Story 4 - Flexible Request Matching (Priority: P2)

**Goal**: Match requests even when certain parameters vary

**Independent Test**: Configure ignored_headers, replay with different header value, verify match still works

### Implementation for User Story 4

- [ ] T045 [US4] Implement MatchingMode enum (Strict, PathOnly, Flexible, Sequential) in src/recording/types.rs
- [ ] T046 [US4] Implement normalize_request() to remove ignored headers/params for matching
- [ ] T047 [US4] Update Player::find_match() to use MatchingMode
- [ ] T048 [US4] Implement PathOnly matching (method + path only)
- [ ] T049 [US4] Implement Flexible matching with ignored_headers and ignored_query_params
- [ ] T050 [US4] Add --matching-mode CLI argument in src/main.rs
- [ ] T051 [US4] Add --ignore-header and --ignore-query CLI arguments

**Checkpoint**: User Story 4 complete - flexible playback matching available

---

## Phase 7: User Story 5 - Sequential Playback for Stateful Workflows (Priority: P3)

**Goal**: Replay recordings in order for stateful workflow testing

**Independent Test**: Record create-then-update sequence, replay in sequential mode, verify correct order

### Implementation for User Story 5

- [ ] T052 [US5] Add sequence tracking to Player with current_sequence counter
- [ ] T053 [US5] Implement Sequential MatchingMode that checks sequence number
- [ ] T054 [US5] Return error when request doesn't match expected sequence
- [ ] T055 [US5] Add --sequential CLI flag to enable sequence-based matching
- [ ] T056 [US5] Reset sequence counter on Player::reset() method

**Checkpoint**: All user stories complete - full recording/playback capabilities

---

## Phase 8: Public API Extensions

**Purpose**: Expose recording features via MockServer and TestServer

- [ ] T057 Implement MockServerConfig::record_mode() constructor in src/config.rs
- [ ] T058 [P] Implement MockServerConfig::playback_mode() constructor in src/config.rs
- [ ] T059 [P] Implement MockServer::is_recording() method in src/server.rs
- [ ] T060 [P] Implement MockServer::is_playback() method in src/server.rs
- [ ] T061 Implement TestServer::start_recording() method in src/testing.rs
- [ ] T062 [P] Implement TestServer::start_playback() method in src/testing.rs

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Large body handling, error cases, and integration tests

- [ ] T063 Implement RecordedBody truncation for bodies exceeding max_body_size
- [ ] T064 [P] Calculate and store SHA256 hash for body integrity verification
- [ ] T065 Handle binary content types (skip content, store metadata only)
- [ ] T066 [P] Implement FallbackMode enum (Error, Passthrough, Empty) for no-match handling
- [ ] T067 [P] Create tests/integration/recording_test.rs with record mode tests
- [ ] T068 [P] Create tests/integration/playback_test.rs with playback mode tests
- [ ] T069 Run cargo fmt and cargo clippy on new files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (record vs playback)
  - US3 depends on US1 (integrates into recording)
  - US4 depends on US2 (enhances playback matching)
  - US5 depends on US2 (extends playback)
- **Public API (Phase 8)**: Can start after US1 and US2 complete
- **Polish (Phase 9)**: Can start after US1 and US2 complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 3 (P2)**: Depends on US1 (anonymization in recording)
- **User Story 4 (P2)**: Depends on US2 (enhances find_match)
- **User Story 5 (P3)**: Depends on US2 (extends playback)

### Parallel Opportunities

- T002, T003, T004, T005, T006 can run in parallel (different files)
- T008, T009, T010, T011 can run in parallel (different structs)
- T012, T013 can run in parallel (different error enums)
- US1 and US2 can be developed in parallel
- T057-T062 can largely run in parallel
- T063-T068 can largely run in parallel

---

## Parallel Example: Setup Phase

```bash
# Launch all setup tasks together:
Task: "Create src/recording/types.rs with Recording struct"
Task: "Create src/recording/recorder.rs with Recorder skeleton"
Task: "Create src/recording/player.rs with Player skeleton"
Task: "Create src/recording/anonymizer.rs with Anonymizer skeleton"
Task: "Create src/recording/storage.rs with file I/O stubs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Record)
4. Complete Phase 4: User Story 2 (Playback)
5. **STOP and VALIDATE**: Test --record and --playback CLI flags
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add US1 -> Can record API interactions (partial MVP)
3. Add US2 -> Can playback recordings (full MVP!)
4. Add US3 -> Recordings are safe to share
5. Add US4 -> More flexible playback matching
6. Add US5 -> Support stateful workflow testing
7. Each story adds value without breaking previous stories

---

## Notes

- Recording format is JSON (primary), YAML supported via extension detection
- Use reqwest for HTTP proxying to backend
- Atomic file writes (temp + rename) prevent corruption
- Default anonymization patterns: Authorization header, Cookie header, access_token, refresh_token JSON fields
- Binary bodies (images, files) store metadata only when exceeding max_body_size
- Session manifest (session.json) created on Recorder::finalize()
- Dependencies: reqwest already in Cargo.toml

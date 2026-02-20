# Tasks: State Persistence and Fixtures

**Input**: Design documents from `/specs/002-state-persistence/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md

**Tests**: Not explicitly requested - tests are optional but recommended.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create fixture module structure and define core types

- [ ] T001 Create src/fixture/ directory structure with mod.rs
- [ ] T002 [P] Create src/fixture/types.rs with Fixture and FixtureMetadata structs
- [ ] T003 [P] Create src/fixture/loader.rs with empty function stubs
- [ ] T004 [P] Create src/fixture/exporter.rs with empty function stubs
- [ ] T005 Add fixture module export in src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Define FIXTURE_VERSION constant ("1.0") in src/fixture/types.rs
- [ ] T007 Implement Fixture::empty() constructor in src/fixture/types.rs
- [ ] T008 Implement Fixture::with_metadata() constructor in src/fixture/types.rs
- [ ] T009 [P] Add FixtureError enum in src/error.rs with NotFound, IoError, YamlError, JsonError, UnsupportedFormat, VersionMismatch, ValidationError variants
- [ ] T010 Add serde Serialize/Deserialize derives to all state info structs (BucketInfo, ObjectInfo, ProjectInfo, IssueInfo, TranslationInfo, WebhookInfo) in respective src/state/*.rs files
- [ ] T011 Add fixture_path: Option<PathBuf> field to MockServerConfig in src/config.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Load Fixture at Startup (Priority: P1)

**Goal**: Start the mock server with pre-configured state from a YAML/JSON fixture file

**Independent Test**: Start server with --fixture flag, verify resources are immediately available via API

### Implementation for User Story 1

- [ ] T012 [US1] Implement load_from_file() async function in src/fixture/loader.rs
- [ ] T013 [US1] Implement from_yaml() function for YAML parsing in src/fixture/loader.rs
- [ ] T014 [US1] Implement from_json() function for JSON parsing in src/fixture/loader.rs
- [ ] T015 [US1] Add file extension detection (.yaml, .yml, .json) in load_from_file()
- [ ] T016 [US1] Implement StateManager::load_fixture() method in src/state/manager.rs
- [ ] T017 [US1] Clear existing state before loading fixture in load_fixture()
- [ ] T018 [US1] Add --fixture CLI argument in src/main.rs using clap
- [ ] T019 [US1] Load fixture on server startup in src/server.rs when fixture_path is set
- [ ] T020 [US1] Add version validation (check fixture version matches FIXTURE_VERSION)

**Checkpoint**: User Story 1 complete - can start server with fixture file

---

## Phase 4: User Story 2 - Export State to Fixture (Priority: P1)

**Goal**: Save current server state to a portable fixture file for later use

**Independent Test**: Create resources via API, export fixture, verify file contains all resources

### Implementation for User Story 2

- [ ] T021 [US2] Implement StateManager::as_fixture() method in src/state/manager.rs
- [ ] T022 [US2] Implement to_yaml() function in src/fixture/exporter.rs
- [ ] T023 [US2] Implement to_json() function in src/fixture/exporter.rs
- [ ] T024 [US2] Implement save_to_file() async function with atomic write in src/fixture/exporter.rs
- [ ] T025 [US2] Implement MockServer::export_fixture() method in src/server.rs
- [ ] T026 [US2] Add export-state CLI subcommand in src/main.rs

**Checkpoint**: User Stories 1 AND 2 complete - can load and export fixtures

---

## Phase 5: User Story 3 - Programmatic Fixture Loading in Tests (Priority: P2)

**Goal**: Load fixtures programmatically in integration tests using TestServer

**Independent Test**: Write test that uses TestServer::start_with_fixture() and verifies data is available

### Implementation for User Story 3

- [ ] T027 [US3] Implement TestServer::start_with_fixture() method in src/testing.rs
- [ ] T028 [US3] Implement TestServer::start_with() method for inline Fixture in src/testing.rs
- [ ] T029 [US3] Add MockServerConfig::with_fixture() builder method in src/config.rs
- [ ] T030 [US3] Add MockServerConfig::with_fixture_at() convenience constructor in src/config.rs

**Checkpoint**: User Story 3 complete - tests can use fixtures programmatically

---

## Phase 6: User Story 4 - Fixture Validation and Error Handling (Priority: P3)

**Goal**: Provide clear error messages for invalid or incompatible fixtures

**Independent Test**: Load fixture with wrong version, verify clear error message

### Implementation for User Story 4

- [ ] T031 [US4] Implement version mismatch checking in load_from_file()
- [ ] T032 [US4] Add validation for required fields (version) on fixture load
- [ ] T033 [US4] Return FixtureError::NotFound when file doesn't exist
- [ ] T034 [US4] Return FixtureError::UnsupportedFormat for unknown file extensions
- [ ] T035 [US4] Log warnings for minor version differences

**Checkpoint**: User Story 4 complete - fixtures have proper error handling

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Sample fixtures and integration testing

- [ ] T036 [P] Create fixtures/empty.yaml sample fixture
- [ ] T037 [P] Create fixtures/basic-buckets.yaml sample fixture with 2 buckets
- [ ] T038 [P] Create fixtures/full-project.yaml sample fixture with buckets, objects, projects, issues
- [ ] T039 [P] Create tests/integration/fixtures_test.rs with load/export integration tests
- [ ] T040 Update README.md with fixture usage examples
- [ ] T041 Run cargo fmt and cargo clippy on new files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel after foundation
  - US3 depends on US1 (uses load_from_file internally)
  - US4 depends on US1 (enhances load functionality)
- **Polish (Phase 7)**: Can start after US1 is complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 3 (P2)**: Depends on US1 (load_from_file function)
- **User Story 4 (P3)**: Depends on US1 (validation integrates with loading)

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different files)
- T009, T010, T011 can run in parallel (different files)
- US1 and US2 implementation can proceed in parallel
- T036, T037, T038, T039 can run in parallel (different files)

---

## Parallel Example: Setup Phase

```bash
# Launch all setup tasks together:
Task: "Create src/fixture/types.rs with Fixture struct"
Task: "Create src/fixture/loader.rs with function stubs"
Task: "Create src/fixture/exporter.rs with function stubs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Load)
4. **STOP and VALIDATE**: Test --fixture CLI flag
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add US1 -> Can load fixtures at startup (MVP!)
3. Add US2 -> Can export state to fixtures
4. Add US3 -> Can use fixtures in tests programmatically
5. Add US4 -> Better error messages
6. Each story adds value without breaking previous stories

---

## Notes

- YAML is the primary format, JSON is supported alternative
- Use atomic writes (temp file + rename) to prevent corruption
- All state collections use Vec for serialization
- Version checking uses major version only (1.x is compatible with 1.0)
- Fixture loading clears existing state by default (Replace mode)
- Dependencies: serde_yaml already in Cargo.toml

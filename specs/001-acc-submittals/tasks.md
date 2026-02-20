# Tasks: ACC Submittals Mock Support

**Input**: Design documents from `/specs/001-acc-submittals/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md

**Tests**: Not explicitly requested - tests are optional but recommended.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure for submittals feature

- [ ] T001 Create src/state/submittals.rs with SubmittalInfo struct and SubmittalsState skeleton
- [ ] T002 [P] Create src/handlers/submittals.rs with handler function stubs
- [ ] T003 Add submittals module export in src/state/mod.rs
- [ ] T004 [P] Add submittals handlers module export in src/handlers/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Add SubmittalsState field to StateManager in src/state/manager.rs
- [ ] T006 Implement SubmittalsState::new() constructor in src/state/submittals.rs
- [ ] T007 [P] Add CreateSubmittalRequest and UpdateSubmittalRequest DTOs in src/state/submittals.rs
- [ ] T008 [P] Add SubmittalListResponse DTO in src/state/submittals.rs
- [ ] T009 Register submittal routes in src/server/router.rs under /construction/submittals/v1/projects/:project_id/submittals

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - List Submittals by Project (Priority: P1)

**Goal**: Retrieve a list of submittals for a project with proper response format

**Independent Test**: Call GET /construction/submittals/v1/projects/{project_id}/submittals and verify array response

### Implementation for User Story 1

- [ ] T010 [US1] Implement SubmittalsState::list_submittals() method in src/state/submittals.rs
- [ ] T011 [US1] Implement list_submittals handler function in src/handlers/submittals.rs
- [ ] T012 [US1] Add 404 error handling for invalid project_id in list handler
- [ ] T013 [US1] Return empty array when project has no submittals

**Checkpoint**: User Story 1 complete - can list submittals for any project

---

## Phase 4: User Story 2 - Create a Submittal (Priority: P1)

**Goal**: Create new submittals with auto-generated ID and default status

**Independent Test**: POST valid payload, verify response contains generated ID and status "open"

### Implementation for User Story 2

- [ ] T014 [US2] Implement SubmittalsState::create_submittal() method in src/state/submittals.rs
- [ ] T015 [US2] Add UUID generation for new submittal IDs in create_submittal method
- [ ] T016 [US2] Add timestamp generation (created_at, updated_at) in create_submittal method
- [ ] T017 [US2] Implement create_submittal handler function in src/handlers/submittals.rs
- [ ] T018 [US2] Add 400 validation error when title is missing in create handler
- [ ] T019 [US2] Return 201 Created status code with created submittal in response

**Checkpoint**: User Stories 1 AND 2 complete - can create and list submittals

---

## Phase 5: User Story 3 - Get Submittal Details (Priority: P2)

**Goal**: Retrieve a single submittal by ID with all fields

**Independent Test**: Create submittal, fetch by ID, verify all fields returned

### Implementation for User Story 3

- [ ] T020 [US3] Implement SubmittalsState::get_submittal() method in src/state/submittals.rs
- [ ] T021 [US3] Implement get_submittal handler function in src/handlers/submittals.rs
- [ ] T022 [US3] Add 404 error handling when submittal_id not found

**Checkpoint**: User Story 3 complete - can retrieve individual submittals

---

## Phase 6: User Story 4 - Update Submittal Status (Priority: P2)

**Goal**: Update submittal fields including status transitions

**Independent Test**: Create submittal, PATCH with new status, verify status changed and updated_at refreshed

### Implementation for User Story 4

- [ ] T023 [US4] Implement SubmittalsState::update_submittal() method in src/state/submittals.rs
- [ ] T024 [US4] Update updated_at timestamp on any field change
- [ ] T025 [US4] Implement update_submittal handler function in src/handlers/submittals.rs
- [ ] T026 [US4] Add 404 error handling when submittal_id not found for update
- [ ] T027 [US4] Support partial updates (only update provided fields)

**Checkpoint**: User Story 4 complete - can update submittal fields

---

## Phase 7: User Story 5 - Delete a Submittal (Priority: P3)

**Goal**: Remove a submittal from mock state

**Independent Test**: Create submittal, delete it, verify GET returns 404

### Implementation for User Story 5

- [ ] T028 [US5] Implement SubmittalsState::delete_submittal() method in src/state/submittals.rs
- [ ] T029 [US5] Implement delete_submittal handler function in src/handlers/submittals.rs
- [ ] T030 [US5] Return 204 No Content on successful deletion
- [ ] T031 [US5] Add 404 error handling when submittal_id not found for delete

**Checkpoint**: All user stories complete - full CRUD operations available

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing and cleanup

- [ ] T032 [P] Create tests/integration/submittals_test.rs with CRUD integration tests
- [ ] T033 Add serde Serialize/Deserialize derives to SubmittalInfo for fixture support
- [ ] T034 Verify all handlers follow existing error response format (code, message)
- [ ] T035 Run cargo fmt and cargo clippy on new files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (both P1)
  - US3 and US4 can proceed in parallel (both P2)
  - US5 depends on at least US2 for meaningful testing
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 3 (P2)**: Can start after Phase 2 - Benefits from US2 for test data
- **User Story 4 (P2)**: Can start after Phase 2 - Benefits from US2 for test data
- **User Story 5 (P3)**: Can start after Phase 2 - Benefits from US2 for test data

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different files)
- T007, T008 can run in parallel (different DTOs in same file)
- US1 and US2 can be developed in parallel
- US3 and US4 can be developed in parallel
- T032, T033 can run in parallel

---

## Parallel Example: Setup Phase

```bash
# Launch all setup tasks together:
Task: "Create src/state/submittals.rs with SubmittalInfo struct"
Task: "Create src/handlers/submittals.rs with handler stubs"
Task: "Add submittals module export in src/state/mod.rs"
Task: "Add handlers module export in src/handlers/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (List)
4. Complete Phase 4: User Story 2 (Create)
5. **STOP and VALIDATE**: Test create and list operations
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add US1 + US2 -> Can create and list (MVP!)
3. Add US3 -> Can view individual submittals
4. Add US4 -> Can update submittals
5. Add US5 -> Can delete submittals
6. Each story adds value without breaking previous stories

---

## Notes

- Follow existing issues.rs pattern in src/state/ for implementation
- Use DashMap<String, DashMap<String, SubmittalInfo>> for thread-safe state
- All timestamps are Unix milliseconds (i64)
- Status values are not validated (accepts any string for flexibility)
- Project validation is optional (mock may accept any project_id)

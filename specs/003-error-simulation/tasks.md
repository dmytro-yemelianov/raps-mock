# Tasks: Error and Latency Simulation

**Input**: Design documents from `/specs/003-error-simulation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md

**Tests**: Not explicitly requested - tests are optional but recommended.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create chaos module structure and define core types

- [ ] T001 Create src/chaos/ directory structure with mod.rs
- [ ] T002 [P] Create src/chaos/config.rs with ChaosConfig, LatencyConfig, WeightedErrorCode structs
- [ ] T003 [P] Create src/chaos/middleware.rs with empty middleware stubs
- [ ] T004 [P] Create src/chaos/errors.rs with error response generation stubs
- [ ] T005 [P] Create src/chaos/rate_limiter.rs with RateLimiter skeleton
- [ ] T006 Add chaos module export in src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Implement ChaosConfig::default() with enabled=true, error_rate=0.0 in src/chaos/config.rs
- [ ] T008 Implement default_error_codes() function returning weighted distribution in src/chaos/config.rs
- [ ] T009 Add ChaosConfig field with Arc<RwLock<ChaosConfig>> to MockServerConfig in src/config.rs
- [ ] T010 Create ChaosLayer and ChaosService Tower middleware structures in src/chaos/middleware.rs
- [ ] T011 Wire ChaosLayer into axum router in src/server/router.rs
- [ ] T012 Add --error-rate CLI argument in src/main.rs
- [ ] T013 [P] Add --latency and --jitter CLI arguments in src/main.rs
- [ ] T014 [P] Add --no-chaos CLI argument to disable chaos in src/main.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Simulate Random Errors (Priority: P1)

**Goal**: Inject random errors at a configurable rate for chaos testing

**Independent Test**: Set 50% error rate, make 100 requests, verify ~50 fail with error responses

### Implementation for User Story 1

- [ ] T015 [US1] Implement should_fail(error_rate: f64) -> bool using rand::thread_rng() in src/chaos/middleware.rs
- [ ] T016 [US1] Implement select_error_code() function using weighted random selection in src/chaos/errors.rs
- [ ] T017 [US1] Implement create_error_response(status: StatusCode) -> Response in APS format in src/chaos/errors.rs
- [ ] T018 [US1] Add error injection logic to ChaosService::call() checking error_rate in src/chaos/middleware.rs
- [ ] T019 [US1] Return early with error response when should_fail() returns true
- [ ] T020 [US1] Add verbose logging when error is injected

**Checkpoint**: User Story 1 complete - can inject random errors at configured rate

---

## Phase 4: User Story 2 - Simulate Latency (Priority: P1)

**Goal**: Add configurable delays to all responses for timeout testing

**Independent Test**: Set 500ms latency, make request, verify response takes ~500ms

### Implementation for User Story 2

- [ ] T021 [US2] Implement LatencyConfig struct with base_ms and jitter_ms in src/chaos/config.rs
- [ ] T022 [US2] Implement apply_latency() async function using tokio::time::sleep() in src/chaos/middleware.rs
- [ ] T023 [US2] Add latency injection to ChaosService::call() before forwarding to inner service
- [ ] T024 [US2] Calculate jitter using rand::thread_rng().gen_range(0..=jitter_ms)
- [ ] T025 [US2] Add verbose logging showing actual delay applied

**Checkpoint**: User Stories 1 AND 2 complete - can inject errors and latency

---

## Phase 5: User Story 3 - Fail Specific Endpoints (Priority: P2)

**Goal**: Configure specific endpoints to always fail with configurable status codes

**Independent Test**: Configure /oss/v2/buckets to fail with 503, verify that endpoint fails but others succeed

### Implementation for User Story 3

- [ ] T026 [US3] Implement EndpointFailure struct with pattern, status_code, message in src/chaos/config.rs
- [ ] T027 [US3] Implement path_matches(pattern: &str, path: &str) -> bool glob matching in src/chaos/middleware.rs
- [ ] T028 [US3] Check failing_endpoints before random error injection in ChaosService::call()
- [ ] T029 [US3] Return configured error code and message for matched endpoints
- [ ] T030 [US3] Add --fail-endpoint CLI argument with format "path:code" in src/main.rs

**Checkpoint**: User Story 3 complete - can fail specific endpoints

---

## Phase 6: User Story 4 - Endpoint-Specific Latency (Priority: P2)

**Goal**: Configure different latency for specific endpoints

**Independent Test**: Configure /modelderivative/** with 2s latency, verify only that endpoint is slow

### Implementation for User Story 4

- [ ] T031 [US4] Implement EndpointLatency struct with pattern, base_ms, jitter_ms in src/chaos/config.rs
- [ ] T032 [US4] Check endpoint_latencies before applying global latency
- [ ] T033 [US4] Apply endpoint-specific latency when path matches
- [ ] T034 [US4] Fall back to global latency if no endpoint match

**Checkpoint**: User Story 4 complete - can set per-endpoint latency

---

## Phase 7: User Story 5 - Simulate Rate Limiting (Priority: P3)

**Goal**: Return 429 responses when request rate exceeds configured limit

**Independent Test**: Set 10 req/min limit, make 15 requests, verify last 5 return 429

### Implementation for User Story 5

- [ ] T035 [US5] Implement RateLimitConfig struct with requests_per_window, window_seconds in src/chaos/config.rs
- [ ] T036 [US5] Implement RateLimiter struct with DashMap<String, VecDeque<Instant>> in src/chaos/rate_limiter.rs
- [ ] T037 [US5] Implement RateLimiter::check() method with sliding window algorithm
- [ ] T038 [US5] Add rate limiting check to ChaosService::call() before other chaos
- [ ] T039 [US5] Return 429 response with Retry-After header when limit exceeded
- [ ] T040 [US5] Add X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset headers
- [ ] T041 [US5] Add --rate-limit CLI argument with format "requests/seconds" in src/main.rs

**Checkpoint**: All user stories complete - full chaos engineering capabilities

---

## Phase 8: Runtime Configuration (Cross-Cutting)

**Purpose**: Allow changing chaos settings at runtime without restart

- [ ] T042 Implement MockServer::set_error_rate() method in src/server.rs
- [ ] T043 [P] Implement MockServer::set_latency() method in src/server.rs
- [ ] T044 [P] Implement MockServer::fail_endpoint() method in src/server.rs
- [ ] T045 [P] Implement MockServer::unfail_endpoint() method in src/server.rs
- [ ] T046 [P] Implement MockServer::set_chaos_enabled() method in src/server.rs
- [ ] T047 Implement MockServer::set_rate_limit() and clear_rate_limit() methods in src/server.rs

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Test helpers and integration testing

- [ ] T048 [P] Implement TestServer::start_with_chaos() method in src/testing.rs
- [ ] T049 [P] Implement TestServer::start_with_error_rate() convenience method in src/testing.rs
- [ ] T050 [P] Implement TestServer::start_with_latency() convenience method in src/testing.rs
- [ ] T051 [P] Create tests/integration/chaos_test.rs with error rate tests
- [ ] T052 Add ChaosConfig builder methods (with_error_rate, fail_endpoint, etc.) in src/chaos/config.rs
- [ ] T053 Run cargo fmt and cargo clippy on new files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (both P1, different features)
  - US3 depends on error response generation from US1
  - US4 depends on latency injection from US2
  - US5 can proceed independently after foundation
- **Runtime Config (Phase 8)**: Can start after US1 complete
- **Polish (Phase 9)**: Can start after US1 and US2 complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **User Story 3 (P2)**: Depends on US1 (uses error response generation)
- **User Story 4 (P2)**: Depends on US2 (extends latency logic)
- **User Story 5 (P3)**: Can start after Phase 2 - Independent implementation

### Parallel Opportunities

- T002, T003, T004, T005 can run in parallel (different files)
- T012, T013, T014 can run in parallel (different CLI args)
- US1 and US2 can be developed in parallel
- US3, US4, US5 can be developed in parallel (after their dependencies)
- T042-T047 can largely run in parallel
- T048, T049, T050, T051 can run in parallel

---

## Parallel Example: Setup Phase

```bash
# Launch all setup tasks together:
Task: "Create src/chaos/config.rs with ChaosConfig struct"
Task: "Create src/chaos/middleware.rs with middleware stubs"
Task: "Create src/chaos/errors.rs with error generation stubs"
Task: "Create src/chaos/rate_limiter.rs with RateLimiter skeleton"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Random Errors)
4. Complete Phase 4: User Story 2 (Latency)
5. **STOP and VALIDATE**: Test --error-rate and --latency CLI flags
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add US1 -> Can inject random errors (MVP!)
3. Add US2 -> Can inject latency
4. Add US3 -> Can fail specific endpoints
5. Add US4 -> Can set per-endpoint latency
6. Add US5 -> Can simulate rate limiting
7. Each story adds value without breaking previous stories

---

## Notes

- Chaos middleware runs BEFORE request handlers
- Errors return early without touching state (safe for stateful mode)
- Use Arc<RwLock<ChaosConfig>> for runtime configuration changes
- Error responses match APS error format for realism
- Default error code distribution: 400(15%), 401(15%), 403(15%), 404(15%), 500(20%), 502(10%), 503(10%)
- Rate limiter uses sliding window algorithm for accuracy
- Dependencies: rand already in Cargo.toml

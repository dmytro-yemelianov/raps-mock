# Feature Specification: Request Recording and Playback

**Feature Branch**: `004-recording-mode`
**Created**: 2026-01-15
**Status**: Draft
**Input**: User description: "Record live APS API calls and replay them"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Record API Interactions (Priority: P1)

A developer wants to capture real API interactions from a live session to use as test fixtures. They start the mock server in recording mode with the real APS API as a backend, make API calls through the mock, and the requests and responses are saved to files for later replay.

**Why this priority**: Recording is the foundation of the record/playback feature. Without recording, there's nothing to replay. This enables creating realistic test data from actual API interactions.

**Independent Test**: Can be fully tested by starting the server in record mode, making a few API calls, and verifying that the recordings are saved to the specified directory with request and response data.

**Acceptance Scenarios**:

1. **Given** the server is started in record mode with an output directory, **When** API calls are made through the mock, **Then** each request-response pair is saved to a file.
2. **Given** recording is enabled, **When** a request is made, **Then** the recorded file includes request method, path, headers, and body.
3. **Given** recording is enabled, **When** a response is received from the backend, **Then** the recorded file includes response status, headers, and body.
4. **Given** recording is enabled, **When** multiple requests are made, **Then** each is saved with a sequential identifier for ordering.

---

### User Story 2 - Replay Recorded Sessions (Priority: P1)

A developer wants to use previously recorded API interactions to run tests without connecting to the real APS API. They start the mock server in playback mode pointing to a recordings directory, and requests are matched against recordings to return the corresponding responses.

**Why this priority**: Playback enables the primary use case of offline testing with real-world data. This is equally critical as recording - they form a complete workflow together.

**Independent Test**: Can be tested by creating a recording directory with captured interactions, starting the server in playback mode, and verifying that requests return the recorded responses.

**Acceptance Scenarios**:

1. **Given** a recordings directory with captured interactions, **When** the server starts in playback mode, **Then** requests matching recorded requests return the recorded responses.
2. **Given** playback mode is active, **When** a request matches a recording by method and path, **Then** the corresponding recorded response is returned.
3. **Given** playback mode is active, **When** a request doesn't match any recording, **Then** a 404 error is returned indicating no matching recording found.
4. **Given** playback mode is active, **When** the same request is made multiple times, **Then** the same recorded response is returned each time.

---

### User Story 3 - Anonymize Sensitive Data (Priority: P2)

A developer wants to share recordings with their team without exposing sensitive credentials. They configure the recording to anonymize or redact authentication tokens and other sensitive information.

**Why this priority**: Security is important but depends on basic recording (P1) being functional first. Recordings with real credentials would be a security risk.

**Independent Test**: Can be tested by recording with anonymization enabled, then inspecting the saved files to verify tokens are redacted.

**Acceptance Scenarios**:

1. **Given** anonymization is enabled, **When** a request with an Authorization header is recorded, **Then** the token value is replaced with a placeholder.
2. **Given** anonymization is enabled, **When** a response contains an access_token field, **Then** the token value is replaced with a placeholder.
3. **Given** anonymization is disabled, **When** recordings are made, **Then** all data is preserved as-is.

---

### User Story 4 - Flexible Request Matching (Priority: P2)

A developer wants playback to match requests even when certain parameters vary (e.g., timestamps, request IDs). They configure matching rules to ignore or normalize certain fields.

**Why this priority**: Strict matching often fails due to dynamic fields. Flexible matching makes playback practical for real applications. Depends on basic playback (P1).

**Independent Test**: Can be tested by recording a request, then replaying with a slightly different request (e.g., different timestamp) and verifying it still matches.

**Acceptance Scenarios**:

1. **Given** matching is configured to ignore query parameter "timestamp", **When** a request matches except for that parameter, **Then** the recording is still matched and response returned.
2. **Given** matching is configured to ignore header "X-Request-ID", **When** a request matches except for that header, **Then** the recording is still matched.
3. **Given** strict matching is configured (default), **When** any part of the request differs, **Then** no match is found.

---

### User Story 5 - Sequential Playback for Stateful Workflows (Priority: P3)

A developer wants to replay a sequence of API calls that depend on each other (e.g., create then update). They record a workflow session, and during playback, requests are matched in sequence to support stateful scenarios.

**Why this priority**: Advanced use case for complex workflows. Basic playback (P1) handles most use cases; sequential playback adds sophistication.

**Independent Test**: Can be tested by recording a multi-step workflow, then replaying and verifying each step returns the appropriate response in order.

**Acceptance Scenarios**:

1. **Given** a recording with a sequence of create-then-update calls, **When** replaying in sequence mode, **Then** the first call returns the create response and the second returns the update response.
2. **Given** sequential mode is active, **When** calls are made out of order, **Then** an error indicates the expected sequence.

---

### Edge Cases

- What happens when the recording directory is not writable? The system returns an error at startup and refuses to enter record mode.
- What happens when a recording file is corrupted? The system skips that recording and logs a warning, continuing with other recordings.
- What happens when disk space runs out during recording? The system stops recording new interactions and logs an error, but continues proxying requests.
- What happens when the real APS API is unreachable during recording? The system returns an error to the client and records the error response.
- What happens when request bodies are very large (e.g., file uploads)? The system records metadata only for large bodies, with an option to skip body recording.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support recording mode that captures request-response pairs while proxying to a real backend
- **FR-002**: System MUST support playback mode that returns recorded responses without connecting to a backend
- **FR-003**: System MUST save recordings to files in a structured format (one file per interaction or grouped by session)
- **FR-004**: System MUST include request method, path, headers, and body in recordings
- **FR-005**: System MUST include response status, headers, and body in recordings
- **FR-006**: System MUST support anonymizing sensitive data (tokens, credentials) in recordings
- **FR-007**: System MUST support configurable request matching strategies (strict vs. flexible)
- **FR-008**: System MUST preserve request ordering when recording a session
- **FR-009**: System MUST support sequential playback for stateful workflow testing
- **FR-010**: System MUST handle large request/response bodies gracefully (configurable size limits)
- **FR-011**: System MUST provide clear error messages when playback cannot find a matching recording

### Key Entities

- **Recording**: A captured request-response interaction. Key attributes: id, timestamp, request (method, path, headers, body), response (status, headers, body), metadata (session_id, sequence_number).
- **Recording Session**: A group of recordings captured together. Key attributes: session_id, start_time, end_time, recording_count, backend_url.
- **Matching Rules**: Configuration for how to match incoming requests to recordings. Key attributes: ignored_headers, ignored_query_params, body_comparison_mode.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can capture a real API workflow and replay it offline with 100% response fidelity
- **SC-002**: Recorded sessions can be shared between team members without exposing credentials (when anonymization is enabled)
- **SC-003**: Playback matches requests accurately with less than 1% false negatives when using flexible matching
- **SC-004**: Recording adds less than 50ms overhead to request latency
- **SC-005**: A typical workflow recording (50 requests) can be loaded and ready for playback in under 500ms

## Assumptions

- The backend URL for recording mode is provided at server startup and cannot change during a session
- Recordings are stored locally; cloud storage is out of scope for initial implementation
- Request matching uses method + path as primary keys; headers and body are secondary matching criteria
- Binary request/response bodies (file uploads, downloads) are stored with size limits and optional exclusion
- Anonymization targets common patterns (Authorization header, access_token field) but is configurable

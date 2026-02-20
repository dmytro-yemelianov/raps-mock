# Feature Specification: Error and Latency Simulation

**Feature Branch**: `003-error-simulation`
**Created**: 2026-01-15
**Status**: Draft
**Input**: User description: "Add configurable error responses and latency simulation"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Simulate Random Errors (Priority: P1)

A developer wants to test how their application handles API failures. They configure the mock server with an error rate (e.g., 5% of requests fail), and random requests return error responses, allowing them to verify their error handling and retry logic.

**Why this priority**: Testing error handling is the primary use case for chaos engineering. Without this, developers cannot verify their applications are resilient to API failures.

**Independent Test**: Can be fully tested by starting the server with an error rate, making multiple requests, and verifying that approximately the configured percentage fail with appropriate error responses.

**Acceptance Scenarios**:

1. **Given** an error rate of 10% configured, **When** 100 requests are made, **Then** approximately 10 requests return error responses (within statistical variance).
2. **Given** an error rate of 0% (default), **When** requests are made, **Then** no artificial errors are injected.
3. **Given** an error rate of 100%, **When** any request is made, **Then** all requests return error responses.
4. **Given** random errors are enabled, **When** an error occurs, **Then** the response includes a valid error code (400, 401, 403, 404, 500, 502, 503) and error message.

---

### User Story 2 - Simulate Latency (Priority: P1)

A developer wants to test how their application handles slow API responses. They configure the mock server with latency settings, and all responses are delayed by the specified amount, allowing them to verify timeout handling and user experience under degraded conditions.

**Why this priority**: Latency testing is equally important as error testing - many real-world issues stem from slow responses, not failures. This is a core chaos engineering capability.

**Independent Test**: Can be tested by starting the server with latency configured, making a request, and verifying the response takes at least the configured time.

**Acceptance Scenarios**:

1. **Given** a base latency of 500ms configured, **When** a request is made, **Then** the response is delayed by at least 500ms.
2. **Given** latency with jitter of 100ms, **When** multiple requests are made, **Then** response times vary randomly within the jitter range.
3. **Given** no latency configured (default), **When** requests are made, **Then** responses are returned as fast as possible.
4. **Given** latency is configured, **When** verbose logging is enabled, **Then** logs show the actual delay applied to each request.

---

### User Story 3 - Fail Specific Endpoints (Priority: P2)

A developer wants to test error handling for a specific endpoint without affecting others. They configure the server to always fail requests to a particular path pattern, simulating a partial outage scenario.

**Why this priority**: Targeted failures are more realistic than random errors and allow testing specific error handling paths. Depends on basic error injection (P1) being established.

**Independent Test**: Can be tested by configuring a specific endpoint to fail, making requests to that endpoint and others, and verifying only the targeted endpoint returns errors.

**Acceptance Scenarios**:

1. **Given** endpoint `/oss/v2/buckets` configured to fail, **When** a request is made to that endpoint, **Then** an error response is returned.
2. **Given** endpoint `/oss/v2/buckets` configured to fail, **When** a request is made to `/projects/v1/hubs`, **Then** a normal response is returned.
3. **Given** a pattern like `/oss/v2/buckets/*` configured to fail, **When** a request matches the pattern, **Then** an error is returned.
4. **Given** a specific error code (e.g., 503) configured for an endpoint, **When** a request is made, **Then** that specific error code is returned.

---

### User Story 4 - Endpoint-Specific Latency (Priority: P2)

A developer wants to simulate a specific slow endpoint (e.g., Model Derivative translations are slow). They configure different latency for specific endpoints.

**Why this priority**: Realistic simulations often require different latencies for different operations. Depends on base latency (P1) being established.

**Independent Test**: Can be tested by configuring high latency for one endpoint and low latency for another, making requests to both, and verifying different delays.

**Acceptance Scenarios**:

1. **Given** endpoint `/modelderivative/v2/designdata/:urn/manifest` configured with 2s latency, **When** a request is made to that endpoint, **Then** response is delayed by 2s.
2. **Given** specific endpoint latency configured, **When** a request is made to a different endpoint, **Then** the default (or no) latency is applied.

---

### User Story 5 - Simulate Rate Limiting (Priority: P3)

A developer wants to test how their application handles rate limiting responses. They configure the mock server with a request limit per time window, and once exceeded, subsequent requests receive 429 responses.

**Why this priority**: Rate limiting is important but less common than error/latency testing. Can be implemented after core chaos features.

**Independent Test**: Can be tested by setting a low rate limit, making requests until the limit is exceeded, and verifying 429 responses are returned.

**Acceptance Scenarios**:

1. **Given** a rate limit of 10 requests per minute, **When** 11 requests are made within a minute, **Then** the 11th request returns 429 Too Many Requests.
2. **Given** rate limiting is triggered, **When** the time window resets, **Then** requests succeed again.
3. **Given** rate limiting is configured, **When** a 429 response is returned, **Then** it includes appropriate headers (Retry-After).

---

### Edge Cases

- What happens when both error rate and specific endpoint failures are configured for the same endpoint? Specific endpoint configuration takes precedence over random error rate.
- What happens when latency is very high (e.g., 30 seconds)? The request is delayed but eventually completes (no artificial timeout imposed by the mock).
- What happens when error rate is set to a non-numeric value? Validation error at startup, server refuses to start.
- What happens when jitter exceeds base latency? Jitter is applied additively, so response time = base + random(0, jitter).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support configuring a global error rate (0-100%) for random failures
- **FR-002**: System MUST support configuring base latency (in milliseconds) applied to all responses
- **FR-003**: System MUST support configuring latency jitter for realistic variance
- **FR-004**: System MUST support configuring specific endpoints to always fail
- **FR-005**: System MUST support configuring specific error codes for targeted failures
- **FR-006**: System MUST support per-endpoint latency overrides
- **FR-007**: System MUST support rate limiting with configurable limits and time windows
- **FR-008**: System MUST return valid error response bodies with appropriate messages
- **FR-009**: System MUST support configuring chaos settings via command-line options
- **FR-010**: System MUST support configuring chaos settings programmatically via library API
- **FR-011**: System MUST log chaos events (injected errors, applied latency) when verbose mode is enabled
- **FR-012**: System MUST apply latency before processing the request, not after (to avoid state changes on failed requests)

### Key Entities

- **Chaos Configuration**: Settings that control error injection behavior. Key attributes: error_rate (percentage), error_codes (list of possible codes), failing_endpoints (list of patterns).
- **Latency Configuration**: Settings that control response timing. Key attributes: base_latency_ms, jitter_ms, endpoint_overrides (map of pattern to latency).
- **Rate Limit Configuration**: Settings that control request throttling. Key attributes: requests_per_window, window_seconds, per_endpoint_limits.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can verify their error handling logic by triggering controlled failures without modifying application code
- **SC-002**: Configured error rates produce statistically accurate failure distributions (within 5% variance over 100+ requests)
- **SC-003**: Latency injection adds the configured delay consistently (within 10ms variance)
- **SC-004**: Applications can test timeout handling by configuring latency higher than their timeout threshold
- **SC-005**: Chaos configuration can be changed without restarting the server (for programmatic API)

## Assumptions

- Error codes returned are standard HTTP error codes that APS APIs would return (400, 401, 403, 404, 500, 502, 503)
- Random number generation uses a standard pseudo-random generator; cryptographic randomness is not required
- Latency is applied at the middleware level before request processing
- Chaos configuration does not persist across server restarts (ephemeral by default)
- Rate limiting counts are per-server-instance, not distributed

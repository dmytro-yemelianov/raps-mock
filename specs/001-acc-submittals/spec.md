# Feature Specification: ACC Submittals Mock Support

**Feature Branch**: `001-acc-submittals`
**Created**: 2026-01-15
**Status**: Draft
**Input**: User description: "Add stateful mock support for ACC Submittals API"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List Submittals by Project (Priority: P1)

A developer testing their APS integration wants to retrieve a list of submittals for a construction project. They call the mock server's submittals list endpoint and receive a collection of submittal records that match the expected ACC API response format.

**Why this priority**: Listing submittals is the most common operation and the foundation for all other submittal workflows. Without this, users cannot verify their integration displays submittals correctly.

**Independent Test**: Can be fully tested by calling GET on the submittals endpoint and verifying the response contains a valid array of submittal objects with required fields.

**Acceptance Scenarios**:

1. **Given** a project with existing submittals in state, **When** a user requests the submittals list, **Then** all submittals for that project are returned with id, title, number, status, and dates.
2. **Given** a project with no submittals, **When** a user requests the submittals list, **Then** an empty array is returned with 200 status.
3. **Given** an invalid project ID, **When** a user requests the submittals list, **Then** a 404 error is returned.

---

### User Story 2 - Create a Submittal (Priority: P1)

A developer testing submittal creation wants to verify their application can create new submittals. They POST to the submittals endpoint with title and required fields, and the mock server creates a new submittal in memory and returns it with a generated ID.

**Why this priority**: Creating submittals is essential for testing workflows where users initiate the submittal process. This is equally critical as listing.

**Independent Test**: Can be tested by POSTing a valid submittal payload and verifying the response contains the created submittal with a unique ID and default status.

**Acceptance Scenarios**:

1. **Given** a valid project ID and submittal payload with title, **When** a user creates a submittal, **Then** a new submittal is created with generated ID, status "open", and current timestamp.
2. **Given** a project ID and submittal payload with optional fields (spec_section, due_date, description), **When** a user creates a submittal, **Then** all provided fields are stored and returned.
3. **Given** an invalid project ID, **When** a user attempts to create a submittal, **Then** a 404 error is returned.
4. **Given** a payload missing required field (title), **When** a user attempts to create a submittal, **Then** a 400 error is returned with validation message.

---

### User Story 3 - Get Submittal Details (Priority: P2)

A developer testing their detail view wants to retrieve a single submittal by ID to display its full information.

**Why this priority**: Viewing individual submittals is required for detail pages but depends on having submittals to view (P1 stories).

**Independent Test**: Can be tested by creating a submittal, then fetching it by ID and verifying all fields are returned correctly.

**Acceptance Scenarios**:

1. **Given** an existing submittal, **When** a user requests it by ID, **Then** the complete submittal object is returned.
2. **Given** a non-existent submittal ID, **When** a user requests it, **Then** a 404 error is returned.

---

### User Story 4 - Update Submittal Status (Priority: P2)

A developer testing workflow transitions wants to update a submittal's status (e.g., from "open" to "submitted" to "approved").

**Why this priority**: Status updates are core to submittal workflows but depend on having submittals to update.

**Independent Test**: Can be tested by creating a submittal, updating its status via PATCH, and verifying the change persists.

**Acceptance Scenarios**:

1. **Given** an existing submittal with status "open", **When** a user updates status to "submitted", **Then** the status is changed and updated_at timestamp is refreshed.
2. **Given** a non-existent submittal ID, **When** a user attempts to update status, **Then** a 404 error is returned.

---

### User Story 5 - Delete a Submittal (Priority: P3)

A developer testing cleanup operations wants to remove a submittal from the mock state.

**Why this priority**: Deletion is less common in typical workflows but needed for test cleanup.

**Independent Test**: Can be tested by creating a submittal, deleting it, and verifying it no longer appears in list.

**Acceptance Scenarios**:

1. **Given** an existing submittal, **When** a user deletes it, **Then** it is removed from state and subsequent GET returns 404.
2. **Given** a non-existent submittal ID, **When** a user attempts to delete, **Then** a 404 error is returned.

---

### Edge Cases

- What happens when the same submittal number is used twice within a project? The system accepts it (numbers are optional user-provided references, not unique keys).
- How does the system handle very long titles or descriptions? The system accepts them without truncation (matching APS API behavior).
- What happens when filtering by status that doesn't exist? The system returns an empty array.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide an endpoint to list all submittals for a given project ID
- **FR-002**: System MUST provide an endpoint to create a new submittal with at minimum a title
- **FR-003**: System MUST auto-generate unique IDs for created submittals
- **FR-004**: System MUST store submittals in memory indexed by project ID
- **FR-005**: System MUST provide an endpoint to retrieve a single submittal by project ID and submittal ID
- **FR-006**: System MUST provide an endpoint to update submittal fields (title, status, description, spec_section, due_date)
- **FR-007**: System MUST provide an endpoint to delete a submittal by ID
- **FR-008**: System MUST track created_at and updated_at timestamps for each submittal
- **FR-009**: System MUST return appropriate error responses (404 for not found, 400 for validation errors)
- **FR-010**: System MUST isolate submittals state per TestServer instance (no state leakage between test runs)

### Key Entities

- **Submittal**: A construction document requiring review and approval. Key attributes: id, title, number (optional user reference), status, spec_section (specification section reference), description, due_date, created_at, updated_at, project_id.
- **Project**: Parent container for submittals (already exists in projects state manager). Relationship: A project contains zero or more submittals.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can create, read, update, and delete submittals through the mock server without any external dependencies
- **SC-002**: All submittal endpoints respond in under 50ms for typical payloads
- **SC-003**: Mock responses match the structure expected by applications consuming ACC Submittals API
- **SC-004**: Integration tests using TestServer can create isolated submittal state that doesn't affect other test runs
- **SC-005**: Submittal state persists correctly across multiple requests within the same server session

## Assumptions

- Submittal status values follow ACC conventions: "open", "submitted", "approved", "rejected", "closed" (but mock accepts any string for flexibility)
- The submittal number field is an optional user-provided reference, not a system-generated unique identifier
- Authentication is handled by existing auth middleware; this feature assumes valid tokens
- The existing project state manager provides project validation; submittals reference existing project IDs

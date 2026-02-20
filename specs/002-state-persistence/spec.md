# Feature Specification: State Persistence and Fixtures

**Feature Branch**: `002-state-persistence`
**Created**: 2026-01-15
**Status**: Draft
**Input**: User description: "Add ability to save/load mock server state to disk"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load Fixture at Startup (Priority: P1)

A developer wants to start the mock server with pre-configured test data (buckets, projects, issues, etc.) so they don't have to manually create resources before each testing session. They specify a fixture file when starting the server, and all the defined resources are immediately available.

**Why this priority**: Loading fixtures is the primary use case - it enables reproducible test scenarios and saves significant setup time. This is the core value proposition of state persistence.

**Independent Test**: Can be fully tested by creating a fixture file, starting the server with that fixture, and verifying the resources defined in the fixture are accessible via API calls.

**Acceptance Scenarios**:

1. **Given** a valid fixture file containing buckets and projects, **When** the server starts with that fixture, **Then** all defined resources are immediately accessible via their respective endpoints.
2. **Given** a fixture file with nested data (buckets with objects, projects with issues), **When** the server loads the fixture, **Then** all parent-child relationships are preserved.
3. **Given** a malformed fixture file, **When** the server attempts to load it, **Then** an error message is displayed and the server starts with empty state.
4. **Given** a non-existent fixture file path, **When** the server attempts to load it, **Then** an error message is displayed indicating the file was not found.

---

### User Story 2 - Export Current State (Priority: P1)

A developer has created several resources through API calls and wants to save the current server state as a fixture for future use. They export the state to a file, which can later be loaded to recreate the exact same test environment.

**Why this priority**: Export complements import - together they enable the fixture workflow. Without export, users would have to manually write fixture files, which defeats the purpose.

**Independent Test**: Can be tested by creating resources via API, exporting state, then loading the exported file and verifying all resources are restored.

**Acceptance Scenarios**:

1. **Given** a server with resources created via API, **When** a user exports state, **Then** a fixture file is created containing all current resources.
2. **Given** an export is performed, **When** the exported file is loaded on a fresh server, **Then** the state matches the original server's state.
3. **Given** a server with no resources, **When** a user exports state, **Then** a valid fixture file is created with empty collections.

---

### User Story 3 - Programmatic Fixture Loading (Priority: P2)

A developer writing integration tests wants to load fixtures programmatically within their test code, so each test can start with specific pre-configured state without relying on command-line arguments.

**Why this priority**: Library users need programmatic control. This is essential for integration testing workflows but depends on the fixture format being established (P1).

**Independent Test**: Can be tested by writing a test that creates a MockServer, loads a fixture programmatically, and verifies resources are available.

**Acceptance Scenarios**:

1. **Given** a MockServer instance and a fixture file, **When** a user calls the load_fixture method, **Then** the fixture is loaded into the server's state.
2. **Given** an already-populated server state, **When** a user loads a fixture, **Then** the fixture data is merged with or replaces existing state (based on configuration).
3. **Given** an invalid fixture file, **When** a user calls load_fixture, **Then** an error is returned without crashing the server.

---

### User Story 4 - List Available Fixtures (Priority: P3)

A developer wants to see what fixture files are available in a given directory, so they can choose which scenario to load.

**Why this priority**: Convenience feature that improves discoverability but not essential for core functionality.

**Independent Test**: Can be tested by placing fixture files in a directory, running the list command, and verifying all fixtures are displayed with names and descriptions.

**Acceptance Scenarios**:

1. **Given** a directory with multiple fixture files, **When** a user lists fixtures, **Then** all fixture names and descriptions are displayed.
2. **Given** an empty directory, **When** a user lists fixtures, **Then** a message indicates no fixtures are available.

---

### Edge Cases

- What happens when a fixture references an entity that depends on another entity not in the fixture? The system loads what it can and logs warnings for unresolvable references.
- What happens when loading a fixture with a different schema version than the server supports? The system attempts migration for minor version differences, fails gracefully for major version mismatches.
- What happens if the state file is corrupted mid-write during export? The system uses atomic writes (write to temp, then rename) to prevent corruption.
- What happens when disk is full during export? The system returns an error with a clear message about insufficient disk space.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support loading state from a fixture file at server startup via command-line option
- **FR-002**: System MUST support exporting current state to a fixture file via command-line command or API
- **FR-003**: System MUST support YAML format for fixture files (human-readable, editable)
- **FR-004**: System MUST support JSON format for fixture files (machine-friendly, compact)
- **FR-005**: System MUST include a version field in fixture files for forward compatibility
- **FR-006**: System MUST validate fixture files against the expected schema before loading
- **FR-007**: System MUST provide clear error messages when fixture loading fails
- **FR-008**: System MUST support loading fixtures programmatically via library API
- **FR-009**: System MUST preserve all resource relationships when exporting/importing (e.g., objects belong to buckets)
- **FR-010**: System MUST support atomic writes when exporting to prevent file corruption
- **FR-011**: System MUST allow fixtures to include optional metadata (name, description, author)

### Key Entities

- **Fixture**: A portable snapshot of server state. Key attributes: version, name (optional), description (optional), and collections for each resource type (buckets, objects, projects, issues, translations, webhooks, submittals).
- **Fixture Metadata**: Descriptive information about a fixture. Key attributes: name, description, author, created_at.
- **State Manager**: The existing component that holds all in-memory state. Relationship: Fixture serializes/deserializes the StateManager's data.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can save complete server state to disk and restore it on a fresh server with 100% data fidelity
- **SC-002**: Fixture files are human-readable and can be manually edited to create custom test scenarios
- **SC-003**: Loading a typical fixture (100 resources) completes in under 1 second
- **SC-004**: Exported fixtures from one version of raps-mock can be imported into future versions (forward compatibility)
- **SC-005**: Integration tests can set up specific state in under 5 lines of code using programmatic fixture loading

## Assumptions

- YAML is the preferred format for hand-edited fixtures; JSON is preferred for programmatic generation
- The fixture schema will evolve; the version field enables migration logic in future releases
- State persistence does not include real file uploads (only metadata); binary content is out of scope
- Fixture loading replaces existing state by default; merge behavior may be added in future versions

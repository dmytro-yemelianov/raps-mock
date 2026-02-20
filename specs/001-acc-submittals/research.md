# Research: ACC Submittals Mock Support

**Feature**: 001-acc-submittals
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing ACC Submittals mock support. This is a straightforward feature following established patterns.

---

## Research Topics

### 1. ACC Submittals API Structure

**Question**: What is the API structure for ACC Submittals?

**Decision**: Follow the standard ACC API pattern with project-scoped endpoints.

**Rationale**:
- ACC APIs consistently use `/construction/submittals/v1/projects/{project_id}/submittals`
- CRUD operations follow REST conventions
- This matches the pattern used for Issues (`/construction/issues/v1/projects/{project_id}/issues`)

**API Endpoints**:
```
GET    /construction/submittals/v1/projects/{project_id}/submittals
POST   /construction/submittals/v1/projects/{project_id}/submittals
GET    /construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}
PATCH  /construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}
DELETE /construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}
```

---

### 2. State Manager Pattern

**Question**: How should submittals state be managed?

**Decision**: Use the existing `IssuesState` pattern with DashMap.

**Rationale**:
- `IssuesState` is the closest analog (ACC module, project-scoped, CRUD operations)
- DashMap provides thread-safe concurrent access without explicit locking
- Nested DashMap structure: `project_id -> submittal_id -> SubmittalInfo`

**Implementation Pattern** (from issues.rs):
```rust
pub struct SubmittalsState {
    submittals: DashMap<String, DashMap<String, SubmittalInfo>>,
}
```

---

### 3. Submittal Data Model

**Question**: What fields should SubmittalInfo contain?

**Decision**: Include all fields documented in ACC Submittals API.

**Rationale**:
- Must match real API response structure for test fidelity
- Include both required and optional fields
- Timestamps track creation and modification

**Fields**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String | Yes | Auto-generated UUID |
| project_id | String | Yes | Parent project reference |
| title | String | Yes | Submittal title |
| number | Option<String> | No | User-provided reference number |
| status | String | Yes | Workflow status (default: "open") |
| description | Option<String> | No | Detailed description |
| spec_section | Option<String> | No | Specification section reference |
| due_date | Option<String> | No | ISO 8601 date |
| created_at | i64 | Yes | Unix timestamp (milliseconds) |
| updated_at | i64 | Yes | Unix timestamp (milliseconds) |

---

### 4. Handler Integration

**Question**: How should handlers be integrated with the router?

**Decision**: Add hardcoded routes in router.rs following existing ACC pattern.

**Rationale**:
- No OpenAPI spec exists for Submittals yet in aps-sdk-openapi
- Hardcoded routes (like existing Issues) provide reliable behavior
- Routes registered with StateManager access via axum Extension

**Integration Points**:
1. `src/state/mod.rs` - Export submittals module
2. `src/state/manager.rs` - Add `submittals: Arc<SubmittalsState>` field
3. `src/server/router.rs` - Register hardcoded routes
4. `src/handlers/submittals.rs` - New handler implementations

---

### 5. Error Handling

**Question**: What error responses should submittals endpoints return?

**Decision**: Follow existing error response patterns.

**Rationale**:
- Consistency with other mock endpoints
- Match APS error response format

**Error Responses**:
| Scenario | Status | Body |
|----------|--------|------|
| Project not found | 404 | `{"code": "NOT_FOUND", "message": "Project not found"}` |
| Submittal not found | 404 | `{"code": "NOT_FOUND", "message": "Submittal not found"}` |
| Missing required field | 400 | `{"code": "BAD_REQUEST", "message": "title is required"}` |
| Invalid JSON | 400 | `{"code": "BAD_REQUEST", "message": "Invalid request body"}` |

---

## Integration with Existing Code

### StateManager Changes

```rust
// In src/state/manager.rs
pub struct StateManager {
    pub auth: Arc<AuthState>,
    pub buckets: Arc<BucketsState>,
    pub objects: Arc<ObjectsState>,
    pub projects: Arc<ProjectsState>,
    pub translations: Arc<TranslationsState>,
    pub issues: Arc<IssuesState>,
    pub webhooks: Arc<WebhooksState>,
    pub submittals: Arc<SubmittalsState>,  // NEW
}
```

### Router Registration

```rust
// In src/server/router.rs - add to hardcoded_routes()
.route(
    "/construction/submittals/v1/projects/:project_id/submittals",
    get(list_submittals).post(create_submittal),
)
.route(
    "/construction/submittals/v1/projects/:project_id/submittals/:submittal_id",
    get(get_submittal).patch(update_submittal).delete(delete_submittal),
)
```

---

## Dependencies

### No New Dependencies Required

All required crates are already in Cargo.toml:
- `dashmap` - Concurrent hashmap
- `uuid` - ID generation
- `chrono` - Timestamps
- `serde`/`serde_json` - Serialization
- `axum` - HTTP handlers

---

## Summary of Decisions

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| API Structure | Standard ACC pattern | Consistency with Issues |
| State Storage | Nested DashMap | Thread-safe, established pattern |
| Data Model | Full ACC fields | Test fidelity |
| Handler Integration | Hardcoded routes | No OpenAPI spec available |
| Error Handling | Existing patterns | Consistency |

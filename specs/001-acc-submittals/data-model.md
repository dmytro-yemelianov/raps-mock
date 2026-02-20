# Data Model: ACC Submittals Mock Support

**Feature**: 001-acc-submittals
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document defines the data structures for ACC Submittals mock support. The model follows established patterns from the Issues state manager.

---

## Core Entities

### SubmittalInfo

A construction submittal requiring review and approval.

```
SubmittalInfo
├── id: String (UUID)
├── project_id: String
├── title: String
├── number: Option<String>
├── status: String
├── description: Option<String>
├── spec_section: Option<String>
├── due_date: Option<String>
├── created_at: i64
└── updated_at: i64
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Auto-generated UUID v4 |
| `project_id` | String | Yes | Parent project identifier |
| `title` | String | Yes | Submittal title/name |
| `number` | String | No | User-assigned reference number |
| `status` | String | Yes | Workflow status (default: "open") |
| `description` | String | No | Detailed description |
| `spec_section` | String | No | Specification section reference (e.g., "03 30 00") |
| `due_date` | String | No | Due date in ISO 8601 format |
| `created_at` | i64 | Yes | Creation timestamp (Unix ms) |
| `updated_at` | i64 | Yes | Last update timestamp (Unix ms) |

**Status Values** (not enforced, for reference):
- `open` - Initial state
- `submitted` - Sent for review
- `approved` - Approved by reviewer
- `rejected` - Rejected by reviewer
- `closed` - Workflow complete

---

### SubmittalsState

State manager for submittals, indexed by project.

```
SubmittalsState
└── submittals: DashMap<String, DashMap<String, SubmittalInfo>>
                       └── project_id  └── submittal_id
```

**Structure**:
- Outer map: `project_id` → inner map
- Inner map: `submittal_id` → `SubmittalInfo`

**Thread Safety**: All operations are thread-safe via DashMap.

---

## Request/Response DTOs

### CreateSubmittalRequest

```
CreateSubmittalRequest
├── title: String (required)
├── number: Option<String>
├── description: Option<String>
├── spec_section: Option<String>
└── due_date: Option<String>
```

### UpdateSubmittalRequest

```
UpdateSubmittalRequest
├── title: Option<String>
├── number: Option<String>
├── status: Option<String>
├── description: Option<String>
├── spec_section: Option<String>
└── due_date: Option<String>
```

### SubmittalResponse

Same as `SubmittalInfo` - returned from all endpoints.

### SubmittalListResponse

```
SubmittalListResponse
├── pagination: PaginationInfo (optional, for future)
└── results: Vec<SubmittalInfo>
```

---

## Relationships

```
Project 1──* Submittal
   │
   └── Already exists in ProjectsState
```

**Notes**:
- Submittals are scoped to projects
- Project validation is optional (mock may accept any project_id)
- No cascading delete (deleting project doesn't delete submittals)

---

## State Operations

### SubmittalsState Methods

```rust
impl SubmittalsState {
    /// Create a new submittals state
    pub fn new() -> Self;

    /// Create a submittal in a project
    pub fn create_submittal(
        &self,
        project_id: String,
        title: String,
        number: Option<String>,
        description: Option<String>,
        spec_section: Option<String>,
        due_date: Option<String>,
    ) -> SubmittalInfo;

    /// Get a submittal by ID
    pub fn get_submittal(
        &self,
        project_id: &str,
        submittal_id: &str,
    ) -> Option<SubmittalInfo>;

    /// List all submittals for a project
    pub fn list_submittals(&self, project_id: &str) -> Vec<SubmittalInfo>;

    /// Update a submittal's fields
    pub fn update_submittal(
        &self,
        project_id: &str,
        submittal_id: &str,
        title: Option<String>,
        number: Option<String>,
        status: Option<String>,
        description: Option<String>,
        spec_section: Option<String>,
        due_date: Option<String>,
    ) -> Option<SubmittalInfo>;

    /// Delete a submittal
    pub fn delete_submittal(
        &self,
        project_id: &str,
        submittal_id: &str,
    ) -> bool;
}
```

---

## Validation Rules

1. **SubmittalInfo.id**: Must be valid UUID v4 (auto-generated)
2. **SubmittalInfo.title**: Must not be empty
3. **SubmittalInfo.status**: No validation (accepts any string)
4. **SubmittalInfo.due_date**: Should be ISO 8601 format if provided
5. **SubmittalInfo.created_at**: Auto-set on creation, immutable
6. **SubmittalInfo.updated_at**: Auto-updated on any modification

---

## JSON Examples

### SubmittalInfo Response

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "project_id": "project-001",
  "title": "HVAC Shop Drawings",
  "number": "SUB-001",
  "status": "submitted",
  "description": "Shop drawings for Level 1 HVAC systems",
  "spec_section": "23 00 00",
  "due_date": "2026-02-15",
  "created_at": 1736931600000,
  "updated_at": 1736935200000
}
```

### CreateSubmittalRequest

```json
{
  "title": "Structural Steel Details",
  "number": "SUB-002",
  "description": "Detailed drawings for steel framing",
  "spec_section": "05 12 00"
}
```

### SubmittalListResponse

```json
{
  "results": [
    {
      "id": "submittal-001",
      "project_id": "project-001",
      "title": "HVAC Shop Drawings",
      "status": "open",
      "created_at": 1736931600000,
      "updated_at": 1736931600000
    },
    {
      "id": "submittal-002",
      "project_id": "project-001",
      "title": "Structural Steel Details",
      "status": "submitted",
      "created_at": 1736932000000,
      "updated_at": 1736935200000
    }
  ]
}
```

---

## Rust Implementation

```rust
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmittalInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct SubmittalsState {
    submittals: DashMap<String, DashMap<String, SubmittalInfo>>,
}

impl Default for SubmittalsState {
    fn default() -> Self {
        Self::new()
    }
}
```

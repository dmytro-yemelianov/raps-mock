# Data Model: State Persistence and Fixtures

**Feature**: 002-state-persistence
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document defines the data structures for state persistence and fixtures.

---

## Core Entities

### Fixture

A portable snapshot of server state.

```
Fixture
├── version: String
├── metadata: Option<FixtureMetadata>
├── buckets: Vec<BucketInfo>
├── objects: Vec<ObjectInfo>
├── projects: Vec<ProjectInfo>
├── issues: Vec<IssueInfo>
├── translations: Vec<TranslationInfo>
├── webhooks: Vec<WebhookInfo>
└── submittals: Vec<SubmittalInfo>
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | String | Yes | Schema version (e.g., "1.0") |
| `metadata` | FixtureMetadata | No | Descriptive information |
| `buckets` | Vec | No | OSS buckets |
| `objects` | Vec | No | OSS objects |
| `projects` | Vec | No | Data Management projects |
| `issues` | Vec | No | ACC issues |
| `translations` | Vec | No | Model Derivative jobs |
| `webhooks` | Vec | No | Webhook subscriptions |
| `submittals` | Vec | No | ACC submittals |

**Notes**:
- All collections are optional (empty if not provided)
- Additional collections can be added in future versions

---

### FixtureMetadata

Descriptive information about a fixture.

```
FixtureMetadata
├── name: Option<String>
├── description: Option<String>
├── author: Option<String>
└── created_at: Option<String>
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | No | Display name |
| `description` | String | No | What this fixture represents |
| `author` | String | No | Who created it |
| `created_at` | String | No | ISO 8601 timestamp |

---

## Existing State Entities (Reference)

These entities already exist in the codebase. For fixtures, they need `Serialize` and `Deserialize` derives.

### BucketInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct BucketInfo {
    pub bucket_key: String,
    pub policy_key: String,
    pub region: Option<String>,
    pub created_date: Option<String>,
}
```

### ObjectInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct ObjectInfo {
    pub bucket_key: String,
    pub object_key: String,
    pub size: u64,
    pub sha1: String,
    pub content_type: Option<String>,
}
```

### ProjectInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub hub_id: String,
}
```

### IssueInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct IssueInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: i64,
}
```

### TranslationInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct TranslationInfo {
    pub urn: String,
    pub status: String,
    pub progress: Option<String>,
}
```

### WebhookInfo

```rust
#[derive(Serialize, Deserialize)]
pub struct WebhookInfo {
    pub id: String,
    pub callback_url: String,
    pub event: String,
    pub scope: serde_json::Value,
}
```

---

## Configuration Entities

### LoadMode

How to handle existing state when loading a fixture.

```
LoadMode (Enum)
├── Replace   # Clear existing state, load fixture
└── Merge     # Keep existing, add fixture (future)
```

---

## Rust Implementation

```rust
use serde::{Deserialize, Serialize};

/// Schema version for fixtures
pub const FIXTURE_VERSION: &str = "1.0";

/// A portable snapshot of server state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FixtureMetadata>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buckets: Vec<BucketInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<ObjectInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<IssueInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub translations: Vec<TranslationInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub webhooks: Vec<WebhookInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submittals: Vec<SubmittalInfo>,
}

impl Fixture {
    /// Create an empty fixture with current version
    pub fn empty() -> Self {
        Self {
            version: FIXTURE_VERSION.to_string(),
            metadata: None,
            buckets: vec![],
            objects: vec![],
            projects: vec![],
            issues: vec![],
            translations: vec![],
            webhooks: vec![],
            submittals: vec![],
        }
    }

    /// Create a fixture with metadata
    pub fn with_metadata(name: &str, description: &str) -> Self {
        Self {
            metadata: Some(FixtureMetadata {
                name: Some(name.to_string()),
                description: Some(description.to_string()),
                author: None,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            }),
            ..Self::empty()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
```

---

## YAML Examples

### Minimal Fixture

```yaml
version: "1.0"
```

### Basic Fixture with Buckets

```yaml
version: "1.0"
metadata:
  name: "Basic Buckets"
  description: "Two test buckets for OSS testing"

buckets:
  - bucket_key: "test-bucket-1"
    policy_key: "transient"
  - bucket_key: "test-bucket-2"
    policy_key: "persistent"
    region: "US"
```

### Full Project Fixture

```yaml
version: "1.0"
metadata:
  name: "Hospital Project"
  description: "Complete project setup for ACC testing"
  author: "Test Team"
  created_at: "2026-01-15T10:00:00Z"

buckets:
  - bucket_key: "hospital-models"
    policy_key: "persistent"
    region: "US"

objects:
  - bucket_key: "hospital-models"
    object_key: "hospital-L1.rvt"
    size: 152000000
    sha1: "abc123def456"
    content_type: "application/octet-stream"

projects:
  - id: "project-hospital-001"
    name: "Central Hospital"
    hub_id: "hub-healthcare-001"

issues:
  - id: "issue-001"
    project_id: "project-hospital-001"
    title: "Structural clash at Level 3"
    status: "open"
    created_at: 1736931600000

  - id: "issue-002"
    project_id: "project-hospital-001"
    title: "Missing fire rating"
    status: "closed"
    created_at: 1736932000000

translations:
  - urn: "dXJuOmFkc2sub2JqZWN0czpvcy5vYmplY3Q6aG9zcGl0YWwtbW9kZWxzL2hvc3BpdGFsLUwxLnJ2dA"
    status: "success"
    progress: "complete"
```

---

## Validation Rules

1. **Fixture.version**: Must be a valid version string (e.g., "1.0")
2. **BucketInfo.bucket_key**: Must not be empty
3. **ObjectInfo**: Must reference a valid bucket_key
4. **IssueInfo**: Must reference a valid project_id (warning if not found)
5. **Collections**: Duplicate IDs within a collection trigger warnings

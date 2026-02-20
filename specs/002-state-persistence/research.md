# Research: State Persistence and Fixtures

**Feature**: 002-state-persistence
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing state persistence and fixtures in raps-mock.

---

## Research Topics

### 1. Fixture File Format

**Question**: What format should fixtures use?

**Decision**: YAML as primary format, JSON as alternative.

**Rationale**:
- YAML is more readable and editable by humans
- Comments are supported in YAML (helpful for documentation)
- JSON is already supported via serde_json for programmatic generation
- Both use the same Rust structs via serde

**Alternatives Considered**:
- **JSON only**: Less readable for hand-editing
- **TOML**: Less suited for nested/array data structures
- **Custom format**: No benefit over established formats

---

### 2. Fixture Schema Structure

**Question**: How should fixtures be structured?

**Decision**: Single-file format with version, metadata, and resource collections.

**Rationale**:
- Single file is easier to manage and share
- Version field enables future migrations
- Metadata (name, description) aids discoverability
- Collections mirror StateManager structure

**Structure**:
```yaml
version: "1.0"
metadata:
  name: "Hospital Project"
  description: "Pre-configured state for testing"

buckets:
  - bucket_key: "hospital-models"
    policy_key: "persistent"

projects:
  - id: "project-001"
    name: "Central Hospital"
    hub_id: "hub-001"

issues:
  - id: "issue-001"
    project_id: "project-001"
    title: "Structural clash"
    status: "open"
```

---

### 3. State Serialization Approach

**Question**: How should DashMap state be serialized?

**Decision**: Convert to Vec for serialization, rebuild DashMap on load.

**Rationale**:
- DashMap doesn't directly serialize to useful format
- Converting to Vec produces clean array output
- Rebuilding DashMap on load is fast (O(n))
- Maintains consistency with API response format

**Implementation**:
```rust
impl StateManager {
    pub fn export_fixture(&self) -> Fixture {
        Fixture {
            version: "1.0".to_string(),
            buckets: self.buckets.iter().map(|r| r.value().clone()).collect(),
            projects: self.projects.export_all(),
            issues: self.issues.export_all(),
            // ... other collections
        }
    }

    pub fn load_fixture(&self, fixture: &Fixture) {
        // Clear and repopulate each state module
        self.buckets.clear();
        for bucket in &fixture.buckets {
            self.buckets.insert(bucket.bucket_key.clone(), bucket.clone());
        }
        // ... repeat for other collections
    }
}
```

---

### 4. File I/O Strategy

**Question**: How should fixture files be read/written?

**Decision**: Async file I/O with atomic writes.

**Rationale**:
- Async prevents blocking the server during fixture operations
- Atomic writes (temp file + rename) prevent corruption
- Format detected from file extension (.yaml/.yml vs .json)

**Implementation**:
```rust
pub async fn load_fixture(path: &Path) -> Result<Fixture> {
    let content = tokio::fs::read_to_string(path).await?;

    match path.extension().and_then(|e| e.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content),
        Some("json") => serde_json::from_str(&content),
        _ => Err(FixtureError::UnsupportedFormat),
    }
}

pub async fn save_fixture(path: &Path, fixture: &Fixture) -> Result<()> {
    let content = match path.extension().and_then(|e| e.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::to_string(fixture)?,
        _ => serde_json::to_string_pretty(fixture)?,
    };

    // Atomic write
    let temp_path = path.with_extension("tmp");
    tokio::fs::write(&temp_path, &content).await?;
    tokio::fs::rename(temp_path, path).await?;
    Ok(())
}
```

---

### 5. Version Migration

**Question**: How should version differences be handled?

**Decision**: Simple version checking with warnings.

**Rationale**:
- v1.0 is the initial version; no migrations needed yet
- Future versions can add migration logic
- Warnings for minor version differences, errors for major

**Rules**:
| Fixture Version | Server Version | Action |
|-----------------|----------------|--------|
| 1.0 | 1.0 | Load normally |
| 1.0 | 1.1 | Load with info log |
| 1.0 | 2.0 | Error: major version mismatch |
| 1.1 | 1.0 | Warning: fixture newer than server |

---

### 6. Clear vs. Merge Semantics

**Question**: Should loading a fixture clear existing state or merge?

**Decision**: Clear by default, with optional merge mode.

**Rationale**:
- Clear is safer and more predictable
- Merge is complex (what about conflicts?)
- Default behavior matches user expectation ("load this state")

**Configuration**:
```rust
pub enum LoadMode {
    Replace,  // Default: clear existing, load fixture
    Merge,    // Keep existing, add fixture data (conflicts keep existing)
}
```

---

### 7. Public API Design

**Question**: What API should be exposed?

**Decision**: Extend MockServer and TestServer with fixture methods.

**Rationale**:
- Consistent with library-first architecture
- TestServer gets convenience methods for tests
- CLI uses the same library API

**API**:
```rust
// MockServer methods
impl MockServer {
    pub async fn load_fixture(&self, path: &Path) -> Result<()>;
    pub async fn export_fixture(&self, path: &Path) -> Result<()>;
}

// MockServerConfig extension
impl MockServerConfig {
    pub fn with_fixture(mut self, path: PathBuf) -> Self;
}

// TestServer convenience
impl TestServer {
    pub async fn start_with_fixture(path: &Path) -> Result<Self>;
}
```

---

## Dependencies

### No New Dependencies Required

All required crates are already in Cargo.toml:
- `serde` + `serde_json` - JSON serialization
- `serde_yaml` - YAML serialization
- `tokio` - Async file I/O

---

## File Format Specification

### Fixture Schema v1.0

```yaml
# Required
version: "1.0"

# Optional metadata
metadata:
  name: string        # Display name
  description: string # Description
  author: string      # Creator
  created_at: string  # ISO 8601 timestamp

# Resource collections (all optional)
buckets:
  - bucket_key: string
    policy_key: string
    region: string      # Optional: US, EMEA
    created_date: string

objects:
  - bucket_key: string
    object_key: string
    size: integer
    sha1: string
    content_type: string

projects:
  - id: string
    name: string
    hub_id: string

issues:
  - id: string
    project_id: string
    title: string
    status: string
    description: string

translations:
  - urn: string
    status: string      # pending, inprogress, success, failed
    progress: string

webhooks:
  - id: string
    callback_url: string
    event: string
    scope: object

submittals:  # If feature 001 is implemented
  - id: string
    project_id: string
    title: string
    status: string
```

---

## Summary of Decisions

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| Format | YAML primary, JSON alternative | Human readability |
| Schema | Single file with collections | Simplicity, portability |
| Serialization | Vec conversion | Clean output format |
| File I/O | Async with atomic writes | Non-blocking, corruption-safe |
| Versioning | Check with warnings | Future compatibility |
| Load Mode | Clear by default | Predictability |
| API | Extend MockServer/TestServer | Library-first |

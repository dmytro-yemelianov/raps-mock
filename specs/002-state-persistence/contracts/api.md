# API Contracts: State Persistence

**Feature**: 002-state-persistence
**Date**: 2026-01-15
**Type**: Rust Library API (not HTTP endpoints)

## Overview

This document defines the Rust library API for fixture operations. State persistence is a library/CLI feature, not an HTTP API.

---

## Public API

### MockServer Extensions

```rust
impl MockServer {
    /// Load a fixture from file, replacing current state.
    ///
    /// # Arguments
    /// * `path` - Path to the fixture file (.yaml, .yml, or .json)
    ///
    /// # Errors
    /// * `FixtureError::NotFound` - File doesn't exist
    /// * `FixtureError::ParseError` - Invalid YAML/JSON
    /// * `FixtureError::VersionMismatch` - Major version incompatibility
    pub async fn load_fixture(&self, path: &Path) -> Result<(), FixtureError>;

    /// Export current state to a fixture file.
    ///
    /// # Arguments
    /// * `path` - Output path (format determined by extension)
    ///
    /// # Errors
    /// * `FixtureError::IoError` - Write failed
    pub async fn export_fixture(&self, path: &Path) -> Result<(), FixtureError>;

    /// Get a reference to the current state as a fixture.
    pub fn as_fixture(&self) -> Fixture;
}
```

### MockServerConfig Extensions

```rust
impl MockServerConfig {
    /// Set a fixture file to load at startup.
    ///
    /// # Arguments
    /// * `path` - Path to fixture file
    pub fn with_fixture(mut self, path: impl Into<PathBuf>) -> Self {
        self.fixture_path = Some(path.into());
        self
    }

    /// Create config that loads a fixture at startup.
    pub fn with_fixture_at(path: impl Into<PathBuf>) -> Self {
        Self::default().with_fixture(path)
    }
}
```

### TestServer Extensions

```rust
impl TestServer {
    /// Start a test server with a fixture loaded.
    ///
    /// # Arguments
    /// * `fixture_path` - Path to fixture file
    ///
    /// # Example
    /// ```rust
    /// let server = TestServer::start_with_fixture("fixtures/project.yaml").await?;
    /// ```
    pub async fn start_with_fixture(
        fixture_path: impl AsRef<Path>
    ) -> Result<Self, MockError>;

    /// Start a test server with inline fixture data.
    ///
    /// # Arguments
    /// * `fixture` - Fixture struct with pre-configured data
    ///
    /// # Example
    /// ```rust
    /// let fixture = Fixture::empty()
    ///     .with_bucket("test-bucket", "transient");
    /// let server = TestServer::start_with(fixture).await?;
    /// ```
    pub async fn start_with(fixture: Fixture) -> Result<Self, MockError>;
}
```

---

## Fixture Module API

### Types

```rust
pub use crate::fixture::types::{Fixture, FixtureMetadata, FIXTURE_VERSION};
```

### Loading

```rust
/// Load a fixture from a file path.
///
/// Format is determined by file extension:
/// - `.yaml` or `.yml` → YAML
/// - `.json` → JSON
pub async fn load_from_file(path: &Path) -> Result<Fixture, FixtureError>;

/// Load a fixture from a YAML string.
pub fn from_yaml(content: &str) -> Result<Fixture, FixtureError>;

/// Load a fixture from a JSON string.
pub fn from_json(content: &str) -> Result<Fixture, FixtureError>;
```

### Saving

```rust
/// Save a fixture to a file path.
///
/// Format is determined by file extension.
pub async fn save_to_file(path: &Path, fixture: &Fixture) -> Result<(), FixtureError>;

/// Serialize fixture to YAML string.
pub fn to_yaml(fixture: &Fixture) -> Result<String, FixtureError>;

/// Serialize fixture to JSON string.
pub fn to_json(fixture: &Fixture) -> Result<String, FixtureError>;
```

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("Fixture file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read fixture file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Unsupported fixture format: {0}")]
    UnsupportedFormat(String),

    #[error("Version mismatch: fixture is v{fixture}, server supports v{supported}")]
    VersionMismatch {
        fixture: String,
        supported: String,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),
}
```

---

## CLI Interface

### Load Fixture at Startup

```bash
raps-mock --fixture fixtures/project.yaml
raps-mock --fixture fixtures/data.json
```

### Export Current State

```bash
# Via admin endpoint (if implemented) or shutdown hook
raps-mock export-state --output current-state.yaml
```

### List Available Fixtures

```bash
raps-mock fixtures list --dir ./fixtures/
```

**Output**:
```
Available fixtures in ./fixtures/:
  basic-buckets.yaml     - Two test buckets for OSS testing
  hospital-project.yaml  - Complete project setup for ACC testing
  empty.yaml             - Empty fixture (no resources)
```

---

## Usage Examples

### CLI Usage

```bash
# Start with a fixture
raps-mock --port 3000 --fixture fixtures/hospital.yaml

# Export current state
raps-mock export-state --output backup.yaml
```

### Library Usage

```rust
use raps_mock::{MockServer, MockServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Config with fixture
    let config = MockServerConfig::default()
        .with_fixture("fixtures/project.yaml");
    let server = MockServer::new(config).await?;

    // Option 2: Load fixture after creation
    let server = MockServer::new(MockServerConfig::default()).await?;
    server.load_fixture(Path::new("fixtures/project.yaml")).await?;

    server.start("0.0.0.0:3000").await?;
    Ok(())
}
```

### Test Usage

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_with_fixture() {
    // Load from file
    let server = TestServer::start_with_fixture("fixtures/test-data.yaml")
        .await
        .unwrap();

    // Buckets from fixture are immediately available
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_with_inline_fixture() {
    use raps_mock::Fixture;

    let fixture = Fixture::empty();
    // Add resources programmatically...

    let server = TestServer::start_with(fixture).await.unwrap();
}
```

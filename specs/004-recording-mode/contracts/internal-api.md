# Internal API Contracts: Recording and Playback

**Feature**: 004-recording-mode
**Date**: 2026-01-15
**Type**: Rust Module Interfaces (not HTTP endpoints)

## Overview

This document defines the internal Rust API contracts for the recording and playback feature. These are library interfaces, not HTTP endpoints - the feature operates as middleware that intercepts and records/replays HTTP interactions.

---

## Module: `recording::recorder`

### Recorder

The main component for capturing request-response pairs.

```rust
pub struct Recorder {
    // Private fields
}

impl Recorder {
    /// Create a new recorder with the given configuration.
    ///
    /// # Errors
    /// - `RecordingError::InvalidOutputDir` if output_dir doesn't exist or isn't writable
    /// - `RecordingError::InvalidBackendUrl` if backend_url is malformed
    pub fn new(config: RecordingConfig) -> Result<Self, RecordingError>;

    /// Record a request-response interaction.
    ///
    /// This method is called by the recording middleware after proxying.
    ///
    /// # Errors
    /// - `RecordingError::IoError` if file write fails
    /// - `RecordingError::SerializationError` if JSON encoding fails
    pub async fn record(&self, recording: Recording) -> Result<(), RecordingError>;

    /// Get the current session ID.
    pub fn session_id(&self) -> &str;

    /// Get the number of recordings in the current session.
    pub fn recording_count(&self) -> u64;

    /// Finalize the session and write the session manifest.
    pub async fn finalize(&self) -> Result<RecordingSession, RecordingError>;
}
```

---

## Module: `recording::player`

### Player

The main component for serving recorded responses.

```rust
pub struct Player {
    // Private fields
}

impl Player {
    /// Create a new player by loading recordings from a directory.
    ///
    /// # Errors
    /// - `PlaybackError::InvalidRecordingsDir` if directory doesn't exist
    /// - `PlaybackError::NoRecordingsFound` if directory is empty
    /// - `PlaybackError::InvalidRecording` if a recording file is malformed
    pub async fn new(config: PlaybackConfig) -> Result<Self, PlaybackError>;

    /// Find a matching recording for the given request.
    ///
    /// # Returns
    /// - `Some(Recording)` if a match is found
    /// - `None` if no recording matches
    pub fn find_match(&self, request: &RecordedRequest) -> Option<&Recording>;

    /// Get the number of loaded recordings.
    pub fn recording_count(&self) -> usize;

    /// Get recordings grouped by path for inspection.
    pub fn recordings_by_path(&self) -> &HashMap<(Method, String), Vec<&Recording>>;
}
```

---

## Module: `recording::anonymizer`

### Anonymizer

Component for redacting sensitive data from recordings.

```rust
pub struct Anonymizer {
    // Private fields
}

impl Anonymizer {
    /// Create an anonymizer with the given patterns.
    ///
    /// If patterns is empty, default patterns are used.
    pub fn new(patterns: Vec<AnonymizePattern>) -> Self;

    /// Create an anonymizer with default patterns only.
    pub fn default() -> Self;

    /// Anonymize a request, returning a modified copy.
    pub fn anonymize_request(&self, request: &RecordedRequest) -> RecordedRequest;

    /// Anonymize a response, returning a modified copy.
    pub fn anonymize_response(&self, response: &RecordedResponse) -> RecordedResponse;

    /// Check if a header name should be anonymized.
    pub fn should_anonymize_header(&self, name: &str) -> bool;
}
```

---

## Module: `recording::storage`

### Storage Functions

File I/O utilities for recordings.

```rust
/// Write a recording to a file.
///
/// Uses atomic write (temp file + rename) to prevent corruption.
///
/// # Errors
/// - `StorageError::IoError` on file system errors
/// - `StorageError::SerializationError` on encoding errors
pub async fn write_recording(
    path: &Path,
    recording: &Recording,
    format: RecordingFormat,
) -> Result<(), StorageError>;

/// Read a recording from a file.
///
/// # Errors
/// - `StorageError::IoError` on file system errors
/// - `StorageError::DeserializationError` on decoding errors
/// - `StorageError::InvalidVersion` if schema version is unsupported
pub async fn read_recording(path: &Path) -> Result<Recording, StorageError>;

/// Read all recordings from a directory.
///
/// Skips invalid files and logs warnings.
pub async fn read_recordings_dir(dir: &Path) -> Result<Vec<Recording>, StorageError>;

/// Write a session manifest file.
pub async fn write_session_manifest(
    path: &Path,
    session: &RecordingSession,
) -> Result<(), StorageError>;

/// Read a session manifest file.
pub async fn read_session_manifest(path: &Path) -> Result<RecordingSession, StorageError>;
```

---

## Module: `recording::types`

All type definitions from data-model.md, implemented as Rust structs with serde derives.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub version: String,
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub session_id: String,
    pub request: RecordedRequest,
    pub response: RecordedResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<RecordingMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<RecordedBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<RecordedBody>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub truncated: bool,
}

// Additional types: RecordingMetadata, RecordingSession, RecordingConfig,
// PlaybackConfig, AnonymizePattern, MatchingMode, FallbackMode, RecordingFormat
// (see data-model.md for full definitions)
```

---

## Module: `middleware::recording`

### Recording Middleware

Tower middleware layer for intercepting requests.

```rust
/// Create recording middleware layer.
///
/// This middleware:
/// 1. Captures the incoming request
/// 2. Proxies it to the backend
/// 3. Captures the response
/// 4. Writes the recording asynchronously
/// 5. Returns the response to the client
pub fn recording_layer(recorder: Arc<Recorder>) -> RecordingLayer;

/// Create playback middleware layer.
///
/// This middleware:
/// 1. Matches the incoming request against recordings
/// 2. Returns the recorded response if found
/// 3. Returns 404 or proxies to fallback if not found
pub fn playback_layer(player: Arc<Player>) -> PlaybackLayer;
```

---

## Error Types

### RecordingError

```rust
#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    #[error("Invalid output directory: {0}")]
    InvalidOutputDir(String),

    #[error("Invalid backend URL: {0}")]
    InvalidBackendUrl(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Proxy error: {0}")]
    ProxyError(String),
}
```

### PlaybackError

```rust
#[derive(Debug, thiserror::Error)]
pub enum PlaybackError {
    #[error("Invalid recordings directory: {0}")]
    InvalidRecordingsDir(String),

    #[error("No recordings found in directory")]
    NoRecordingsFound,

    #[error("Invalid recording file {path}: {reason}")]
    InvalidRecording { path: String, reason: String },

    #[error("No matching recording for request: {method} {path}")]
    NoMatch { method: String, path: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

---

## Public API Extensions

### MockServerConfig Extensions

```rust
impl MockServerConfig {
    /// Create config for record mode.
    pub fn record_mode(
        backend_url: impl Into<String>,
        output_dir: impl Into<PathBuf>,
    ) -> Self;

    /// Create config for playback mode.
    pub fn playback_mode(recordings_dir: impl Into<PathBuf>) -> Self;
}
```

### MockServer Extensions

```rust
impl MockServer {
    /// Check if server is in recording mode.
    pub fn is_recording(&self) -> bool;

    /// Check if server is in playback mode.
    pub fn is_playback(&self) -> bool;

    /// Get recording statistics (if in record mode).
    pub fn recording_stats(&self) -> Option<RecordingStats>;

    /// Get playback statistics (if in playback mode).
    pub fn playback_stats(&self) -> Option<PlaybackStats>;
}
```

### TestServer Extensions

```rust
impl TestServer {
    /// Start a test server in record mode.
    pub async fn start_recording(
        backend_url: &str,
        output_dir: &Path,
    ) -> Result<Self>;

    /// Start a test server in playback mode.
    pub async fn start_playback(recordings_dir: &Path) -> Result<Self>;
}
```

---

## Usage Examples

### Recording Mode

```rust
use raps_mock::{MockServer, MockServerConfig};

let config = MockServerConfig::record_mode(
    "https://developer.api.autodesk.com",
    "./recordings/session-001",
);
let server = MockServer::new(config).await?;
server.start("0.0.0.0:3000").await?;
```

### Playback Mode

```rust
use raps_mock::{MockServer, MockServerConfig};

let config = MockServerConfig::playback_mode("./recordings/session-001");
let server = MockServer::new(config).await?;
server.start("0.0.0.0:3000").await?;
```

### In Tests

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_with_recorded_data() {
    let server = TestServer::start_playback("./fixtures/bucket-workflow").await?;

    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await?;

    assert_eq!(resp.status(), 200);
}
```

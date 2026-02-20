# Research: Request Recording and Playback

**Feature**: 004-recording-mode
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing request recording and playback functionality in raps-mock.

---

## Research Topics

### 1. HTTP Proxy Implementation for Recording

**Question**: How should we implement the HTTP proxy to forward requests to the real APS backend while recording?

**Decision**: Use reqwest as the HTTP client for proxying requests.

**Rationale**:
- reqwest is already a dev-dependency in the project (used in tests)
- Mature, well-maintained library with excellent async support
- Supports all HTTP methods, headers, and body streaming
- Handles TLS/SSL automatically with rustls or native-tls

**Alternatives Considered**:
- **hyper directly**: More low-level control but significantly more code required
- **tower-http proxy**: Good option but adds dependency; reqwest simpler for our use case

**Implementation Notes**:
```rust
// Proxy middleware pseudocode
async fn proxy_request(req: Request, backend_url: &str) -> Response {
    let client = reqwest::Client::new();
    let proxied = client
        .request(req.method(), format!("{}{}", backend_url, req.uri()))
        .headers(req.headers())
        .body(req.body())
        .send()
        .await?;
    // Convert reqwest::Response to axum::Response
}
```

---

### 2. Recording File Format

**Question**: What format should we use to store request/response recordings?

**Decision**: JSON as primary format, with optional YAML support for human editing.

**Rationale**:
- JSON is compact and fast to parse
- serde_json already in dependencies
- YAML support via serde_yaml (already available) for human-readable editing
- Single-file-per-interaction is simpler than session-grouped files

**Alternatives Considered**:
- **Binary format (MessagePack, CBOR)**: Better performance but not human-readable
- **HAR format**: Standard but overly complex for our needs
- **Single session file**: Harder to edit individual interactions

**File Structure**:
```json
{
  "version": "1.0",
  "id": "recording-001",
  "timestamp": "2026-01-15T10:30:00Z",
  "sequence": 1,
  "session_id": "session-abc123",
  "request": {
    "method": "GET",
    "path": "/oss/v2/buckets",
    "query": "limit=10",
    "headers": { "Authorization": "[REDACTED]" },
    "body": null
  },
  "response": {
    "status": 200,
    "headers": { "Content-Type": "application/json" },
    "body": "{\"items\": [...]}"
  }
}
```

---

### 3. Request Matching Strategy

**Question**: How should playback mode match incoming requests to recorded interactions?

**Decision**: Multi-level matching with configurable strictness.

**Rationale**:
- Strict matching (exact method + path + query + headers + body) is default
- Flexible matching allows ignoring dynamic fields (timestamps, request IDs)
- Method + path is always required; other fields are configurable

**Matching Levels**:
1. **Strict (default)**: All fields must match exactly
2. **Path-only**: Match method + path, ignore query/headers/body
3. **Custom**: Configurable ignore lists for headers and query params

**Algorithm**:
```
1. Filter recordings by method (exact match)
2. Filter by path (exact match or pattern)
3. Score remaining by query param similarity
4. Score by header similarity (excluding ignored)
5. Score by body similarity (if enabled)
6. Return highest-scoring match above threshold
```

**Alternatives Considered**:
- **Hash-based matching**: Fast but no flexibility for dynamic fields
- **Regex patterns**: Powerful but complex to configure

---

### 4. Anonymization Approach

**Question**: How should we redact sensitive data from recordings?

**Decision**: Pattern-based replacement with configurable patterns.

**Rationale**:
- Common patterns (Authorization header, access_token field) covered by default
- Users can add custom patterns for project-specific sensitive data
- Replacement preserves structure (token becomes placeholder, not deleted)

**Default Patterns**:
- Headers: `Authorization`, `X-Ads-Token`, `Cookie`
- Body fields: `access_token`, `refresh_token`, `client_secret`, `password`
- Query params: `access_token`, `api_key`

**Replacement Strategy**:
- Header values: `[REDACTED]`
- JSON fields: `"[REDACTED]"`
- Query params: `[REDACTED]`

**Alternatives Considered**:
- **Full encryption**: Secure but breaks human readability
- **Hashing**: One-way but can still leak info via rainbow tables
- **Manual redaction**: Error-prone, not scalable

---

### 5. Large Body Handling

**Question**: How should we handle large request/response bodies (e.g., file uploads)?

**Decision**: Configurable size limit with metadata-only option.

**Rationale**:
- Default limit of 10MB for body recording
- Bodies exceeding limit stored as metadata only (size, content-type, hash)
- Users can configure limit up or down based on needs
- Prevents disk space exhaustion from large file uploads

**Configuration**:
```rust
pub struct RecordingConfig {
    pub max_body_size: usize,  // Default: 10MB
    pub skip_binary_bodies: bool,  // Default: true for Content-Type: application/octet-stream
    pub body_hash_algorithm: HashAlgorithm,  // SHA256 for integrity verification
}
```

**Alternatives Considered**:
- **Always record full body**: Risk of disk exhaustion
- **Never record bodies**: Loses important data for many use cases
- **External file storage**: Complex, breaks portability

---

### 6. Concurrent Recording Safety

**Question**: How do we ensure thread-safe recording when multiple requests are processed simultaneously?

**Decision**: Atomic file writes with sequence-based naming.

**Rationale**:
- Each recording is a separate file, avoiding write conflicts
- Atomic write (temp file + rename) prevents corruption
- AtomicU64 counter for sequence numbers ensures ordering
- No need for file locking across processes

**Implementation**:
```rust
impl Recorder {
    fn write_recording(&self, recording: &Recording) -> Result<()> {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let filename = format!("{:06}-{}.json", seq, recording.id);
        let temp_path = self.output_dir.join(format!(".{}.tmp", filename));
        let final_path = self.output_dir.join(&filename);

        // Write to temp file
        std::fs::write(&temp_path, serde_json::to_string_pretty(recording)?)?;
        // Atomic rename
        std::fs::rename(temp_path, final_path)?;
        Ok(())
    }
}
```

---

### 7. Playback Index Strategy

**Question**: How should we index recordings for fast lookup during playback?

**Decision**: In-memory index built at startup, keyed by method + path.

**Rationale**:
- Typical recording sets (50-200 files) easily fit in memory
- HashMap provides O(1) lookup by primary key
- Index built once at startup, no ongoing I/O during playback
- Secondary matching (query, headers) done on candidate set

**Data Structure**:
```rust
pub struct PlaybackIndex {
    // Primary index: method + path -> list of matching recordings
    by_path: HashMap<(Method, String), Vec<Recording>>,
    // Total recordings loaded
    count: usize,
}
```

**Alternatives Considered**:
- **SQLite index**: Overkill for typical sizes, adds dependency
- **On-demand file scanning**: Too slow for playback latency requirements
- **Trie for path matching**: Complex, marginal benefit for typical path counts

---

## Integration with Existing Architecture

### MockMode Extension

Current `MockMode` enum will be extended:

```rust
pub enum MockMode {
    Stateless,
    Stateful,
    Record { backend_url: String, output_dir: PathBuf },
    Playback { recordings_dir: PathBuf },
}
```

### MockServerConfig Extension

```rust
pub struct MockServerConfig {
    // Existing fields...
    pub mode: MockMode,

    // New recording-specific config
    pub recording: Option<RecordingConfig>,
    pub playback: Option<PlaybackConfig>,
}

pub struct RecordingConfig {
    pub anonymize: bool,
    pub anonymize_patterns: Vec<AnonymizePattern>,
    pub max_body_size: usize,
    pub record_bodies: bool,
}

pub struct PlaybackConfig {
    pub matching_mode: MatchingMode,
    pub ignored_headers: Vec<String>,
    pub ignored_query_params: Vec<String>,
    pub sequential: bool,
}
```

---

## Dependencies

### New Dependencies Required

```toml
[dependencies]
# For proxying (promote from dev-dependencies)
reqwest = { version = "0.11", features = ["json", "stream"] }

# Already present, no changes needed:
# - serde, serde_json, serde_yaml
# - tokio
# - axum
# - uuid
# - chrono
```

### No New Dependencies For

- File I/O: std::fs
- Hashing: Use existing or minimal implementation
- Atomic operations: std::sync::atomic

---

## Performance Considerations

1. **Recording Overhead**: Target <50ms additional latency
   - Async file writes (don't block request handling)
   - Buffer writes if needed for high-throughput scenarios

2. **Playback Startup**: Target <500ms for 50 recordings
   - Parallel file reading with tokio::spawn
   - Lazy body loading if recordings are large

3. **Memory Usage**: O(n) where n = recording count
   - Keep full request/response in memory for matching
   - Consider streaming for very large recording sets (future optimization)

---

## Security Considerations

1. **Credential Exposure**: Anonymization enabled by default for record mode
2. **File Permissions**: Recordings written with restrictive permissions (0600)
3. **Temporary Files**: Cleaned up on error; use secure temp directory

---

## Summary of Decisions

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| HTTP Proxy | reqwest | Already in project, mature, async |
| File Format | JSON (primary), YAML (optional) | Compact, human-readable, existing deps |
| Request Matching | Multi-level with config | Balance between strictness and flexibility |
| Anonymization | Pattern-based replacement | Covers common cases, configurable |
| Large Bodies | Size limit + metadata | Prevents disk exhaustion |
| Concurrency | Atomic writes + sequence | No locking, safe parallel recording |
| Playback Index | In-memory HashMap | Fast lookup, reasonable memory for typical sizes |

# Data Model: Request Recording and Playback

**Feature**: 004-recording-mode
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document defines the data structures for the recording and playback feature. These models represent recorded HTTP interactions and the configuration for matching and playback.

---

## Core Entities

### Recording

A single captured request-response interaction.

```
Recording
├── id: String (UUID)
├── version: String ("1.0")
├── timestamp: DateTime (ISO 8601)
├── sequence: u64
├── session_id: String (UUID)
├── request: RecordedRequest
├── response: RecordedResponse
└── metadata: RecordingMetadata
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Unique identifier (UUID v4) |
| `version` | String | Yes | Schema version for forward compatibility |
| `timestamp` | DateTime | Yes | When the interaction was recorded |
| `sequence` | u64 | Yes | Order within the session (0-indexed) |
| `session_id` | String | Yes | Groups related recordings |
| `request` | RecordedRequest | Yes | The captured request |
| `response` | RecordedResponse | Yes | The captured response |
| `metadata` | RecordingMetadata | No | Additional context |

---

### RecordedRequest

The HTTP request captured during recording.

```
RecordedRequest
├── method: String (GET, POST, etc.)
├── path: String
├── query: Option<String>
├── headers: Map<String, String>
├── body: Option<RecordedBody>
└── host: Option<String>
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `method` | String | Yes | HTTP method (GET, POST, PUT, DELETE, PATCH) |
| `path` | String | Yes | Request path (e.g., "/oss/v2/buckets") |
| `query` | String | No | Query string without leading "?" |
| `headers` | Map | Yes | Request headers (may be anonymized) |
| `body` | RecordedBody | No | Request body if present |
| `host` | String | No | Original host header value |

---

### RecordedResponse

The HTTP response captured during recording.

```
RecordedResponse
├── status: u16
├── headers: Map<String, String>
├── body: Option<RecordedBody>
└── duration_ms: u64
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | u16 | Yes | HTTP status code (200, 404, 500, etc.) |
| `headers` | Map | Yes | Response headers |
| `body` | RecordedBody | No | Response body if present |
| `duration_ms` | u64 | Yes | Backend response time in milliseconds |

---

### RecordedBody

Represents a request or response body, with support for large/binary content.

```
RecordedBody
├── content: Option<String>
├── content_type: Option<String>
├── size: u64
├── hash: Option<String>
└── truncated: bool
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | String | No | Body content (if not truncated/binary) |
| `content_type` | String | No | Content-Type header value |
| `size` | u64 | Yes | Original body size in bytes |
| `hash` | String | No | SHA256 hash for integrity verification |
| `truncated` | bool | Yes | True if body exceeded size limit |

**Invariants**:
- If `truncated` is true, `content` is None
- `hash` is always computed regardless of truncation

---

### RecordingMetadata

Optional context about the recording.

```
RecordingMetadata
├── backend_url: String
├── anonymized: bool
├── recorded_by: Option<String>
└── tags: Vec<String>
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `backend_url` | String | Yes | The backend URL used during recording |
| `anonymized` | bool | Yes | Whether sensitive data was redacted |
| `recorded_by` | String | No | Tool/version that created the recording |
| `tags` | Vec<String> | No | User-defined tags for organization |

---

### RecordingSession

A group of recordings captured together.

```
RecordingSession
├── session_id: String (UUID)
├── start_time: DateTime
├── end_time: Option<DateTime>
├── backend_url: String
├── recording_count: u64
└── config: RecordingConfig
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | String | Yes | Unique session identifier |
| `start_time` | DateTime | Yes | When recording started |
| `end_time` | DateTime | No | When recording ended (None if ongoing) |
| `backend_url` | String | Yes | Target backend for proxying |
| `recording_count` | u64 | Yes | Number of recordings in session |
| `config` | RecordingConfig | Yes | Configuration used for this session |

---

## Configuration Entities

### RecordingConfig

Configuration for record mode.

```
RecordingConfig
├── output_dir: PathBuf
├── backend_url: String
├── anonymize: bool
├── anonymize_patterns: Vec<AnonymizePattern>
├── max_body_size: usize
├── record_bodies: bool
└── format: RecordingFormat
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `output_dir` | PathBuf | "./recordings" | Directory for saving recordings |
| `backend_url` | String | (required) | APS backend URL to proxy to |
| `anonymize` | bool | true | Enable credential anonymization |
| `anonymize_patterns` | Vec | (defaults) | Patterns to anonymize |
| `max_body_size` | usize | 10MB | Max body size to record |
| `record_bodies` | bool | true | Whether to record request/response bodies |
| `format` | RecordingFormat | JSON | Output format |

---

### PlaybackConfig

Configuration for playback mode.

```
PlaybackConfig
├── recordings_dir: PathBuf
├── matching_mode: MatchingMode
├── ignored_headers: Vec<String>
├── ignored_query_params: Vec<String>
├── sequential: bool
└── fallback_mode: FallbackMode
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `recordings_dir` | PathBuf | (required) | Directory containing recordings |
| `matching_mode` | MatchingMode | Strict | How to match requests |
| `ignored_headers` | Vec | [] | Headers to ignore in matching |
| `ignored_query_params` | Vec | [] | Query params to ignore |
| `sequential` | bool | false | Enable sequence-based matching |
| `fallback_mode` | FallbackMode | Error | What to do on no match |

---

### AnonymizePattern

A pattern for identifying sensitive data.

```
AnonymizePattern
├── target: AnonymizeTarget
├── pattern: String
└── replacement: String
```

**AnonymizeTarget Enum**:
- `Header` - Match header name
- `QueryParam` - Match query parameter name
- `JsonPath` - Match JSON path in body

**Default Patterns**:
```
[
  { target: Header, pattern: "Authorization", replacement: "[REDACTED]" },
  { target: Header, pattern: "Cookie", replacement: "[REDACTED]" },
  { target: QueryParam, pattern: "access_token", replacement: "[REDACTED]" },
  { target: JsonPath, pattern: "$.access_token", replacement: "[REDACTED]" },
  { target: JsonPath, pattern: "$.refresh_token", replacement: "[REDACTED]" },
]
```

---

### MatchingMode

How strictly to match requests during playback.

```
MatchingMode (Enum)
├── Strict        # All fields must match exactly
├── PathOnly      # Only method + path
├── Flexible      # Method + path + configurable ignores
└── Sequential    # Match in recorded order
```

---

### FallbackMode

What to do when no recording matches during playback.

```
FallbackMode (Enum)
├── Error         # Return 404 with "no matching recording" message
├── Passthrough   # Proxy to configured backend (hybrid mode)
└── Empty         # Return empty 200 response
```

---

## Relationships

```
RecordingSession 1──* Recording
Recording 1──1 RecordedRequest
Recording 1──1 RecordedResponse
RecordedRequest 0..1──1 RecordedBody
RecordedResponse 0..1──1 RecordedBody
RecordingSession 1──1 RecordingConfig
```

---

## State Transitions

### Recording Lifecycle

```
[Not Recording] --start_recording()--> [Recording Active]
[Recording Active] --stop_recording()--> [Recording Complete]
[Recording Active] --on_request()--> [Recording Active] (writes file)
```

### Playback Lifecycle

```
[Not Loaded] --load_recordings()--> [Index Built]
[Index Built] --on_request()--> [Index Built] (serves from memory)
```

---

## File Format Example

### Recording File (JSON)

```json
{
  "version": "1.0",
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "timestamp": "2026-01-15T10:30:00.000Z",
  "sequence": 0,
  "session_id": "session-abc123",
  "request": {
    "method": "GET",
    "path": "/oss/v2/buckets",
    "query": "limit=10&region=US",
    "headers": {
      "Authorization": "[REDACTED]",
      "Content-Type": "application/json",
      "User-Agent": "raps-cli/0.5.0"
    },
    "body": null,
    "host": "developer.api.autodesk.com"
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json",
      "X-Request-Id": "req-12345"
    },
    "body": {
      "content": "{\"items\":[{\"bucketKey\":\"test-bucket\"}]}",
      "content_type": "application/json",
      "size": 42,
      "hash": "sha256:abc123...",
      "truncated": false
    },
    "duration_ms": 150
  },
  "metadata": {
    "backend_url": "https://developer.api.autodesk.com",
    "anonymized": true,
    "recorded_by": "raps-mock/0.3.0",
    "tags": ["oss", "buckets", "list"]
  }
}
```

### Session Manifest (session.json)

```json
{
  "session_id": "session-abc123",
  "start_time": "2026-01-15T10:30:00.000Z",
  "end_time": "2026-01-15T10:35:00.000Z",
  "backend_url": "https://developer.api.autodesk.com",
  "recording_count": 15,
  "config": {
    "anonymize": true,
    "max_body_size": 10485760,
    "format": "json"
  }
}
```

---

## Validation Rules

1. **Recording.id**: Must be valid UUID v4
2. **Recording.sequence**: Must be unique within session
3. **RecordedRequest.method**: Must be valid HTTP method
4. **RecordedRequest.path**: Must start with "/"
5. **RecordedResponse.status**: Must be 100-599
6. **RecordedBody.size**: Must match actual content length if not truncated
7. **RecordingConfig.max_body_size**: Must be > 0
8. **PlaybackConfig.recordings_dir**: Must exist and be readable

# Data Model: Error and Latency Simulation

**Feature**: 003-error-simulation
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document defines the data structures for chaos engineering configuration and state.

---

## Core Entities

### ChaosConfig

Master configuration for all chaos engineering features.

```
ChaosConfig
├── enabled: bool
├── error_rate: f64
├── error_codes: Vec<WeightedErrorCode>
├── failing_endpoints: Vec<EndpointFailure>
├── latency: Option<LatencyConfig>
├── endpoint_latencies: Vec<EndpointLatency>
└── rate_limit: Option<RateLimitConfig>
```

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | bool | Yes | `true` | Master switch for chaos features |
| `error_rate` | f64 | Yes | `0.0` | Probability of random failure (0.0-1.0) |
| `error_codes` | Vec | No | [standard] | Possible error codes with weights |
| `failing_endpoints` | Vec | No | `[]` | Endpoints configured to always fail |
| `latency` | Option | No | `None` | Global latency settings |
| `endpoint_latencies` | Vec | No | `[]` | Per-endpoint latency overrides |
| `rate_limit` | Option | No | `None` | Rate limiting configuration |

---

### LatencyConfig

Configuration for response latency injection.

```
LatencyConfig
├── base_ms: u64
└── jitter_ms: u64
```

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `base_ms` | u64 | Yes | `0` | Base latency in milliseconds |
| `jitter_ms` | u64 | Yes | `0` | Random jitter range (0 to jitter_ms) |

**Notes**:
- Total latency = base_ms + random(0, jitter_ms)
- Setting both to 0 disables latency injection

---

### EndpointFailure

Configuration for an endpoint that should always fail.

```
EndpointFailure
├── pattern: String
├── status_code: u16
└── message: Option<String>
```

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `pattern` | String | Yes | - | Path pattern (supports `*` wildcard) |
| `status_code` | u16 | Yes | `500` | HTTP status code to return |
| `message` | Option | No | [generic] | Custom error message |

**Pattern Examples**:
- `/oss/v2/buckets` - Exact match
- `/oss/v2/buckets/*` - Match path and one segment
- `/modelderivative/**` - Match path and any segments

---

### EndpointLatency

Per-endpoint latency override.

```
EndpointLatency
├── pattern: String
├── base_ms: u64
└── jitter_ms: u64
```

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `pattern` | String | Yes | - | Path pattern (supports wildcards) |
| `base_ms` | u64 | Yes | - | Base latency for this endpoint |
| `jitter_ms` | u64 | Yes | `0` | Jitter range for this endpoint |

---

### WeightedErrorCode

Error code with selection weight for random errors.

```
WeightedErrorCode
├── code: u16
└── weight: u32
```

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | u16 | Yes | HTTP status code |
| `weight` | u32 | Yes | Relative weight for selection |

**Default Distribution**:
```rust
vec![
    WeightedErrorCode { code: 400, weight: 15 },
    WeightedErrorCode { code: 401, weight: 15 },
    WeightedErrorCode { code: 403, weight: 15 },
    WeightedErrorCode { code: 404, weight: 15 },
    WeightedErrorCode { code: 500, weight: 20 },
    WeightedErrorCode { code: 502, weight: 10 },
    WeightedErrorCode { code: 503, weight: 10 },
]
```

---

### RateLimitConfig

Configuration for request rate limiting.

```
RateLimitConfig
├── requests_per_window: usize
├── window_seconds: u64
└── per_endpoint: Vec<EndpointRateLimit>
```

**Fields**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `requests_per_window` | usize | Yes | - | Max requests in time window |
| `window_seconds` | u64 | Yes | `60` | Length of time window in seconds |
| `per_endpoint` | Vec | No | `[]` | Endpoint-specific overrides |

---

### EndpointRateLimit

Per-endpoint rate limit override.

```
EndpointRateLimit
├── pattern: String
├── requests_per_window: usize
└── window_seconds: u64
```

---

## Runtime State Entities

### RateLimiterState

In-memory state for tracking request counts.

```
RateLimiterState
└── timestamps: DashMap<String, VecDeque<Instant>>
```

**Notes**:
- Key is endpoint pattern or "global" for global limit
- VecDeque stores timestamps of recent requests
- Old timestamps are pruned on each check

---

## Rust Implementation

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Master configuration for chaos engineering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub error_rate: f64,

    #[serde(default = "default_error_codes")]
    pub error_codes: Vec<WeightedErrorCode>,

    #[serde(default)]
    pub failing_endpoints: Vec<EndpointFailure>,

    #[serde(default)]
    pub latency: Option<LatencyConfig>,

    #[serde(default)]
    pub endpoint_latencies: Vec<EndpointLatency>,

    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

fn default_enabled() -> bool { true }

fn default_error_codes() -> Vec<WeightedErrorCode> {
    vec![
        WeightedErrorCode { code: 400, weight: 15 },
        WeightedErrorCode { code: 401, weight: 15 },
        WeightedErrorCode { code: 403, weight: 15 },
        WeightedErrorCode { code: 404, weight: 15 },
        WeightedErrorCode { code: 500, weight: 20 },
        WeightedErrorCode { code: 502, weight: 10 },
        WeightedErrorCode { code: 503, weight: 10 },
    ]
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error_rate: 0.0,
            error_codes: default_error_codes(),
            failing_endpoints: vec![],
            latency: None,
            endpoint_latencies: vec![],
            rate_limit: None,
        }
    }
}

impl ChaosConfig {
    /// Create a config with just an error rate
    pub fn with_error_rate(rate: f64) -> Self {
        Self {
            error_rate: rate.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Create a config with just latency
    pub fn with_latency(base_ms: u64, jitter_ms: u64) -> Self {
        Self {
            latency: Some(LatencyConfig { base_ms, jitter_ms }),
            ..Default::default()
        }
    }

    /// Add an endpoint that should always fail
    pub fn fail_endpoint(mut self, pattern: &str, status_code: u16) -> Self {
        self.failing_endpoints.push(EndpointFailure {
            pattern: pattern.to_string(),
            status_code,
            message: None,
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    pub base_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointFailure {
    pub pattern: String,
    #[serde(default = "default_status")]
    pub status_code: u16,
    pub message: Option<String>,
}

fn default_status() -> u16 { 500 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointLatency {
    pub pattern: String,
    pub base_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedErrorCode {
    pub code: u16,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_window: usize,
    #[serde(default = "default_window")]
    pub window_seconds: u64,
    #[serde(default)]
    pub per_endpoint: Vec<EndpointRateLimit>,
}

fn default_window() -> u64 { 60 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimit {
    pub pattern: String,
    pub requests_per_window: usize,
    #[serde(default = "default_window")]
    pub window_seconds: u64,
}
```

---

## Configuration Examples

### CLI Configuration

```bash
# Simple error rate
raps-mock --error-rate 0.1

# With latency
raps-mock --latency 500 --jitter 100

# Combined
raps-mock --error-rate 0.05 --latency 200 --jitter 50
```

### YAML Configuration

```yaml
chaos:
  enabled: true
  error_rate: 0.1

  latency:
    base_ms: 500
    jitter_ms: 100

  failing_endpoints:
    - pattern: "/oss/v2/buckets/*"
      status_code: 503
      message: "Service temporarily unavailable"

  endpoint_latencies:
    - pattern: "/modelderivative/**"
      base_ms: 2000
      jitter_ms: 500

  rate_limit:
    requests_per_window: 100
    window_seconds: 60
```

### Programmatic Configuration

```rust
let config = ChaosConfig::default()
    .with_error_rate(0.1)
    .with_latency(500, 100)
    .fail_endpoint("/oss/v2/buckets", 503);
```

---

## Validation Rules

1. **error_rate**: Must be between 0.0 and 1.0 inclusive
2. **status_code**: Must be a valid HTTP error status (400-599)
3. **base_ms**: Must be non-negative (0 = no latency)
4. **jitter_ms**: Must be non-negative
5. **requests_per_window**: Must be greater than 0
6. **window_seconds**: Must be greater than 0
7. **pattern**: Must start with `/` for path patterns

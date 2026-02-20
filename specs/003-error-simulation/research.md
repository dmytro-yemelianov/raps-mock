# Research: Error and Latency Simulation

**Feature**: 003-error-simulation
**Date**: 2026-01-15
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing error and latency simulation (chaos engineering) in raps-mock.

---

## Research Topics

### 1. Chaos Middleware Architecture

**Question**: Where in the request pipeline should chaos be applied?

**Decision**: Axum middleware layer that runs before request handlers.

**Rationale**:
- Middleware can intercept all requests uniformly
- Failures return early without touching state
- Latency is applied consistently regardless of handler
- Matches axum's layer-based architecture

**Implementation**:
```rust
pub struct ChaosLayer {
    config: Arc<RwLock<ChaosConfig>>,
}

impl<S> Layer<S> for ChaosLayer {
    type Service = ChaosService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ChaosService {
            inner,
            config: Arc::clone(&self.config),
        }
    }
}
```

---

### 2. Random Number Generation

**Question**: What RNG approach should be used for error injection?

**Decision**: Use `rand::thread_rng()` with standard distribution.

**Rationale**:
- thread_rng is thread-safe and efficient
- Cryptographic randomness is unnecessary for testing
- Standard library provides sufficient randomness
- No additional dependencies required (rand already used)

**Implementation**:
```rust
use rand::Rng;

fn should_fail(error_rate: f64) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < error_rate
}
```

---

### 3. Latency Injection Approach

**Question**: How should latency be applied to requests?

**Decision**: Use `tokio::time::sleep()` at middleware start.

**Rationale**:
- Async sleep doesn't block the runtime
- Applied before handler means no partial state changes on timeout
- Jitter is simply added to base latency
- Efficient: no thread parking, just timer registration

**Implementation**:
```rust
async fn apply_latency(config: &LatencyConfig) {
    let base = config.base_latency_ms;
    let jitter = if config.jitter_ms > 0 {
        rand::thread_rng().gen_range(0..=config.jitter_ms)
    } else {
        0
    };

    tokio::time::sleep(Duration::from_millis(base + jitter)).await;
}
```

---

### 4. Endpoint Pattern Matching

**Question**: How should endpoint patterns be matched?

**Decision**: Simple glob-style matching with `*` wildcards.

**Rationale**:
- Full regex is overkill for most use cases
- Glob patterns are familiar to developers
- `/oss/v2/buckets/*` covers most scenarios
- Can use existing `glob` crate or simple manual matching

**Alternatives Considered**:
- **Full regex**: More powerful but complex to configure
- **Exact match only**: Too limiting for practical use
- **Route parameter matching**: Tied to axum internals

**Implementation**:
```rust
fn path_matches(pattern: &str, path: &str) -> bool {
    // Simple glob: * matches any segment, ** matches any path
    if pattern == "*" || pattern == "**" {
        return true;
    }

    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len()-2];
        return path.starts_with(prefix);
    }

    pattern == path
}
```

---

### 5. Error Response Format

**Question**: What format should injected error responses use?

**Decision**: Match APS API error format with configurable codes.

**Rationale**:
- Errors should be indistinguishable from real APS errors
- Applications already handle this format
- Includes error code, message, and optional details

**Error Format**:
```json
{
    "developerMessage": "Simulated error for testing",
    "errorCode": "CHAOS_INJECTION",
    "moreInfo": "https://forge.autodesk.com/en/docs/oauth/v2/developers_guide/error_handling/",
    "userMessage": "An error occurred while processing your request"
}
```

**Implementation**:
```rust
fn create_error_response(status: StatusCode) -> Response {
    let body = json!({
        "developerMessage": "Simulated error for testing",
        "errorCode": "CHAOS_INJECTION",
        "moreInfo": "...",
        "userMessage": "An error occurred"
    });

    (status, Json(body)).into_response()
}
```

---

### 6. Rate Limiter Strategy

**Question**: What rate limiting algorithm should be used?

**Decision**: Token bucket algorithm with sliding window.

**Rationale**:
- Token bucket is well-understood and predictable
- Allows bursts up to bucket size
- Sliding window prevents edge-case gaming
- Matches how real APIs implement rate limiting

**Alternatives Considered**:
- **Fixed window**: Simpler but allows double-burst at window boundaries
- **Leaky bucket**: Smoother but doesn't allow bursts
- **Per-IP tracking**: Unnecessary for mock server

**Implementation**:
```rust
pub struct RateLimiter {
    requests: DashMap<String, VecDeque<Instant>>,
    limit: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn check(&self, key: &str) -> bool {
        let mut timestamps = self.requests.entry(key.to_string())
            .or_insert_with(VecDeque::new);

        let now = Instant::now();
        let cutoff = now - self.window;

        // Remove old timestamps
        while timestamps.front().map_or(false, |t| *t < cutoff) {
            timestamps.pop_front();
        }

        if timestamps.len() >= self.limit {
            false
        } else {
            timestamps.push_back(now);
            true
        }
    }
}
```

---

### 7. Configuration Hot-Reload

**Question**: Should chaos configuration be changeable at runtime?

**Decision**: Yes, via `Arc<RwLock<ChaosConfig>>`.

**Rationale**:
- Tests may want to change chaos mid-scenario
- No server restart needed for adjustments
- RwLock allows concurrent reads with rare writes
- Library API exposes mutation methods

**Implementation**:
```rust
impl MockServer {
    pub fn set_error_rate(&self, rate: f64) {
        let mut config = self.chaos_config.write().unwrap();
        config.error_rate = rate;
    }

    pub fn set_latency(&self, base_ms: u64, jitter_ms: u64) {
        let mut config = self.chaos_config.write().unwrap();
        config.latency = Some(LatencyConfig { base_ms, jitter_ms });
    }
}
```

---

## Dependencies

### Required Crates (already in Cargo.toml)

- `rand` - Random number generation
- `tokio` - Async sleep for latency
- `dashmap` - Concurrent rate limiter state
- `serde` / `serde_json` - Error response serialization

### No New Dependencies Required

All functionality can be implemented with existing dependencies.

---

## Error Code Selection

When random errors are enabled, the following codes are used:

| Code | Description | Weight |
|------|-------------|--------|
| 400 | Bad Request | 15% |
| 401 | Unauthorized | 15% |
| 403 | Forbidden | 15% |
| 404 | Not Found | 15% |
| 500 | Internal Server Error | 20% |
| 502 | Bad Gateway | 10% |
| 503 | Service Unavailable | 10% |

Weights can be customized via configuration.

---

## Summary of Decisions

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| Architecture | Axum middleware layer | Early interception, no state pollution |
| RNG | thread_rng() | Efficient, sufficient randomness |
| Latency | tokio::time::sleep() | Async, applied before handler |
| Pattern Matching | Glob-style wildcards | Familiar, covers common cases |
| Error Format | APS-compatible JSON | Realistic for application testing |
| Rate Limiting | Token bucket | Standard, allows bursts |
| Hot-Reload | Arc<RwLock<Config>> | Runtime adjustability |

# API Contracts: Error and Latency Simulation

**Feature**: 003-error-simulation
**Date**: 2026-01-15
**Type**: Rust Library API + CLI Interface

## Overview

This document defines the Rust library API and CLI interface for chaos engineering features. These are configuration options, not HTTP endpoints.

---

## Public API

### MockServerConfig Extensions

```rust
impl MockServerConfig {
    /// Set chaos configuration for the server.
    ///
    /// # Arguments
    /// * `config` - Chaos configuration settings
    pub fn with_chaos(mut self, config: ChaosConfig) -> Self {
        self.chaos = Some(config);
        self
    }

    /// Set a simple error rate (shorthand).
    ///
    /// # Arguments
    /// * `rate` - Error probability (0.0 to 1.0)
    ///
    /// # Example
    /// ```rust
    /// let config = MockServerConfig::default()
    ///     .with_error_rate(0.1); // 10% of requests fail
    /// ```
    pub fn with_error_rate(mut self, rate: f64) -> Self {
        let chaos = self.chaos.get_or_insert_with(ChaosConfig::default);
        chaos.error_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set global latency (shorthand).
    ///
    /// # Arguments
    /// * `base_ms` - Base latency in milliseconds
    /// * `jitter_ms` - Random jitter range
    ///
    /// # Example
    /// ```rust
    /// let config = MockServerConfig::default()
    ///     .with_latency(500, 100); // 500-600ms delay
    /// ```
    pub fn with_latency(mut self, base_ms: u64, jitter_ms: u64) -> Self {
        let chaos = self.chaos.get_or_insert_with(ChaosConfig::default);
        chaos.latency = Some(LatencyConfig { base_ms, jitter_ms });
        self
    }
}
```

### MockServer Runtime Methods

```rust
impl MockServer {
    /// Get current chaos configuration.
    pub fn chaos_config(&self) -> ChaosConfig;

    /// Update error rate at runtime.
    ///
    /// # Arguments
    /// * `rate` - New error probability (0.0 to 1.0)
    pub fn set_error_rate(&self, rate: f64);

    /// Update latency settings at runtime.
    ///
    /// # Arguments
    /// * `base_ms` - Base latency in milliseconds
    /// * `jitter_ms` - Random jitter range
    pub fn set_latency(&self, base_ms: u64, jitter_ms: u64);

    /// Configure an endpoint to always fail.
    ///
    /// # Arguments
    /// * `pattern` - Path pattern to match
    /// * `status_code` - HTTP status to return
    pub fn fail_endpoint(&self, pattern: &str, status_code: u16);

    /// Remove a failing endpoint configuration.
    ///
    /// # Arguments
    /// * `pattern` - Path pattern to remove
    pub fn unfail_endpoint(&self, pattern: &str);

    /// Enable or disable all chaos features.
    ///
    /// # Arguments
    /// * `enabled` - Whether chaos is active
    pub fn set_chaos_enabled(&self, enabled: bool);

    /// Configure rate limiting.
    ///
    /// # Arguments
    /// * `requests_per_window` - Max requests allowed
    /// * `window_seconds` - Time window in seconds
    pub fn set_rate_limit(&self, requests_per_window: usize, window_seconds: u64);

    /// Disable rate limiting.
    pub fn clear_rate_limit(&self);
}
```

### TestServer Extensions

```rust
impl TestServer {
    /// Start a test server with chaos configuration.
    ///
    /// # Arguments
    /// * `chaos` - Chaos configuration
    ///
    /// # Example
    /// ```rust
    /// let chaos = ChaosConfig::with_error_rate(0.5);
    /// let server = TestServer::start_with_chaos(chaos).await?;
    /// ```
    pub async fn start_with_chaos(chaos: ChaosConfig) -> Result<Self, MockError>;

    /// Start with a simple error rate.
    ///
    /// # Example
    /// ```rust
    /// let server = TestServer::start_with_error_rate(0.1).await?;
    /// ```
    pub async fn start_with_error_rate(rate: f64) -> Result<Self, MockError>;

    /// Start with latency injection.
    ///
    /// # Example
    /// ```rust
    /// let server = TestServer::start_with_latency(500, 100).await?;
    /// ```
    pub async fn start_with_latency(base_ms: u64, jitter_ms: u64) -> Result<Self, MockError>;
}
```

---

## ChaosConfig Builder API

```rust
impl ChaosConfig {
    /// Create an empty chaos config (no chaos).
    pub fn none() -> Self;

    /// Create with just error rate.
    pub fn with_error_rate(rate: f64) -> Self;

    /// Create with just latency.
    pub fn with_latency(base_ms: u64, jitter_ms: u64) -> Self;

    /// Add a failing endpoint.
    pub fn fail_endpoint(self, pattern: &str, status_code: u16) -> Self;

    /// Add a failing endpoint with custom message.
    pub fn fail_endpoint_with_message(
        self,
        pattern: &str,
        status_code: u16,
        message: &str
    ) -> Self;

    /// Set endpoint-specific latency.
    pub fn endpoint_latency(
        self,
        pattern: &str,
        base_ms: u64,
        jitter_ms: u64
    ) -> Self;

    /// Configure rate limiting.
    pub fn rate_limit(
        self,
        requests_per_window: usize,
        window_seconds: u64
    ) -> Self;

    /// Set custom error code distribution.
    pub fn error_codes(self, codes: Vec<WeightedErrorCode>) -> Self;
}
```

---

## CLI Interface

### Basic Flags

```bash
# Error rate (percentage or decimal)
raps-mock --error-rate 10        # 10% failure rate
raps-mock --error-rate 0.1       # Same as above

# Latency
raps-mock --latency 500          # 500ms base latency
raps-mock --latency 500 --jitter 100   # 500-600ms latency

# Disable chaos (useful if config file has it)
raps-mock --no-chaos
```

### Advanced Configuration

```bash
# Fail specific endpoint
raps-mock --fail-endpoint "/oss/v2/buckets:503"

# Multiple failing endpoints
raps-mock --fail-endpoint "/oss/v2/buckets:503" \
          --fail-endpoint "/projects/v1/hubs/*:500"

# Rate limiting
raps-mock --rate-limit 100/60    # 100 requests per 60 seconds

# Combined
raps-mock --error-rate 0.05 \
          --latency 200 \
          --jitter 50 \
          --fail-endpoint "/modelderivative/**:503"
```

### Configuration File

```bash
# Load from config file
raps-mock --config chaos.yaml
```

**chaos.yaml**:
```yaml
chaos:
  error_rate: 0.1
  latency:
    base_ms: 500
    jitter_ms: 100
  failing_endpoints:
    - pattern: "/oss/v2/buckets/*"
      status_code: 503
```

---

## Error Responses

When chaos injects an error, the response follows APS error format:

```json
{
    "developerMessage": "Simulated error for chaos testing",
    "errorCode": "CHAOS_INJECTION",
    "moreInfo": "https://forge.autodesk.com/en/docs/oauth/v2/developers_guide/error_handling/",
    "userMessage": "An error occurred while processing your request"
}
```

### Rate Limit Response (429)

```json
{
    "developerMessage": "Rate limit exceeded. Try again later.",
    "errorCode": "RATE_LIMITED",
    "moreInfo": "...",
    "userMessage": "Too many requests"
}
```

**Headers**:
```
Retry-After: 30
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704067200
```

---

## Usage Examples

### CLI Usage

```bash
# Start with 10% error rate
raps-mock --port 3000 --error-rate 0.1

# Start with latency
raps-mock --port 3000 --latency 500 --jitter 100

# Start with specific endpoint failures
raps-mock --port 3000 --fail-endpoint "/oss/v2/buckets:503"

# Combined chaos settings
raps-mock --port 3000 \
    --error-rate 0.05 \
    --latency 200 \
    --fail-endpoint "/modelderivative/**:503"
```

### Library Usage

```rust
use raps_mock::{MockServer, MockServerConfig, ChaosConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Simple error rate
    let config = MockServerConfig::default()
        .with_error_rate(0.1);

    // Option 2: Full chaos config
    let chaos = ChaosConfig::with_error_rate(0.1)
        .with_latency(500, 100)
        .fail_endpoint("/oss/v2/buckets", 503);

    let config = MockServerConfig::default()
        .with_chaos(chaos);

    let server = MockServer::new(config).await?;
    server.start("0.0.0.0:3000").await?;
    Ok(())
}
```

### Test Usage

```rust
use raps_mock::{TestServer, ChaosConfig};

#[tokio::test]
async fn test_error_handling() {
    // Start with 50% error rate
    let server = TestServer::start_with_error_rate(0.5).await.unwrap();

    let client = reqwest::Client::new();
    let mut errors = 0;
    let mut successes = 0;

    for _ in 0..100 {
        let resp = client
            .get(&format!("{}/oss/v2/buckets", server.url))
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            successes += 1;
        } else {
            errors += 1;
        }
    }

    // Approximately 50/50 split (with some variance)
    assert!(errors > 30 && errors < 70);
}

#[tokio::test]
async fn test_timeout_handling() {
    // Start with 2 second latency
    let server = TestServer::start_with_latency(2000, 0).await.unwrap();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    let result = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await;

    // Should timeout
    assert!(result.is_err());
}

#[tokio::test]
async fn test_specific_endpoint_failure() {
    let chaos = ChaosConfig::default()
        .fail_endpoint("/oss/v2/buckets", 503);

    let server = TestServer::start_with_chaos(chaos).await.unwrap();

    let client = reqwest::Client::new();

    // This endpoint should fail
    let resp = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);

    // Other endpoints work normally
    let resp = client
        .get(&format!("{}/project/v1/hubs", server.url))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}
```

### Runtime Configuration

```rust
#[tokio::test]
async fn test_runtime_chaos_change() {
    let server = TestServer::start_default().await.unwrap();

    // Initially no chaos
    let resp = make_request(&server).await;
    assert!(resp.status().is_success());

    // Enable chaos at runtime
    server.set_error_rate(1.0); // 100% failure

    let resp = make_request(&server).await;
    assert!(resp.status().is_server_error());

    // Disable chaos
    server.set_error_rate(0.0);

    let resp = make_request(&server).await;
    assert!(resp.status().is_success());
}
```

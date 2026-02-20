# Quickstart: Error and Latency Simulation

**Feature**: 003-error-simulation
**Date**: 2026-01-15

This guide shows you how to use chaos engineering features to test your application's resilience.

---

## What is Chaos Engineering?

Chaos engineering lets you intentionally introduce failures and delays to test how your application handles adverse conditions. raps-mock supports:

- **Random Errors**: Configurable percentage of requests fail
- **Latency Injection**: Add delays to responses
- **Endpoint Failures**: Make specific endpoints always fail
- **Rate Limiting**: Simulate API throttling

---

## Quick Start

### 1. Start with Error Rate

Make 10% of requests fail randomly:

```bash
raps-mock --port 3000 --error-rate 10
```

Test it:
```bash
# Run this multiple times - some will fail
for i in {1..10}; do
  curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3000/oss/v2/buckets
done
```

### 2. Add Latency

Add 500ms delay with 100ms jitter (500-600ms total):

```bash
raps-mock --port 3000 --latency 500 --jitter 100
```

Test it:
```bash
time curl http://localhost:3000/oss/v2/buckets
# Should take ~500-600ms
```

### 3. Fail Specific Endpoints

Make the buckets endpoint return 503:

```bash
raps-mock --port 3000 --fail-endpoint "/oss/v2/buckets:503"
```

Test it:
```bash
# This fails with 503
curl -i http://localhost:3000/oss/v2/buckets

# This still works
curl -i http://localhost:3000/project/v1/hubs
```

---

## Common Scenarios

### Test Retry Logic

Set a moderate error rate to trigger retries:

```bash
raps-mock --error-rate 30
```

Your application should:
- Retry failed requests
- Eventually succeed (most of the time)
- Not crash on errors

### Test Timeout Handling

Set latency higher than your application's timeout:

```bash
# If your app times out after 5 seconds
raps-mock --latency 6000
```

Your application should:
- Timeout gracefully
- Show appropriate error message
- Not hang indefinitely

### Simulate Service Degradation

Combine error rate and latency:

```bash
raps-mock --error-rate 5 --latency 1000 --jitter 500
```

This simulates a degraded service that's slow and occasionally fails.

### Test Rate Limit Handling

Enable rate limiting:

```bash
raps-mock --rate-limit 10/60  # 10 requests per minute
```

Make many requests quickly:
```bash
for i in {1..15}; do
  curl -s -o /dev/null -w "%{http_code} " http://localhost:3000/oss/v2/buckets
done
# Output: 200 200 200 200 200 200 200 200 200 200 429 429 429 429 429
```

---

## Using in Tests

### Rust Integration Tests

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_app_handles_errors() {
    // Start mock with 50% error rate
    let server = TestServer::start_with_error_rate(0.5).await.unwrap();

    let client = reqwest::Client::new();
    let mut success_count = 0;

    for _ in 0..100 {
        let resp = client
            .get(&format!("{}/oss/v2/buckets", server.url))
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            success_count += 1;
        }
    }

    // Should have roughly 50% success rate
    assert!(success_count > 30 && success_count < 70);
}

#[tokio::test]
async fn test_app_handles_timeouts() {
    // Start mock with 2 second latency
    let server = TestServer::start_with_latency(2000, 0).await.unwrap();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    let result = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await;

    // Request should timeout
    assert!(result.is_err());
}
```

### Change Chaos at Runtime

```rust
#[tokio::test]
async fn test_dynamic_chaos() {
    let server = TestServer::start_default().await.unwrap();

    // No chaos initially
    assert_request_succeeds(&server).await;

    // Enable 100% errors
    server.set_error_rate(1.0);
    assert_request_fails(&server).await;

    // Disable errors
    server.set_error_rate(0.0);
    assert_request_succeeds(&server).await;
}
```

---

## Configuration Reference

### CLI Options

| Flag | Description | Example |
|------|-------------|---------|
| `--error-rate` | Percentage of requests to fail | `--error-rate 10` |
| `--latency` | Base latency in milliseconds | `--latency 500` |
| `--jitter` | Latency jitter range | `--jitter 100` |
| `--fail-endpoint` | Endpoint to always fail | `--fail-endpoint "/path:503"` |
| `--rate-limit` | Rate limit (requests/seconds) | `--rate-limit 100/60` |
| `--no-chaos` | Disable all chaos features | `--no-chaos` |

### Programmatic Configuration

```rust
use raps_mock::{MockServerConfig, ChaosConfig};

// Simple error rate
let config = MockServerConfig::default()
    .with_error_rate(0.1);

// Full configuration
let chaos = ChaosConfig::with_error_rate(0.1)
    .with_latency(500, 100)
    .fail_endpoint("/oss/v2/buckets", 503)
    .rate_limit(100, 60);

let config = MockServerConfig::default()
    .with_chaos(chaos);
```

---

## Error Response Format

When chaos injects an error, the response looks like:

```json
{
    "developerMessage": "Simulated error for chaos testing",
    "errorCode": "CHAOS_INJECTION",
    "moreInfo": "https://forge.autodesk.com/en/docs/oauth/v2/developers_guide/error_handling/",
    "userMessage": "An error occurred while processing your request"
}
```

This matches the standard APS error format, so your error handling code works the same way.

---

## Best Practices

### 1. Start Small

Begin with low error rates (1-5%) to validate basic error handling before ramping up.

### 2. Test Specific Scenarios

Use endpoint failures for targeted testing:
```bash
# Test what happens when auth fails
raps-mock --fail-endpoint "/authentication/*:401"

# Test what happens when a specific API is down
raps-mock --fail-endpoint "/modelderivative/**:503"
```

### 3. Combine Realistically

Real degradation often involves both errors and latency:
```bash
# Realistic degraded service
raps-mock --error-rate 5 --latency 1000 --jitter 500
```

### 4. Test Edge Cases

- 100% error rate: Does your app handle total failure?
- Very high latency: Does your app timeout properly?
- Rate limiting: Does your app respect Retry-After?

---

## Troubleshooting

### Errors Not Being Injected

- Check `--error-rate` is between 0-100 (or 0.0-1.0)
- Verify chaos is enabled (not using `--no-chaos`)
- Low error rates may need many requests to see failures

### Latency Not Working

- Check `--latency` value is in milliseconds
- Verify the server started with the flag
- Run `time curl ...` to measure actual delay

### Wrong Error Code

- Default errors are randomly selected (400, 401, 403, 404, 500, 502, 503)
- Use `--fail-endpoint "/path:CODE"` for specific codes

---

## Next Steps

- See [data-model.md](./data-model.md) for full configuration schema
- See [contracts/api.md](./contracts/api.md) for the complete library API
- Run `/speckit.tasks` to generate implementation tasks

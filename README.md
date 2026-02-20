# raps-mock: APS API Mock Server

A comprehensive mock server for Autodesk Platform Services (APS) APIs, auto-generated from OpenAPI specifications.

## Features

- **Auto-generated routes** from OpenAPI 3.0 specifications
- **Configurable modes**: Stateless (fixed responses) or Stateful (in-memory storage)
- **Library and CLI**: Use as a library or standalone server
- **Full APS coverage**: Authentication, OSS, Data Management, Model Derivative, Construction, Webhooks
- **Custom handlers**: Extend with custom endpoint handlers
- **Type-safe**: Built with Rust for reliability and performance

## Why raps-mock?

**The Problem:** Developing against live APS APIs requires valid credentials, costs tokens for Model Derivative operations, and creates test data pollution in your ACC environment. Network issues can derail your development session. CI/CD pipelines become flaky when dependent on external services.

**The Solution:** raps-mock provides a fully functional APS mock server that runs locally or in your CI environment. Test your integrations without credentials, without costs, without network dependencies.

### Use Cases

- **Local Development**: Work offline, iterate quickly, no credential setup for new team members
- **CI/CD Testing**: Reliable, deterministic tests that don't depend on external services
- **Demo Environments**: Show APS features without real data or credentials
- **Learning**: Experiment with APS APIs without consequences

## Quick Start

### As a Standalone Server

```bash
# Start the mock server (stateful mode by default)
raps-mock --port 3000

# Start in stateless mode (fixed OpenAPI example responses)
raps-mock --port 3000 --mode stateless

# With custom OpenAPI directory
raps-mock --openapi-dir ./my-openapi-specs --port 3000
```

### As a Library

```rust
use raps_mock::{MockServer, MockServerConfig, MockMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MockServerConfig {
        mode: MockMode::Stateful,
        openapi_dir: "../aps-sdk-openapi".into(),
        ..Default::default()
    };

    let server = MockServer::new(config).await?;
    server.start("0.0.0.0:3000").await?;
    Ok(())
}
```

### In Integration Tests

```rust
use raps_mock::testing::TestServer;

#[tokio::test]
async fn test_bucket_creation() {
    // Start mock server on random port
    let server = TestServer::start_default().await.unwrap();
    
    // Use any HTTP client against server.url
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/oss/v2/buckets", server.url))
        .json(&serde_json::json!({
            "bucketKey": "test-bucket",
            "policyKey": "transient"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}
```

## Command Line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--port` | `-p` | 3000 | Server port |
| `--host` | `-H` | 0.0.0.0 | Server host |
| `--mode` | `-m` | stateful | `stateless` or `stateful` |
| `--openapi-dir` | | ../aps-sdk-openapi | Path to OpenAPI specs |
| `--state-file` | | - | Path to state persistence file |
| `--verbose` | `-v` | false | Enable verbose logging |

## Supported APIs

| API | Version | Status | Notes |
|-----|---------|--------|-------|
| Authentication | v2 | ✅ Full | 2-legged and 3-legged OAuth simulation |
| OSS (Buckets) | v2 | ✅ Full | Create, list, delete buckets |
| OSS (Objects) | v2 | ✅ Full | Upload metadata, listing, signed URLs |
| Data Management | v1 | ✅ Full | Hubs, projects, folders, items |
| Model Derivative | v2 | ✅ Full | Translation jobs, manifests, status |
| Construction Issues | v1 | ✅ Full | Issues CRUD, comments |
| ACC Account Admin | v1 | ✅ Partial | Basic account operations |
| Webhooks | v1 | ✅ Full | Subscription management |
| ACC Submittals | v1 | 🚧 Planned | Phase 1 |
| ACC Checklists | v1 | 🚧 Planned | Phase 1 |
| ACC RFIs | v1 | 🚧 Planned | Phase 1 |
| ACC Assets | v1 | 🚧 Planned | Phase 1 |
| Design Automation | v3 | 🚧 Planned | Phase 4 |
| Reality Capture | v1 | 🚧 Planned | Phase 4 |

## Modes Explained

### Stateful Mode (Default)

In stateful mode, the mock server maintains in-memory state. Operations have side effects:

- Creating a bucket adds it to the list
- Uploading an object associates it with a bucket
- Starting a translation creates a job with status that can be queried
- Deleting resources removes them from state

This mode is ideal for testing workflows that span multiple API calls.

### Stateless Mode

In stateless mode, every request returns fixed responses based on OpenAPI specification examples. No state is maintained between requests.

This mode is useful for:
- Simple contract testing
- Validating request/response formats
- Scenarios where state persistence isn't needed

## Integration with raps CLI

raps-mock is designed to work seamlessly with [raps CLI](https://github.com/dmytro-yemelianov/raps):

```bash
# Terminal 1: Start mock server
raps-mock --port 3000

# Terminal 2: Configure raps to use mock server
export APS_BASE_URL=http://localhost:3000
export APS_CLIENT_ID=mock-client
export APS_CLIENT_SECRET=mock-secret

# Now raps commands work against the mock
raps auth test
raps bucket create --key test-bucket --policy transient
raps bucket list
```

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for the detailed evolution plan including:

- **Phase 1 (v0.3.0)**: Complete ACC module coverage (Submittals, Checklists, RFIs, Assets)
- **Phase 2 (v0.4.0)**: State persistence and fixtures
- **Phase 3 (v0.5.0)**: Simulation capabilities (latency, errors, progress)
- **Phase 4 (v0.6.0)**: Design Automation and Reality Capture
- **Phase 5 (v0.7.0)**: Developer experience (recording, dashboard, validation)
- **Phase 6 (v1.0.0)**: Docker, GitHub Actions, production release

## Contributing

Contributions are welcome! See [PHASE1_ISSUES.md](./PHASE1_ISSUES.md) for immediate implementation opportunities.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/dmytro-yemelianov/raps-mock.git
cd raps-mock

# Also clone the OpenAPI specs
git clone https://github.com/autodesk-platform-services/aps-sdk-openapi.git ../aps-sdk-openapi

# Build
cargo build

# Run tests
cargo test

# Run the server
cargo run -- --port 3000 --verbose
```

## License

Apache 2.0 License

## Related Projects

- [raps](https://github.com/dmytro-yemelianov/raps) - Rust APS CLI
- [raps-website](https://github.com/dmytro-yemelianov/raps-website) - Documentation site
- [aps-sdk-openapi](https://github.com/autodesk-platform-services/aps-sdk-openapi) - Official APS OpenAPI specs

# raps-mock: APS API Mock Server

A comprehensive mock server for Autodesk Platform Services (APS) APIs, auto-generated from OpenAPI specifications.

## Features

- **Auto-generated routes** from OpenAPI 3.0 specifications
- **Configurable modes**: Stateless (fixed responses) or Stateful (in-memory storage)
- **Library and CLI**: Use as a library or standalone server
- **Full APS coverage**: Authentication, OSS, Data Management, Model Derivative, Construction, Webhooks
- **Custom handlers**: Extend with custom endpoint handlers
- **Type-safe**: Built with Rust for reliability and performance

## Quick Start

### As a Standalone Server

```bash
# Start the mock server
raps-mock --port 3000 --mode stateful

# With custom OpenAPI directory
raps-mock --openapi-dir ../aps-sdk-openapi --port 3000
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

## Command Line Options

- `--port` / `-p`: Server port (default: 3000)
- `--host` / `-H`: Server host (default: 0.0.0.0)
- `--mode` / `-m`: `stateless` or `stateful` (default: stateful)
- `--openapi-dir`: Path to OpenAPI specs (default: ../aps-sdk-openapi)
- `--state-file`: Path to state persistence file (optional)
- `--verbose` / `-v`: Enable verbose logging

## Supported APIs

- Authentication API v2 - OAuth 2.0 flows
- OSS API v2 - Buckets and objects
- Data Management API v1 - Hubs, projects, folders, items
- Model Derivative API v2 - Translation jobs, manifests
- Construction Issues API v1 - ACC Issues
- ACC Account Admin API v1
- Webhooks API v1 - Event subscriptions

## Integration with raps Project

This mock server can be used as a drop-in replacement for wiremock in integration tests, providing consistent mock responses across all raps crates.

## License

Apache 2.0 License

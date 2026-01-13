// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Basic usage example for raps-mock library

use raps_mock::{MockMode, MockServer, MockServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create server configuration
    let config = MockServerConfig {
        mode: MockMode::Stateful,
        openapi_dir: "../aps-sdk-openapi".into(),
        state_file: None,
        verbose: true,
        host: "0.0.0.0".to_string(),
        port: 3000,
    };

    // Create and start the server
    let server = MockServer::new(config).await?;
    println!("Starting mock server on http://0.0.0.0:3000");
    server.start("0.0.0.0:3000").await?;

    Ok(())
}

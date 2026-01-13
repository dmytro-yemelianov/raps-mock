// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use raps_mock::{MockMode, MockServer, MockServerConfig};
use std::path::PathBuf;

#[tokio::test]
async fn test_server_creation() {
    let config = MockServerConfig {
        mode: MockMode::Stateful,
        openapi_dir: PathBuf::from("../aps-sdk-openapi"),
        state_file: None,
        verbose: false,
        host: "127.0.0.1".to_string(),
        port: 0, // Let OS choose port
    };

    let server = MockServer::new(config).await;
    assert!(server.is_ok());
}

#[tokio::test]
async fn test_stateless_mode() {
    let config = MockServerConfig {
        mode: MockMode::Stateless,
        openapi_dir: PathBuf::from("../aps-sdk-openapi"),
        state_file: None,
        verbose: false,
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let server = MockServer::new(config).await;
    assert!(server.is_ok());
}

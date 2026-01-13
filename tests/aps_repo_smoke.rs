// SPDX-License-Identifier: Apache-2.0
// Basic smoke test that parses the external APS OpenAPI repo.

#![cfg(feature = "aps_ci")]

use std::path::PathBuf;

use raps_mock::{MockMode, MockServer, MockServerConfig};

#[tokio::test]
async fn aps_repo_parses_and_builds_router() {
    let dir = std::env::var("APS_OPENAPI_DIR")
        .map(PathBuf::from)
        .expect("APS_OPENAPI_DIR env var must be set in CI");

    let config = MockServerConfig {
        mode: MockMode::Stateless,
        openapi_dir: dir,
        state_file: None,
        verbose: false,
        host: "127.0.0.1".into(),
        port: 0,
    };

    let server = MockServer::new(config).await;
    assert!(server.is_ok());
}

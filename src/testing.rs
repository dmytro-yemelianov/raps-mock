// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Test utilities for using raps-mock in integration tests.
//!
//! This module provides a convenient `TestServer` helper that starts a mock server
//! in the background on a random port, making it easy to use in tests.
//!
//! # Example
//!
//! ```rust,no_run
//! use raps_mock::testing::TestServer;
//!
//! #[tokio::test]
//! async fn test_api() {
//!     let server = TestServer::start_default().await.unwrap();
//!     let client = reqwest::Client::new();
//!     let response = client.get(&format!("{}/oss/v2/buckets", server.url))
//!         .send()
//!         .await
//!         .unwrap();
//!     assert!(response.status().is_success());
//! }
//! ```

use crate::config::{MockMode, MockServerConfig};
use crate::error::Result;
use crate::server::MockServer;
use std::path::PathBuf;
use tokio::net::TcpListener;

/// A test server that runs in the background on a random port.
///
/// The server is automatically started when created and runs until dropped.
pub struct TestServer {
    /// The base URL of the running server (e.g., "http://127.0.0.1:12345")
    pub url: String,
    /// Handle to the background task running the server
    _task: tokio::task::JoinHandle<()>,
}

impl TestServer {
    /// Start a test server with the given configuration.
    ///
    /// The server binds to a random available port on localhost.
    pub async fn start(config: MockServerConfig) -> Result<Self> {
        let server = MockServer::new(config).await?;
        let app = server.router();

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Ok(Self {
            url: format!("http://{}", addr),
            _task: task,
        })
    }

    /// Start a test server with default configuration.
    ///
    /// Uses stateful mode and looks for OpenAPI specs in `../aps-sdk-openapi`.
    pub async fn start_default() -> Result<Self> {
        Self::start(MockServerConfig::default()).await
    }

    /// Start a test server with a custom OpenAPI directory.
    pub async fn start_with_openapi_dir(openapi_dir: PathBuf) -> Result<Self> {
        let config = MockServerConfig {
            mode: MockMode::Stateful,
            openapi_dir,
            state_file: None,
            verbose: false,
            host: "127.0.0.1".to_string(),
            port: 0,
        };
        Self::start(config).await
    }

    /// Start a test server in stateless mode.
    ///
    /// Stateless mode returns fixed example responses from OpenAPI specs.
    pub async fn start_stateless() -> Result<Self> {
        let config = MockServerConfig {
            mode: MockMode::Stateless,
            ..MockServerConfig::default()
        };
        Self::start(config).await
    }

    /// Get the base URL of the server.
    pub fn uri(&self) -> &str {
        &self.url
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self._task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_on_random_port() {
        let server = TestServer::start_default().await;
        assert!(server.is_ok());
        let server = server.unwrap();
        assert!(server.url.starts_with("http://127.0.0.1:"));
    }

    #[tokio::test]
    async fn test_server_uri_method() {
        let server = TestServer::start_default().await.unwrap();
        assert_eq!(server.uri(), &server.url);
    }
}

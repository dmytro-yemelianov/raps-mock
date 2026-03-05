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
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Test server failed: {}", e);
            }
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
            openapi_dir,
            ..MockServerConfig::default()
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

use crate::trace::{ApiCall, TraceRecorder};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Variant of TestServer that also records every write API call.
pub struct TestServerWithTrace {
    /// Base URL of the running server.
    pub url: String,
    /// Shared trace recorder — query this after running operations.
    pub trace: TraceRecorder,
    _task: tokio::task::JoinHandle<()>,
}

impl Drop for TestServerWithTrace {
    fn drop(&mut self) {
        self._task.abort();
    }
}

/// Axum middleware that records write operations into a TraceRecorder.
async fn recording_middleware(
    axum::Extension(recorder): axum::Extension<TraceRecorder>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    // Record POST, PATCH, PUT, DELETE — skip GET and the auth endpoint
    if matches!(method.as_str(), "POST" | "PATCH" | "PUT" | "DELETE")
        && path != "/authentication/v2/token"
    {
        recorder.record(ApiCall { method, path });
    }
    next.run(request).await
}

impl TestServer {
    /// Start a server that also records all write-API calls.
    /// Access recorded calls via `TestServerWithTrace::trace`.
    pub async fn start_with_trace() -> Result<TestServerWithTrace> {
        let config = MockServerConfig::default();
        let server = MockServer::new(config).await?;

        let recorder = TraceRecorder::new();
        let app = server
            .router()
            .layer(axum::middleware::from_fn(recording_middleware))
            .layer(axum::Extension(recorder.clone()));

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let task = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Trace test server failed: {}", e);
            }
        });

        Ok(TestServerWithTrace {
            url: format!("http://{}", addr),
            trace: recorder,
            _task: task,
        })
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

#[cfg(test)]
mod trace_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_with_trace_records_post_calls() {
        let ts = TestServer::start_with_trace().await.unwrap();
        let client = reqwest::Client::new();

        // Get a real token
        let resp = client
            .post(format!("{}/authentication/v2/token", ts.url))
            .json(&serde_json::json!({
                "client_id": "test-client",
                "client_secret": "test-secret",
                "grant_type": "client_credentials"
            }))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        let token = body["access_token"].as_str().unwrap().to_string();

        // Make a POST that should be recorded
        client
            .post(format!(
                "{}/construction/admin/v1/projects/proj-001/users",
                ts.url
            ))
            .bearer_auth(&token)
            .json(&serde_json::json!({"email": "trace@test.com", "roleId": "role-admin"}))
            .send()
            .await
            .unwrap();

        ts.trace.assert_call_count(1);
        ts.trace.assert_called_with("POST", "/projects/proj-001/users");
    }
}

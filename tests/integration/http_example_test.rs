// SPDX-License-Identifier: Apache-2.0
// Verifies HTTP responses via a temporary OpenAPI spec

use raps_mock::{MockMode, MockServer, MockServerConfig};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn http_hello_example() {
    // Arrange: temp OpenAPI spec with a JSON example
    let dir = tempdir().unwrap();
    let spec_path = dir.path().join("hello.yaml");
    let spec_content = r#"
openapi: 3.0.0
info:
  title: Hello
  version: 1.0.0
paths:
  /hello:
    get:
      summary: Say hello
      responses:
        "200":
          description: OK
          content:
            application/json:
              example:
                message: "Hello, world!"
"#;
    fs::write(&spec_path, spec_content).unwrap();

    // Build server and get its router
    let config = MockServerConfig {
        mode: MockMode::Stateless,
        openapi_dir: dir.path().to_path_buf(),
        state_file: None,
        verbose: false,
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let server = MockServer::new(config).await.expect("server");
    let app = server.router();

    // Bind to a random local port and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Act: call the endpoint
    let url = format!("http://{}/hello", addr);
    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["message"], "Hello, world!");

    // Cleanup: cancel the server task
    server_task.abort();
}


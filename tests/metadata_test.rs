// SPDX-License-Identifier: Apache-2.0
// Integration test for Model Derivative metadata endpoints (T015)

use raps_mock::{MockMode, MockServer, MockServerConfig};
use std::path::PathBuf;

/// Helper: start a stateful mock server on a random port, return (base_url, server_task)
async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let config = MockServerConfig {
        mode: MockMode::Stateful,
        openapi_dir: PathBuf::from("../aps-sdk-openapi"),
        db_path: None,
        verbose: false,
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let server = MockServer::new(config).await.expect("server");
    let app = server.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}", addr), task)
}

/// Get a valid Bearer token from the mock auth endpoint
async fn get_token(client: &reqwest::Client, base: &str) -> String {
    let resp = client
        .post(&format!("{}/authentication/v2/token", base))
        .json(&serde_json::json!({
            "client_id": "test-client",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_metadata_endpoints() {
    let (base, task) = start_test_server().await;
    let client = reqwest::Client::new();
    let token = get_token(&client, &base).await;

    // 1. Create a translation job
    let urn = "dXJuOmFkc2sub2JqZWN0czpvcy5vYmplY3Q6dGVzdC9tb2RlbC5ydnQ"; // base64 URN
    let resp = client
        .post(&format!("{}/modelderivative/v2/designdata/job", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "input": { "urn": urn },
            "output": { "formats": [{ "type": "svf2" }] }
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create job failed: {}",
        resp.status()
    );

    // 2. Simulate progress to Success (calls: pending→inprogress→50%→75%→100%→success)
    for _ in 0..5 {
        let resp = client
            .get(&format!(
                "{}/modelderivative/v2/designdata/{}/manifest",
                base, urn
            ))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }

    // 3. GET metadata
    let resp = client
        .get(&format!(
            "{}/modelderivative/v2/designdata/{}/metadata",
            base, urn
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "metadata GET failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"]["metadata"].is_array(),
        "metadata should be array"
    );
    let guid = body["data"]["metadata"][0]["guid"]
        .as_str()
        .expect("guid should exist");

    // 4. GET object tree
    let resp = client
        .get(&format!(
            "{}/modelderivative/v2/designdata/{}/metadata/{}",
            base, urn, guid
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "object tree GET failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"]["objects"].is_array(),
        "objects should be array"
    );

    // 5. GET properties
    let resp = client
        .get(&format!(
            "{}/modelderivative/v2/designdata/{}/metadata/{}/properties",
            base, urn, guid
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "properties GET failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"]["collection"].is_array(),
        "collection should be array"
    );

    // 6. POST query properties (filter by object IDs)
    let resp = client
        .post(&format!(
            "{}/modelderivative/v2/designdata/{}/metadata/{}/properties:query",
            base, urn, guid
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "query": {
                "$in": ["objectid", 2, 3]
            }
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "query properties POST failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    let collection = body["data"]["collection"].as_array().unwrap();
    assert_eq!(collection.len(), 2, "should filter to 2 objects");

    task.abort();
}

#[tokio::test]
async fn test_metadata_not_found_before_success() {
    let (base, task) = start_test_server().await;
    let client = reqwest::Client::new();
    let token = get_token(&client, &base).await;

    // Create a translation but don't advance to success
    let urn = "dXJuOmFkc2sub2JqZWN0czpvcy5vYmplY3Q6dGVzdC9ub3RyZWFkeQ";
    let resp = client
        .post(&format!("{}/modelderivative/v2/designdata/job", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "input": { "urn": urn },
            "output": { "formats": [{ "type": "svf2" }] }
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Metadata should return 404 since translation is not yet complete
    let resp = client
        .get(&format!(
            "{}/modelderivative/v2/designdata/{}/metadata",
            base, urn
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);

    task.abort();
}

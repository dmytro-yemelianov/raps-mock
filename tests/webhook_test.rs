// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use raps_mock::TestServer;
use serde_json::{json, Value};

async fn get_token(client: &reqwest::Client, base: &str) -> String {
    let resp = client
        .post(format!("{}/authentication/v2/token", base))
        .json(&json!({
            "client_id": "test-client",
            "client_secret": "test-secret",
            "grant_type": "client_credentials"
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_webhook_requires_callback_url() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    let resp = client
        .post(format!(
            "{}/webhooks/v1/systems/data/events/dm.version.added/hooks",
            server.url
        ))
        .bearer_auth(&token)
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["reason"], "callbackUrl is required");
}

#[tokio::test]
async fn test_create_webhook_accepts_callback_url() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    let callback = "https://example.com/webhook";
    let resp = client
        .post(format!(
            "{}/webhooks/v1/systems/data/events/dm.version.added/hooks",
            server.url
        ))
        .bearer_auth(&token)
        .json(&json!({
            "callbackUrl": callback
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["callbackUrl"], callback);
}

// SPDX-License-Identifier: Apache-2.0
// Integration test for DA appbundle upload endpoint (T032)

use raps_mock::{MockMode, MockServer, MockServerConfig};
use std::path::PathBuf;

async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let config = MockServerConfig {
        mode: MockMode::Stateful,
        openapi_dir: PathBuf::from("../aps-sdk-openapi"),
        db_path: None,
        verbose: false,
        host: "127.0.0.1".to_string(),
        port: 0,
        ..MockServerConfig::default()
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
async fn test_appbundle_upload_flow() {
    let (base, task) = start_test_server().await;
    let client = reqwest::Client::new();
    let token = get_token(&client, &base).await;

    // 1. Create an appbundle (requires auth)
    let resp = client
        .post(&format!("{}/da/us-east/v3/appbundles", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "id": "TestBundle",
            "engine": "Autodesk.Revit+2024",
            "description": "Test bundle for upload"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create appbundle failed: {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.unwrap();

    // 2. Verify uploadParameters are present
    let upload_params = &body["uploadParameters"];
    assert!(
        upload_params.is_object(),
        "uploadParameters should be present in response"
    );
    let endpoint_url = upload_params["endpointUrl"]
        .as_str()
        .expect("endpointUrl should be a string");
    assert!(
        endpoint_url.contains("mock-s3-upload"),
        "endpointUrl should point to mock-s3-upload"
    );
    let form_data = &upload_params["formData"];
    assert!(form_data.is_object(), "formData should be present");

    // 3. POST multipart form to the upload URL (no auth needed — mock S3)
    let full_url = format!("{}{}", base, endpoint_url);
    let form = reqwest::multipart::Form::new()
        .text("key", form_data["key"].as_str().unwrap_or("").to_string())
        .text(
            "policy",
            form_data["policy"].as_str().unwrap_or("").to_string(),
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(b"PK\x03\x04fake zip content".to_vec())
                .file_name("bundle.zip"),
        );

    let resp = client.post(&full_url).multipart(form).send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "upload to mock-s3-upload failed: {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");

    task.abort();
}

#[tokio::test]
async fn test_upload_empty_body_rejected() {
    let (base, task) = start_test_server().await;

    // POST empty body to mock-s3-upload should fail (no auth needed)
    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("{}/mock-s3-upload/some-bundle", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 400, "empty body should be rejected");

    task.abort();
}

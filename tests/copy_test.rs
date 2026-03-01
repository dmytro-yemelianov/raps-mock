// SPDX-License-Identifier: Apache-2.0
// Integration test for OSS server-side copy endpoint (T019)

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
        .post(format!("{}/authentication/v2/token", base))
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
async fn test_copy_object() {
    let (base, task) = start_test_server().await;
    let client = reqwest::Client::new();
    let token = get_token(&client, &base).await;

    let src_bucket = "test-src-bucket";
    let dest_bucket = "test-dest-bucket";
    let src_key = "model.rvt";
    let dest_key = "model-copy.rvt";

    // 1. Create source bucket
    let resp = client
        .post(format!("{}/oss/v2/buckets", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "bucketKey": src_bucket,
            "policyKey": "transient"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create src bucket failed: {}",
        resp.status()
    );

    // 2. Create destination bucket
    let resp = client
        .post(format!("{}/oss/v2/buckets", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "bucketKey": dest_bucket,
            "policyKey": "transient"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create dest bucket failed: {}",
        resp.status()
    );

    // 3. Upload source object via signed S3 flow
    let resp = client
        .get(format!(
            "{}/oss/v2/buckets/{}/objects/{}/signeds3upload",
            base, src_bucket, src_key
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "get signed upload URL failed");
    let signed: serde_json::Value = resp.json().await.unwrap();
    let upload_url = signed["urls"][0].as_str().unwrap();

    // PUT data to the signed URL (mock-s3 endpoint bypasses auth)
    let resp = client
        .put(upload_url)
        .body(b"test file content".to_vec())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "PUT to signed URL failed");

    // Complete the upload
    let upload_key = signed["uploadKey"].as_str().unwrap();
    let resp = client
        .post(format!(
            "{}/oss/v2/buckets/{}/objects/{}/signeds3upload",
            base, src_bucket, src_key
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "uploadKey": upload_key }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "complete upload failed");

    // 4. Copy object via PUT with x-ads-copy-from header
    let copy_from = format!("{}/objects/{}", src_bucket, src_key);
    let resp = client
        .put(format!(
            "{}/oss/v2/buckets/{}/objects/{}",
            base, dest_bucket, dest_key
        ))
        .bearer_auth(&token)
        .header("x-ads-copy-from", &copy_from)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "copy object failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["bucketKey"], dest_bucket);
    assert_eq!(body["objectKey"], dest_key);

    // 5. Verify destination object exists by checking list objects
    let resp = client
        .get(format!("{}/oss/v2/buckets/{}/objects", base, dest_bucket))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "list objects failed");
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert!(
        items.iter().any(|item| item["objectKey"] == dest_key),
        "destination object should appear in bucket listing after copy"
    );

    task.abort();
}

#[tokio::test]
async fn test_copy_nonexistent_source() {
    let (base, task) = start_test_server().await;
    let client = reqwest::Client::new();
    let token = get_token(&client, &base).await;

    // Create destination bucket
    let resp = client
        .post(format!("{}/oss/v2/buckets", base))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "bucketKey": "dest-bucket",
            "policyKey": "transient"
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Try to copy from non-existent source
    let resp = client
        .put(format!(
            "{}/oss/v2/buckets/dest-bucket/objects/target.rvt",
            base
        ))
        .bearer_auth(&token)
        .header("x-ads-copy-from", "nonexistent/objects/missing.rvt")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);

    task.abort();
}

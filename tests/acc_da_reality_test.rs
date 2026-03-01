// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov
//
// Integration tests for ACC (RFI, Asset, Submittal, Checklist),
// Design Automation, and Reality Capture endpoints.

use raps_mock::TestServer;
use serde_json::{json, Value};

/// Get a valid Bearer token from the mock auth endpoint
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
    assert!(resp.status().is_success(), "token request failed: {}", resp.status());
    let body: Value = resp.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

// ── ACC: RFIs ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_rfi_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "test-project-001";

    // List (includes pre-seeded data)
    let resp = client
        .get(format!(
            "{}/construction/rfis/v2/projects/{}/rfis",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create
    let resp = client
        .post(format!(
            "{}/construction/rfis/v2/projects/{}/rfis",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"title": "Test RFI", "description": "Integration test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let rfi_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["title"], "Test RFI");
    assert_eq!(body["status"], "open");

    // Get
    let resp = client
        .get(format!(
            "{}/construction/rfis/v2/projects/{}/rfis/{}",
            server.url, project, rfi_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Test RFI");

    // Update
    let resp = client
        .patch(format!(
            "{}/construction/rfis/v2/projects/{}/rfis/{}",
            server.url, project, rfi_id
        ))
        .bearer_auth(&token)
        .json(&json!({"title": "Updated RFI", "status": "answered"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Updated RFI");
    assert_eq!(body["status"], "answered");

    // Delete
    let resp = client
        .delete(format!(
            "{}/construction/rfis/v2/projects/{}/rfis/{}",
            server.url, project, rfi_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify deleted (404)
    let resp = client
        .get(format!(
            "{}/construction/rfis/v2/projects/{}/rfis/{}",
            server.url, project, rfi_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ── ACC: Assets ────────────────────────────────────────────────────

#[tokio::test]
async fn test_asset_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "test-project-002";

    // Create
    let resp = client
        .post(format!(
            "{}/construction/assets/v1/projects/{}/assets",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"title": "HVAC Unit", "description": "Rooftop unit"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let asset_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["status"], "active");

    // List
    let resp = client
        .get(format!(
            "{}/construction/assets/v1/projects/{}/assets",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.iter().any(|r| r["id"] == asset_id));

    // Update
    let resp = client
        .patch(format!(
            "{}/construction/assets/v1/projects/{}/assets/{}",
            server.url, project, asset_id
        ))
        .bearer_auth(&token)
        .json(&json!({"status": "retired"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "retired");

    // Delete
    let resp = client
        .delete(format!(
            "{}/construction/assets/v1/projects/{}/assets/{}",
            server.url, project, asset_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// ── ACC: Submittals ────────────────────────────────────────────────

#[tokio::test]
async fn test_submittal_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "test-project-003";

    // Create
    let resp = client
        .post(format!(
            "{}/construction/submittals/v1/projects/{}/items",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"title": "Shop Drawing", "description": "Steel framing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let sub_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["status"], "waiting");

    // Get
    let resp = client
        .get(format!(
            "{}/construction/submittals/v1/projects/{}/items/{}",
            server.url, project, sub_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Update
    let resp = client
        .patch(format!(
            "{}/construction/submittals/v1/projects/{}/items/{}",
            server.url, project, sub_id
        ))
        .bearer_auth(&token)
        .json(&json!({"status": "approved"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "approved");

    // Delete
    let resp = client
        .delete(format!(
            "{}/construction/submittals/v1/projects/{}/items/{}",
            server.url, project, sub_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// ── ACC: Checklists ────────────────────────────────────────────────

#[tokio::test]
async fn test_checklist_crud_and_templates() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "test-project-004";

    // Create
    let resp = client
        .post(format!(
            "{}/construction/checklists/v1/projects/{}/checklists",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"title": "Fire Safety", "description": "Floor 3 inspection"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let chk_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["status"], "not_started");

    // Update
    let resp = client
        .patch(format!(
            "{}/construction/checklists/v1/projects/{}/checklists/{}",
            server.url, project, chk_id
        ))
        .bearer_auth(&token)
        .json(&json!({"status": "completed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // List templates (static — always returns 3)
    let resp = client
        .get(format!(
            "{}/construction/checklists/v1/projects/{}/templates",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let templates = body["results"].as_array().unwrap();
    assert_eq!(templates.len(), 3);
}

// ── Design Automation: Engines ─────────────────────────────────────

#[tokio::test]
async fn test_da_engines_list() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    let resp = client
        .get(format!("{}/da/us-east/v3/engines", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 4, "expected at least 4 engines, got {}", data.len());
}

// ── Design Automation: App Bundles ─────────────────────────────────

#[tokio::test]
async fn test_da_appbundle_lifecycle() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create
    let resp = client
        .post(format!("{}/da/us-east/v3/appbundles", server.url))
        .bearer_auth(&token)
        .json(&json!({
            "id": "TestBundle",
            "engine": "Autodesk.Revit+2025",
            "description": "Integration test bundle"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "TestBundle");
    assert!(body["uploadParameters"].is_object());

    // List
    let resp = client
        .get(format!("{}/da/us-east/v3/appbundles", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().any(|d| d.as_str() == Some("TestBundle")));

    // Delete
    let resp = client
        .delete(format!("{}/da/us-east/v3/appbundles/TestBundle", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// ── Design Automation: Activities ──────────────────────────────────

#[tokio::test]
async fn test_da_activity_lifecycle() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create
    let resp = client
        .post(format!("{}/da/us-east/v3/activities", server.url))
        .bearer_auth(&token)
        .json(&json!({
            "id": "TestActivity",
            "engine": "Autodesk.Revit+2025",
            "description": "Integration test activity"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "TestActivity");

    // List
    let resp = client
        .get(format!("{}/da/us-east/v3/activities", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().any(|d| d.as_str() == Some("TestActivity")));

    // Delete
    let resp = client
        .delete(format!(
            "{}/da/us-east/v3/activities/TestActivity",
            server.url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// ── Design Automation: Work Items ──────────────────────────────────

#[tokio::test]
async fn test_da_workitem_lifecycle() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create work item
    let resp = client
        .post(format!("{}/da/us-east/v3/workitems", server.url))
        .bearer_auth(&token)
        .json(&json!({"activityId": "owner.TestActivity+prod"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let workitem_id = body["id"].as_str().unwrap().to_string();
    assert!(workitem_id.starts_with("workitem-"));
    assert_eq!(body["status"], "pending");

    // Get status
    let resp = client
        .get(format!(
            "{}/da/us-east/v3/workitems/{}",
            server.url, workitem_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], workitem_id);

    // List work items
    let resp = client
        .get(format!("{}/da/us-east/v3/workitems", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().any(|w| w["id"] == workitem_id));
}

// ── Reality Capture: Photoscenes ───────────────────────────────────

#[tokio::test]
async fn test_reality_photoscene_lifecycle() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create photoscene
    let resp = client
        .post(format!("{}/photo-to-3d/v1/photoscene", server.url))
        .bearer_auth(&token)
        .json(&json!({
            "scenename": "Test Scene",
            "scenetype": "object",
            "format": "rcm"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let scene_id = body["Photoscene"]["photosceneid"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(scene_id.starts_with("ps-"));
    assert_eq!(body["Photoscene"]["status"], "Created");

    // List photoscenes
    let resp = client
        .get(format!("{}/photo-to-3d/v1/photoscene", server.url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let scenes = body["Photoscenes"]["photoscene"].as_array().unwrap();
    assert!(scenes
        .iter()
        .any(|s| s["photosceneid"] == scene_id));

    // Process photoscene
    let resp = client
        .post(format!(
            "{}/photo-to-3d/v1/photoscene/{}",
            server.url, scene_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Check progress
    let resp = client
        .get(format!(
            "{}/photo-to-3d/v1/photoscene/{}/progress",
            server.url, scene_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["Photoscene"]["status"], "Done");

    // Get result (GET on the photoscene itself — returns scenelink after processing)
    let resp = client
        .get(format!(
            "{}/photo-to-3d/v1/photoscene/{}",
            server.url, scene_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["Photoscene"]["scenelink"].is_string());

    // Delete photoscene (Reality Capture API returns 200, not 204)
    let resp = client
        .delete(format!(
            "{}/photo-to-3d/v1/photoscene/{}",
            server.url, scene_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ── ACC: Pre-seeded demo data ──────────────────────────────────────

#[tokio::test]
async fn test_preseeded_demo_data() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "mock-project-001";

    // RFIs (3 pre-seeded)
    let resp = client
        .get(format!(
            "{}/construction/rfis/v2/projects/{}/rfis",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rfis = body["results"].as_array().unwrap();
    assert!(rfis.len() >= 3, "expected >=3 pre-seeded RFIs, got {}", rfis.len());

    // Assets (3 pre-seeded)
    let resp = client
        .get(format!(
            "{}/construction/assets/v1/projects/{}/assets",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let assets = body["results"].as_array().unwrap();
    assert!(assets.len() >= 3, "expected >=3 pre-seeded assets, got {}", assets.len());
}

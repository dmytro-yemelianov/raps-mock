// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Integration tests for Data Management Folders and Items state managers.

use reqwest::Client;
use serde_json::{json, Value};

async fn setup() -> (raps_mock::TestServer, Client, String) {
    let server = raps_mock::TestServer::start_default()
        .await
        .expect("failed to start test server");
    let client = Client::new();
    let token = acquire_token(&client, &server.url).await;
    (server, client, token)
}

async fn acquire_token(client: &Client, base: &str) -> String {
    let resp = client
        .post(format!("{}/authentication/v2/token", base))
        .json(&json!({
            "grant_type": "client_credentials",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

const PROJECT: &str = "b.default-project";

// ---- Top Folders ----

#[tokio::test]
async fn test_list_top_folders_returns_seeded_data() {
    let (server, client, token) = setup().await;

    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/topFolders",
            server.url, PROJECT
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2, "expected at least 2 top folders, got {}", data.len());

    let names: Vec<&str> = data
        .iter()
        .map(|f| f["attributes"]["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Project Files"));
    assert!(names.contains(&"Plans"));
}

// ---- Folder CRUD ----

#[tokio::test]
async fn test_folder_create_get_update_delete() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    // Create a folder
    let resp = client
        .post(format!("{}/data/v1/projects/{}/folders", base, PROJECT))
        .bearer_auth(&token)
        .json(&json!({
            "data": {
                "type": "folders",
                "attributes": { "name": "Test Folder" },
                "relationships": {
                    "parent": {
                        "data": {
                            "type": "folders",
                            "id": "urn:adsk.wipprod:fs.folder:co.mock-top-folder-001"
                        }
                    }
                }
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let folder_id = body["data"]["id"].as_str().unwrap().to_string();
    assert!(folder_id.starts_with("urn:adsk.wipprod:fs.folder:co."));
    assert_eq!(body["data"]["attributes"]["name"], "Test Folder");
    assert_eq!(body["data"]["attributes"]["displayName"], "Test Folder");

    // Get the folder
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}",
            base, PROJECT, folder_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["attributes"]["name"], "Test Folder");

    // Update the folder
    let resp = client
        .patch(format!(
            "{}/data/v1/projects/{}/folders/{}",
            base, PROJECT, folder_id
        ))
        .bearer_auth(&token)
        .json(&json!({
            "data": {
                "type": "folders",
                "attributes": { "name": "Renamed Folder" }
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["attributes"]["name"], "Renamed Folder");
    assert_eq!(body["data"]["attributes"]["displayName"], "Renamed Folder");

    // Delete the folder
    let resp = client
        .delete(format!(
            "{}/data/v1/projects/{}/folders/{}",
            base, PROJECT, folder_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify it's gone
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}",
            base, PROJECT, folder_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ---- Folder Contents ----

#[tokio::test]
async fn test_list_folder_contents_includes_subfolders_and_items() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    // The seeded "Project Files" folder has a subfolder and an item
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}/contents",
            base, PROJECT, "mock-top-folder-001"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();

    let types: Vec<&str> = data.iter().map(|d| d["type"].as_str().unwrap()).collect();
    assert!(types.contains(&"folders"), "expected a subfolder in contents");
    assert!(types.contains(&"items"), "expected an item in contents");
}

// ---- Item CRUD ----

#[tokio::test]
async fn test_item_create_get_update_delete() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    // Create an item in "Project Files" folder
    let resp = client
        .post(format!("{}/data/v1/projects/{}/items", base, PROJECT))
        .bearer_auth(&token)
        .json(&json!({
            "data": {
                "type": "items",
                "attributes": { "displayName": "Model.rvt" },
                "relationships": {
                    "parent": {
                        "data": {
                            "type": "folders",
                            "id": "urn:adsk.wipprod:fs.folder:co.mock-top-folder-001"
                        }
                    }
                }
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let item_id = body["data"]["id"].as_str().unwrap().to_string();
    assert!(item_id.starts_with("urn:adsk.wipprod:dm.lineage:"));
    assert_eq!(body["data"]["attributes"]["displayName"], "Model.rvt");

    // Get the item
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/items/{}",
            base, PROJECT, item_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["attributes"]["displayName"], "Model.rvt");

    // Update the item
    let resp = client
        .patch(format!(
            "{}/data/v1/projects/{}/items/{}",
            base, PROJECT, item_id
        ))
        .bearer_auth(&token)
        .json(&json!({
            "data": {
                "type": "items",
                "attributes": { "displayName": "Model-v2.rvt" }
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["attributes"]["displayName"], "Model-v2.rvt");

    // Delete the item
    let resp = client
        .delete(format!(
            "{}/data/v1/projects/{}/items/{}",
            base, PROJECT, item_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify it's gone
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/items/{}",
            base, PROJECT, item_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ---- Item Versions ----

#[tokio::test]
async fn test_item_versions() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    // Create an item
    let resp = client
        .post(format!("{}/data/v1/projects/{}/items", base, PROJECT))
        .bearer_auth(&token)
        .json(&json!({
            "data": {
                "type": "items",
                "attributes": { "displayName": "Sheet.pdf" },
                "relationships": {
                    "parent": {
                        "data": {
                            "type": "folders",
                            "id": "urn:adsk.wipprod:fs.folder:co.mock-top-folder-001"
                        }
                    }
                }
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let item_id = body["data"]["id"].as_str().unwrap().to_string();

    // List versions — should have version 1 auto-created
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/items/{}/versions",
            base, PROJECT, item_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let versions = body["data"].as_array().unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0]["attributes"]["versionNumber"], 1);
    assert_eq!(versions[0]["attributes"]["displayName"], "Sheet.pdf");
    assert!(versions[0]["id"]
        .as_str()
        .unwrap()
        .starts_with("urn:adsk.wipprod:fs.file:vf."));
}

// ---- Seeded Item Versions ----

#[tokio::test]
async fn test_seeded_item_has_version() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    // The seeded item "mock-item-001" should have a version
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/items/{}/versions",
            base, PROJECT, "urn:adsk.wipprod:dm.lineage:mock-item-001"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let versions = body["data"].as_array().unwrap();
    assert!(!versions.is_empty(), "seeded item should have at least 1 version");
    assert_eq!(versions[0]["attributes"]["displayName"], "Drawing.dwg");
    assert_eq!(versions[0]["attributes"]["storageSize"], 1024000);
}

// ---- Folder permissions (remain static/stateless) ----

#[tokio::test]
async fn test_folder_permissions() {
    let (server, client, token) = setup().await;
    let base = &server.url;

    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}/permissions",
            base, PROJECT, "mock-top-folder-001"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(data[0]["type"], "folder-permissions");
}

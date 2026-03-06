// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov
//
// Integration tests for Admin (users, projects, project users, jobs, companies,
// templates), DA aliases, folder permissions, and issue attachments.

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

// ── Admin Users ──────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_user_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let account = "mock-account-001";

    // List seeded users
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/accounts/{}/users",
            server.url, account
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["pagination"]["totalResults"].as_i64().unwrap() >= 2);

    // Add user
    let resp = client
        .post(format!(
            "{}/construction/admin/v1/accounts/{}/users",
            server.url, account
        ))
        .bearer_auth(&token)
        .json(&json!({"email": "carol@example.com", "name": "Carol Davis", "role": "project_user"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let user: Value = resp.json().await.unwrap();
    let user_id = user["id"].as_str().unwrap().to_string();
    assert_eq!(user["email"], "carol@example.com");

    // Search user
    let resp = client
        .post(format!(
            "{}/construction/admin/v1/accounts/{}/users/search",
            server.url, account
        ))
        .bearer_auth(&token)
        .json(&json!({"email": "carol@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let found: Value = resp.json().await.unwrap();
    assert_eq!(found["email"], "carol@example.com");

    // Update user
    let resp = client
        .patch(format!(
            "{}/construction/admin/v1/accounts/{}/users/{}",
            server.url, account, user_id
        ))
        .bearer_auth(&token)
        .json(&json!({"name": "Carol Davis-Updated", "role": "project_admin"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["role"], "project_admin");

    // Delete user
    let resp = client
        .delete(format!(
            "{}/construction/admin/v1/accounts/{}/users/{}",
            server.url, account, user_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn test_admin_import_users() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let account = "mock-account-001";

    let resp = client
        .post(format!(
            "{}/construction/admin/v1/accounts/{}/users/import",
            server.url, account
        ))
        .bearer_auth(&token)
        .json(&json!({
            "users": [
                {"email": "imp1@example.com", "name": "Import One", "role": "project_user"},
                {"email": "imp2@example.com", "name": "Import Two", "role": "project_user"}
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["success"].as_i64().unwrap(), 2);
    assert_eq!(body["failure"].as_i64().unwrap(), 0);
}

// ── Admin Projects ──────────────────────────────────────────────

#[tokio::test]
async fn test_admin_project_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let account = "mock-account-001";

    // List seeded projects
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/accounts/{}/projects",
            server.url, account
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["pagination"]["totalResults"].as_i64().unwrap() >= 2);

    // Create project
    let resp = client
        .post(format!(
            "{}/construction/admin/v1/accounts/{}/projects",
            server.url, account
        ))
        .bearer_auth(&token)
        .json(&json!({"name": "Test Project Gamma"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let proj: Value = resp.json().await.unwrap();
    let proj_id = proj["id"].as_str().unwrap().to_string();
    assert_eq!(proj["name"], "Test Project Gamma");

    // Get project
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/accounts/{}/projects/{}",
            server.url, account, proj_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let got: Value = resp.json().await.unwrap();
    assert_eq!(got["name"], "Test Project Gamma");

    // Update project
    let resp = client
        .patch(format!(
            "{}/construction/admin/v1/accounts/{}/projects/{}",
            server.url, account, proj_id
        ))
        .bearer_auth(&token)
        .json(&json!({"name": "Renamed Gamma", "status": "inactive"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Gamma");
    assert_eq!(updated["status"], "inactive");
}

// ── Project Users ───────────────────────────────────────────────

#[tokio::test]
async fn test_project_user_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "proj-001";

    // List seeded project users
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/projects/{}/users",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["pagination"]["totalResults"].as_i64().unwrap() >= 1);

    // Add project user
    let resp = client
        .post(format!(
            "{}/construction/admin/v1/projects/{}/users",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"userId": "user-new-001", "email": "new@example.com", "name": "New Guy", "roleIds": ["role-viewer"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let pu: Value = resp.json().await.unwrap();
    assert_eq!(pu["id"], "user-new-001");
    assert_eq!(pu["roleIds"][0], "role-viewer");

    // Get project user
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/projects/{}/users/user-new-001",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Update project user role
    let resp = client
        .patch(format!(
            "{}/construction/admin/v1/projects/{}/users/user-new-001",
            server.url, project
        ))
        .bearer_auth(&token)
        .json(&json!({"roleIds": ["role-editor"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["roleIds"][0], "role-editor");

    // Delete project user
    let resp = client
        .delete(format!(
            "{}/construction/admin/v1/projects/{}/users/user-new-001",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify deleted — should 404
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/projects/{}/users/user-new-001",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ── HQ Companies ────────────────────────────────────────────────

#[tokio::test]
async fn test_hq_companies_seeded() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    let resp = client
        .get(format!(
            "{}/hq/v1/accounts/mock-account-001/companies",
            server.url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert!(arr.len() >= 2);
    assert_eq!(arr[0]["name"], "Mock Construction Co");
}

// ── DA Aliases ──────────────────────────────────────────────────

#[tokio::test]
async fn test_da_appbundle_alias() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create an app bundle first
    let resp = client
        .post(format!("{}/da/us-east/v3/appbundles", server.url))
        .bearer_auth(&token)
        .json(&json!({"id": "MyBundle", "engine": "Autodesk.Revit+2025", "description": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create alias
    let resp = client
        .post(format!(
            "{}/da/us-east/v3/appbundles/MyBundle/aliases",
            server.url
        ))
        .bearer_auth(&token)
        .json(&json!({"id": "prod", "version": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let alias: Value = resp.json().await.unwrap();
    assert_eq!(alias["id"], "prod");
    assert_eq!(alias["version"], 1);
    assert_eq!(alias["receiver"], "MyBundle");
}

#[tokio::test]
async fn test_da_activity_alias() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;

    // Create an activity first
    let resp = client
        .post(format!("{}/da/us-east/v3/activities", server.url))
        .bearer_auth(&token)
        .json(&json!({"id": "MyActivity", "engine": "Autodesk.AutoCAD+24"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Create alias
    let resp = client
        .post(format!(
            "{}/da/us-east/v3/activities/MyActivity/aliases",
            server.url
        ))
        .bearer_auth(&token)
        .json(&json!({"id": "staging", "version": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let alias: Value = resp.json().await.unwrap();
    assert_eq!(alias["id"], "staging");
    assert_eq!(alias["receiver"], "MyActivity");
}

// ── Folder Permissions ──────────────────────────────────────────

#[tokio::test]
async fn test_folder_permissions_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "b.default-project";
    let folder = "mock-top-folder-001";

    // Get seeded permissions
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}/permissions",
            server.url, project, folder
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(!body["data"].as_array().unwrap().is_empty());
    assert_eq!(body["data"][0]["attributes"]["subjectId"], "user-001");

    // Batch update permissions (POST, not PATCH — matches route registration)
    let resp = client
        .post(format!(
            "{}/data/v1/projects/{}/folders/{}/permissions:batch-update",
            server.url, project, folder
        ))
        .bearer_auth(&token)
        .json(&json!({
            "data": [
                {
                    "type": "folder-permissions",
                    "attributes": {
                        "subjectId": "user-099",
                        "subjectType": "user",
                        "actions": ["view", "download"]
                    }
                }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["attributes"]["subjectId"], "user-099");

    // Verify new permission is persisted
    let resp = client
        .get(format!(
            "{}/data/v1/projects/{}/folders/{}/permissions",
            server.url, project, folder
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let perms = body["data"].as_array().unwrap();
    assert!(perms.len() >= 2);
}

// ── Issue Attachments ───────────────────────────────────────────

#[tokio::test]
async fn test_issue_attachments_empty_initially() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let project = "mock-project-001";

    let resp = client
        .get(format!(
            "{}/construction/issues/v1/projects/{}/issues/demo-issue-001/attachments",
            server.url, project
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["pagination"]["totalResults"].as_i64().unwrap(), 0);
}

// ── Project Templates ───────────────────────────────────────────

#[tokio::test]
async fn test_project_template_crud() {
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();
    let token = get_token(&client, &server.url).await;
    let account = "mock-account-001";

    // List seeded templates
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/accounts/{}/project_templates",
            server.url, account
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["pagination"]["totalResults"].as_i64().unwrap() >= 2);

    // Create template
    let resp = client
        .post(format!(
            "{}/construction/admin/v1/accounts/{}/project_templates",
            server.url, account
        ))
        .bearer_auth(&token)
        .json(&json!({"name": "Custom Template"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let tmpl: Value = resp.json().await.unwrap();
    let tmpl_id = tmpl["id"].as_str().unwrap().to_string();
    assert_eq!(tmpl["name"], "Custom Template");

    // Get template
    let resp = client
        .get(format!(
            "{}/construction/admin/v1/accounts/{}/project_templates/{}",
            server.url, account, tmpl_id
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Update template
    let resp = client
        .patch(format!(
            "{}/construction/admin/v1/accounts/{}/project_templates/{}",
            server.url, account, tmpl_id
        ))
        .bearer_auth(&token)
        .json(&json!({"name": "Renamed Template", "status": "archived"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Template");
    assert_eq!(updated["status"], "archived");
}

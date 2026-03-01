// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Extracted route handlers for hardcoded endpoints.
//!
//! Each handler is an async function that takes an `Option<StateManager>` plus
//! any request-specific parameters, and returns an `impl IntoResponse`.

use axum::response::{IntoResponse, Json as JsonResponse};
use base64::Engine as _; // needed for .decode() method on engine instances
use serde_json::{Value, json};

use crate::state::StateManager;

/// Unwrap a `crate::error::Result`, returning HTTP 500 on error.
macro_rules! try_state {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({"error": format!("Internal error: {}", e)})),
                )
                    .into_response();
            }
        }
    };
}

/// Decode a base64-encoded URN, supporting both standard and URL-safe alphabets.
/// Returns the original string if decoding fails (already decoded or not base64).
fn decode_base64_urn(urn: &str) -> String {
    // Try URL-safe base64 first (APS uses this), then standard
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(urn)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(urn))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(urn))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| urn.to_string())
}

// ---- Auth ----

pub async fn handle_auth_token(state: Option<StateManager>, body: Value) -> impl IntoResponse {
    let grant_type = body
        .get("grant_type")
        .and_then(|v| v.as_str())
        .unwrap_or("client_credentials");

    if let Some(ref state_manager) = state {
        let client_id = body
            .get("client_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default-client");

        let scope = body
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let token = try_state!(state_manager.auth.generate_token(client_id, 3600, scope));

        let mut response = json!({
            "access_token": token.access_token,
            "token_type": token.token_type,
            "expires_in": token.expires_in
        });

        // Include refresh_token for 3-legged flows
        if (grant_type == "authorization_code" || grant_type == "refresh_token") && let Some(ref rt) = token.refresh_token {
            response["refresh_token"] = json!(rt);
        }

        (axum::http::StatusCode::OK, JsonResponse(response)).into_response()
    } else {
        let mut response = json!({
            "access_token": "mock-token",
            "token_type": "Bearer",
            "expires_in": 3600
        });

        // Include refresh_token for 3-legged flows
        if grant_type == "authorization_code" || grant_type == "refresh_token" {
            response["refresh_token"] = json!("mock-refresh-token-xxx");
        }

        (axum::http::StatusCode::OK, JsonResponse(response)).into_response()
    }
}

pub async fn handle_userinfo() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "sub": "mock-user-001",
            "name": "Mock User",
            "given_name": "Mock",
            "family_name": "User",
            "email": "mock@example.com",
            "email_verified": true,
            "picture": ""
        })),
    )
        .into_response()
}

// ---- OSS Buckets ----

pub async fn handle_list_buckets(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let buckets = try_state!(state_manager.buckets.list_buckets());
        let items: Vec<Value> = buckets
            .into_iter()
            .map(|b| {
                json!({
                    "bucketKey": b.bucket_key,
                    "bucketOwner": b.bucket_owner,
                    "createdDate": b.created_date,
                    "policyKey": b.policy_key,
                    "permissions": b.permissions
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": items, "next": null })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": [], "next": null })),
        )
            .into_response()
    }
}

pub async fn handle_create_bucket(state: Option<StateManager>, body: Value) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let bucket_key = body
            .get("bucketKey")
            .and_then(|v| v.as_str())
            .unwrap_or("default-bucket");

        let policy_key = body
            .get("policyKey")
            .and_then(|v| v.as_str())
            .unwrap_or("transient");

        let bucket = try_state!(state_manager
            .buckets
            .create_bucket(bucket_key.to_string(), policy_key.to_string()));

        (axum::http::StatusCode::OK, JsonResponse(json!(bucket))).into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "bucketKey": "mock-bucket",
                "createdDate": chrono::Utc::now().timestamp_millis(),
                "policyKey": "transient"
            })),
        )
            .into_response()
    }
}

// ---- OSS Objects ----

pub async fn handle_list_objects(
    state: Option<StateManager>,
    bucket_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let objects = try_state!(state_manager.objects.list_objects(&bucket_key));
        let items: Vec<Value> = objects
            .into_iter()
            .map(|o| {
                json!({
                    "bucketKey": o.bucket_key,
                    "objectKey": o.object_key,
                    "objectId": o.object_id,
                    "sha1": o.sha1,
                    "size": o.size,
                    "contentType": o.content_type,
                    "location": o.location
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": items, "next": null })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": [], "next": null })),
        )
            .into_response()
    }
}

// ---- Data Management ----

pub async fn handle_list_hubs(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let hubs = try_state!(state_manager.projects.list_hubs());
        let data: Vec<Value> = hubs
            .into_iter()
            .map(|h| {
                json!({
                    "type": "hubs",
                    "id": h.id,
                    "attributes": {
                        "name": h.name,
                        "region": h.region
                    }
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": data
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": []
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_hub(state: Option<StateManager>, hub_id: String) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(hub) = try_state!(state_manager.projects.get_hub(&hub_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "jsonapi": { "version": "1.0" },
                    "data": {
                        "type": "hubs",
                        "id": hub.id,
                        "attributes": {
                            "name": hub.name,
                            "region": hub.region
                        }
                    }
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "jsonapi": { "version": "1.0" },
                    "errors": [{
                        "status": "404",
                        "title": "Not Found",
                        "detail": format!("Hub {} not found", hub_id)
                    }]
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "errors": [{
                    "status": "404",
                    "title": "Not Found"
                }]
            })),
        )
            .into_response()
    }
}

pub async fn handle_list_hub_projects(
    state: Option<StateManager>,
    hub_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let projects = try_state!(state_manager.projects.list_projects(&hub_id));
        let data: Vec<Value> = projects
            .into_iter()
            .map(|p| {
                json!({
                    "type": "projects",
                    "id": p.id,
                    "attributes": {
                        "name": p.name
                    }
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": data
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": []
            })),
        )
            .into_response()
    }
}

// ---- Model Derivative ----

pub async fn handle_create_translation(
    state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let input_urn = body
            .get("input")
            .and_then(|i| i.get("urn"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let output_type = body
            .get("output")
            .and_then(|o| o.get("formats"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("svf2");

        // Decode the base64 URN before storing so lookups match
        let decoded_urn = decode_base64_urn(input_urn);
        let job = try_state!(state_manager.translations.create_job(decoded_urn));

        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "result": "success",
                "urn": job.urn,
                "acceptedJobs": {
                    "output": {
                        "formats": [{ "type": output_type }]
                    }
                }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "result": "success" })),
        )
            .into_response()
    }
}

pub async fn handle_get_manifest(state: Option<StateManager>, urn: String) -> impl IntoResponse {
    let decoded_urn = decode_base64_urn(&urn);

    if let Some(ref state_manager) = state {
        // Simulate progress on each poll (like real APS: repeated GET advances the job)
        try_state!(state_manager.translations.simulate_progress(&decoded_urn));

        if let Some(job) = try_state!(state_manager.translations.get_job(&decoded_urn)) {
            let status_str = match job.status {
                crate::state::translations::TranslationStatus::Pending => "pending",
                crate::state::translations::TranslationStatus::InProgress => "inprogress",
                crate::state::translations::TranslationStatus::Success => "success",
                crate::state::translations::TranslationStatus::Failed => "failed",
            };

            let manifest = json!({
                "type": "manifest",
                "hasThumbnail": if status_str == "success" { "true" } else { "false" },
                "status": status_str,
                "progress": job.progress,
                "region": "US",
                "urn": decoded_urn,
                "version": "1.0",
                "derivatives": if status_str == "success" {
                    vec![json!({
                        "status": "success",
                        "progress": "complete",
                        "outputType": "svf2",
                        "children": []
                    })]
                } else {
                    vec![]
                }
            });

            (axum::http::StatusCode::OK, JsonResponse(manifest)).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Translation job for URN {} not found", decoded_urn)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "type": "manifest",
                "hasThumbnail": "false",
                "status": "pending",
                "progress": "0%",
                "region": "US",
                "urn": decoded_urn,
                "version": "1.0",
                "derivatives": []
            })),
        )
            .into_response()
    }
}

// ---- ACC Issues ----

pub async fn handle_list_issues(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let issues = try_state!(state_manager.issues.list_issues(&project_id));
        let total = issues.len() as i32;
        let results: Vec<Value> = issues
            .into_iter()
            .map(|i| {
                json!({
                    "id": i.id,
                    "title": i.title,
                    "description": i.description,
                    "status": i.status,
                    "createdAt": i.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": {
                    "limit": 100,
                    "offset": 0,
                    "totalResults": total
                }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": {
                    "limit": 100,
                    "offset": 0,
                    "totalResults": 0
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_issue(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Issue")
            .to_string();

        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let issue = try_state!(state_manager
            .issues
            .create_issue(project_id, title, description));

        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": issue.id,
                "title": issue.title,
                "description": issue.description,
                "status": issue.status,
                "createdAt": issue.created_at
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-issue-id",
                "title": "Mock Issue",
                "status": "open"
            })),
        )
            .into_response()
    }
}

// ---- ACC Issues (additional endpoints) ----

pub async fn handle_get_issue_types(
    _state: Option<StateManager>,
    _project_id: String,
) -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [
                { "id": "issue-type-001", "title": "Design", "isActive": true },
                { "id": "issue-type-002", "title": "Clash", "isActive": true },
                { "id": "issue-type-003", "title": "Safety", "isActive": true },
                { "id": "issue-type-004", "title": "Commissioning", "isActive": true }
            ],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 4 }
        })),
    )
        .into_response()
}

pub async fn handle_get_issue(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(issue) = try_state!(state_manager.issues.get_issue(&project_id, &issue_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": issue.id,
                    "title": issue.title,
                    "description": issue.description,
                    "status": issue.status,
                    "createdAt": issue.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Issue {} not found", issue_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Issue not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_update_issue(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(issue) =
            try_state!(state_manager
                .issues
                .update_issue(&project_id, &issue_id, title, description, status))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": issue.id,
                    "title": issue.title,
                    "description": issue.description,
                    "status": issue.status,
                    "createdAt": issue.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Issue {} not found", issue_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Issue not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_issue(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager.issues.delete_issue(&project_id, &issue_id)) {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Issue {} not found", issue_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

pub async fn handle_list_issue_comments(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let comments = try_state!(state_manager.issues.list_comments(&project_id, &issue_id));
        let total = comments.len() as i32;
        let results: Vec<Value> = comments
            .into_iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "issueId": c.issue_id,
                    "body": c.body,
                    "createdAt": c.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": {
                    "limit": 50,
                    "offset": 0,
                    "totalResults": total
                }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_issue_comment(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let comment_body = body
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(comment) =
            try_state!(state_manager
                .issues
                .add_comment(&project_id, &issue_id, comment_body))
        {
            (
                axum::http::StatusCode::CREATED,
                JsonResponse(json!({
                    "id": comment.id,
                    "issueId": comment.issue_id,
                    "body": comment.body,
                    "createdAt": comment.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Issue {} not found", issue_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-comment-id",
                "issueId": issue_id,
                "body": "Mock comment"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_issue_comment(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
    comment_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager
            .issues
            .delete_comment(&project_id, &issue_id, &comment_id))
        {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Comment {} not found", comment_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

pub async fn handle_list_issue_attachments(
    state: Option<StateManager>,
    project_id: String,
    issue_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let attachments = try_state!(sm.issues.list_attachments(&project_id, &issue_id));
        let total = attachments.len() as i32;
        let results: Vec<Value> = attachments
            .into_iter()
            .map(|a| json!({ "id": a.id, "name": a.name, "url": a.url, "createdAt": a.created_at }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
        })),
    )
        .into_response()
}

// ---- ACC RFIs ----

pub async fn handle_list_rfis(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let rfis = try_state!(state_manager.acc.list_rfis(&project_id));
        let total = rfis.len() as i32;
        let results: Vec<Value> = rfis
            .into_iter()
            .map(|r| {
                json!({
                    "id": r.id,
                    "title": r.title,
                    "description": r.description,
                    "status": r.status,
                    "createdAt": r.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_rfi(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled RFI")
            .to_string();
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let rfi = try_state!(state_manager.acc.create_rfi(project_id, title, description));
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": rfi.id,
                "title": rfi.title,
                "description": rfi.description,
                "status": rfi.status,
                "createdAt": rfi.created_at
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-rfi-id",
                "title": "Mock RFI",
                "status": "open"
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_rfi(
    state: Option<StateManager>,
    project_id: String,
    rfi_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(rfi) = try_state!(state_manager.acc.get_rfi(&project_id, &rfi_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": rfi.id,
                    "title": rfi.title,
                    "description": rfi.description,
                    "status": rfi.status,
                    "createdAt": rfi.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("RFI {} not found", rfi_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "RFI not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_update_rfi(
    state: Option<StateManager>,
    project_id: String,
    rfi_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(rfi) =
            try_state!(state_manager
                .acc
                .update_rfi(&project_id, &rfi_id, title, description, status))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": rfi.id,
                    "title": rfi.title,
                    "description": rfi.description,
                    "status": rfi.status,
                    "createdAt": rfi.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("RFI {} not found", rfi_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "RFI not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_rfi(
    state: Option<StateManager>,
    project_id: String,
    rfi_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager.acc.delete_rfi(&project_id, &rfi_id)) {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("RFI {} not found", rfi_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

// ---- ACC Assets ----

pub async fn handle_list_assets(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let assets = try_state!(state_manager.acc.list_assets(&project_id));
        let total = assets.len() as i32;
        let results: Vec<Value> = assets
            .into_iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "title": a.title,
                    "description": a.description,
                    "status": a.status,
                    "createdAt": a.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_asset(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Asset")
            .to_string();
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let asset = try_state!(state_manager
            .acc
            .create_asset(project_id, title, description));
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": asset.id,
                "title": asset.title,
                "description": asset.description,
                "status": asset.status,
                "createdAt": asset.created_at
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-asset-id",
                "title": "Mock Asset",
                "status": "active"
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_asset(
    state: Option<StateManager>,
    project_id: String,
    asset_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(asset) = try_state!(state_manager.acc.get_asset(&project_id, &asset_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": asset.id,
                    "title": asset.title,
                    "description": asset.description,
                    "status": asset.status,
                    "createdAt": asset.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Asset {} not found", asset_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Asset not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_update_asset(
    state: Option<StateManager>,
    project_id: String,
    asset_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(asset) =
            try_state!(state_manager
                .acc
                .update_asset(&project_id, &asset_id, title, description, status))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": asset.id,
                    "title": asset.title,
                    "description": asset.description,
                    "status": asset.status,
                    "createdAt": asset.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Asset {} not found", asset_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Asset not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_asset(
    state: Option<StateManager>,
    project_id: String,
    asset_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager.acc.delete_asset(&project_id, &asset_id)) {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Asset {} not found", asset_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

// ---- ACC Submittals ----

pub async fn handle_list_submittals(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let submittals = try_state!(state_manager.acc.list_submittals(&project_id));
        let total = submittals.len() as i32;
        let results: Vec<Value> = submittals
            .into_iter()
            .map(|s| {
                json!({
                    "id": s.id,
                    "title": s.title,
                    "description": s.description,
                    "status": s.status,
                    "createdAt": s.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_submittal(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Submittal")
            .to_string();
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let submittal = try_state!(state_manager
            .acc
            .create_submittal(project_id, title, description));
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": submittal.id,
                "title": submittal.title,
                "description": submittal.description,
                "status": submittal.status,
                "createdAt": submittal.created_at
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-submittal-id",
                "title": "Mock Submittal",
                "status": "waiting"
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_submittal(
    state: Option<StateManager>,
    project_id: String,
    submittal_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(submittal) = try_state!(state_manager.acc.get_submittal(&project_id, &submittal_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": submittal.id,
                    "title": submittal.title,
                    "description": submittal.description,
                    "status": submittal.status,
                    "createdAt": submittal.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Submittal {} not found", submittal_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Submittal not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_update_submittal(
    state: Option<StateManager>,
    project_id: String,
    submittal_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(submittal) = try_state!(state_manager.acc.update_submittal(
            &project_id,
            &submittal_id,
            title,
            description,
            status,
        )) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": submittal.id,
                    "title": submittal.title,
                    "description": submittal.description,
                    "status": submittal.status,
                    "createdAt": submittal.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Submittal {} not found", submittal_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Submittal not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_submittal(
    state: Option<StateManager>,
    project_id: String,
    submittal_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager
            .acc
            .delete_submittal(&project_id, &submittal_id))
        {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Submittal {} not found", submittal_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

// ---- ACC Checklists ----

pub async fn handle_list_checklists(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let checklists = try_state!(state_manager.acc.list_checklists(&project_id));
        let total = checklists.len() as i32;
        let results: Vec<Value> = checklists
            .into_iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "title": c.title,
                    "description": c.description,
                    "status": c.status,
                    "createdAt": c.created_at
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_checklist(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Checklist")
            .to_string();
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let checklist = try_state!(state_manager
            .acc
            .create_checklist(project_id, title, description));
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": checklist.id,
                "title": checklist.title,
                "description": checklist.description,
                "status": checklist.status,
                "createdAt": checklist.created_at
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": "mock-checklist-id",
                "title": "Mock Checklist",
                "status": "not_started"
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_checklist(
    state: Option<StateManager>,
    project_id: String,
    checklist_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(checklist) = try_state!(state_manager.acc.get_checklist(&project_id, &checklist_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": checklist.id,
                    "title": checklist.title,
                    "description": checklist.description,
                    "status": checklist.status,
                    "createdAt": checklist.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Checklist {} not found", checklist_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Checklist not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_update_checklist(
    state: Option<StateManager>,
    project_id: String,
    checklist_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let title = body
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(checklist) = try_state!(state_manager.acc.update_checklist(
            &project_id,
            &checklist_id,
            title,
            description,
            status,
        )) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": checklist.id,
                    "title": checklist.title,
                    "description": checklist.description,
                    "status": checklist.status,
                    "createdAt": checklist.created_at
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Checklist {} not found", checklist_id)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "code": "NOT_FOUND",
                "message": "Checklist not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_list_checklist_templates(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let templates = state_manager.acc.list_templates(&project_id);
        let total = templates.len() as i32;
        let results: Vec<Value> = templates
            .into_iter()
            .map(|t| {
                json!({
                    "id": t.id,
                    "title": t.title,
                    "description": t.description
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": [],
                "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
            })),
        )
            .into_response()
    }
}

// ---- Webhooks ----

pub async fn handle_list_all_webhooks(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let subscriptions = try_state!(state_manager.webhooks.list_subscriptions());
        let data: Vec<Value> = subscriptions
            .into_iter()
            .map(|s| {
                json!({
                    "hookId": s.hook_id,
                    "tenant": s.tenant,
                    "callbackUrl": s.callback_url,
                    "event": s.event,
                    "system": s.system,
                    "createdDate": s.created_date,
                    "status": s.status,
                    "scope": s.scope
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": data,
                "links": { "next": null }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": [],
                "links": { "next": null }
            })),
        )
            .into_response()
    }
}

pub async fn handle_list_webhooks(
    state: Option<StateManager>,
    system: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let subscriptions = try_state!(state_manager.webhooks.list_subscriptions());
        let data: Vec<Value> = subscriptions
            .into_iter()
            .filter(|s| s.system == system)
            .map(|s| {
                json!({
                    "hookId": s.hook_id,
                    "tenant": s.tenant,
                    "callbackUrl": s.callback_url,
                    "event": s.event,
                    "system": s.system,
                    "createdDate": s.created_date,
                    "status": s.status,
                    "scope": s.scope
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": data,
                "links": { "next": null }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": [],
                "links": { "next": null }
            })),
        )
            .into_response()
    }
}

pub async fn handle_create_webhook(
    state: Option<StateManager>,
    system: String,
    event: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let callback_url = body
            .get("callbackUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("https://example.com/webhook")
            .to_string();

        let scope = crate::state::webhooks::WebhookScope {
            folder: body
                .get("scope")
                .and_then(|s| s.get("folder"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            workflow: body
                .get("scope")
                .and_then(|s| s.get("workflow"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        let subscription = try_state!(state_manager.webhooks.create_subscription(
            system.clone(),
            callback_url,
            event.clone(),
            system,
            scope,
        ));

        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "hookId": subscription.hook_id,
                "tenant": subscription.tenant,
                "callbackUrl": subscription.callback_url,
                "event": subscription.event,
                "system": subscription.system,
                "createdDate": subscription.created_date,
                "status": subscription.status,
                "scope": subscription.scope
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "hookId": "mock-hook-id",
                "event": event,
                "system": system,
                "status": "active"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_webhook(
    state: Option<StateManager>,
    hook_id: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager.webhooks.delete_subscription(&hook_id)) {
            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Webhook {} not found", hook_id)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({}))).into_response()
    }
}

// ---- OSS Bucket Details / Delete ----

pub async fn handle_get_bucket(
    state: Option<StateManager>,
    bucket_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(bucket) = try_state!(state_manager.buckets.get_bucket(&bucket_key)) {
            (axum::http::StatusCode::OK, JsonResponse(json!(bucket))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Bucket {} does not exist", bucket_key)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "reason": "Bucket not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_bucket(
    state: Option<StateManager>,
    bucket_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager.buckets.delete_bucket(&bucket_key)) {
            (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Bucket {} does not exist", bucket_key)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
    }
}

// ---- OSS Object Details / Delete ----

pub async fn handle_get_object_details(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(obj) = try_state!(state_manager.objects.get_object(&bucket_key, &object_key)) {
            let now = chrono::Utc::now().to_rfc3339();
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "bucketKey": obj.bucket_key,
                    "objectKey": obj.object_key,
                    "objectId": obj.object_id,
                    "sha1": obj.sha1,
                    "size": obj.size,
                    "contentType": obj.content_type,
                    "location": obj.location,
                    "createdDate": now,
                    "lastModifiedDate": now
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Object {}/{} does not exist", bucket_key, object_key)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "reason": "Object not found"
            })),
        )
            .into_response()
    }
}

pub async fn handle_delete_object(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if try_state!(state_manager
            .objects
            .delete_object(&bucket_key, &object_key))
        {
            (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Object {}/{} does not exist", bucket_key, object_key)
                })),
            )
                .into_response()
        }
    } else {
        (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
    }
}

// ---- OSS Signed S3 Upload / Download ----

pub async fn handle_signed_s3_upload_get(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
    host: String,
) -> impl IntoResponse {
    // Returns a signed upload URL pointing back to this mock server.
    let upload_key = format!("mock-upload-key-{}", uuid::Uuid::new_v4());
    let _state = state;
    let base = format!("http://{}", host);
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "uploadKey": upload_key,
            "urls": [
                format!("{}/oss/v2/buckets/{}/objects/{}/signeds3upload/mock-s3", base, bucket_key, object_key)
            ]
        })),
    )
        .into_response()
}

pub async fn handle_signed_s3_upload_complete(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
    body: Value,
) -> impl IntoResponse {
    // Finalize the upload: return existing object (stored by the PUT) or create one
    let _upload_key = body.get("uploadKey").and_then(|v| v.as_str()).unwrap_or("");

    if let Some(ref state_manager) = state {
        // Return existing object if the mock-s3 PUT already stored it
        let obj = match try_state!(state_manager.objects.get_object(&bucket_key, &object_key)) {
            Some(existing) => existing,
            None => try_state!(state_manager
                .objects
                .upload_object(bucket_key, object_key, 0, None)),
        };
        (axum::http::StatusCode::OK, JsonResponse(json!(obj))).into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "bucketKey": bucket_key,
                "objectKey": object_key,
                "objectId": format!("urn:adsk.objects:os.object:{}/{}", bucket_key, object_key),
                "size": 0,
                "contentType": "application/octet-stream"
            })),
        )
            .into_response()
    }
}

pub async fn handle_signed_s3_upload_put(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Mock S3 PUT endpoint — receives the actual file bytes.
    // Store the object with the real size.
    let size = body.len() as u64;
    if let Some(ref state_manager) = state {
        try_state!(state_manager
            .objects
            .upload_object(bucket_key, object_key, size, None));
    }
    (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
}

pub async fn handle_signed_s3_download(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
    host: String,
) -> impl IntoResponse {
    let base = format!("http://{}", host);
    if let Some(ref state_manager) = state {
        if let Some(obj) = try_state!(state_manager.objects.get_object(&bucket_key, &object_key)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "url": format!("{}/oss/v2/buckets/{}/objects/{}/signeds3download/mock-s3", base, bucket_key, object_key),
                    "size": obj.size,
                    "sha1": obj.sha1,
                    "status": "complete"
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "reason": format!("Object {}/{} does not exist", bucket_key, object_key)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "url": format!("{}/oss/v2/mock-download", base),
                "status": "complete"
            })),
        )
            .into_response()
    }
}

// ---- Design Automation ----

pub async fn handle_da_list_engines(state: Option<StateManager>) -> impl IntoResponse {
    let data = if let Some(ref sm) = state {
        sm.da
            .list_engines()
            .into_iter()
            .map(serde_json::Value::String)
            .collect::<Vec<_>>()
    } else {
        vec![json!("Autodesk.Revit+2025"), json!("Autodesk.AutoCAD+24")]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({ "data": data, "paginationToken": null })),
    )
        .into_response()
}

pub async fn handle_da_list_appbundles(state: Option<StateManager>) -> impl IntoResponse {
    let data = if let Some(ref sm) = state {
        try_state!(sm.da
            .list_app_bundles())
            .into_iter()
            .map(serde_json::Value::String)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({ "data": data, "paginationToken": null })),
    )
        .into_response()
}

pub async fn handle_da_create_appbundle(
    state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("bundle")
        .to_string();
    let engine = body
        .get("engine")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let desc = body
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(ref sm) = state {
        let info = try_state!(sm.da.create_app_bundle(id, engine, desc));
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": info.id,
                "engine": info.engine,
                "description": info.description,
                "version": info.version,
                "uploadParameters": {
                    "endpointUrl": format!("/mock-s3-upload/{}", info.id),
                    "formData": {
                        "key": format!("apps/mock-bucket/{}.zip", info.id),
                        "policy": "bW9jay1wb2xpY3k=",
                        "x-amz-signature": "mock-signature",
                        "x-amz-credential": "mock-credential",
                        "x-amz-date": "20260224T000000Z"
                    }
                }
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": "mock-bundle",
                "engine": "Autodesk.Revit+2025",
                "version": 1,
                "uploadParameters": {
                    "endpointUrl": "/mock-s3-upload/mock-bundle",
                    "formData": {
                        "key": "apps/mock-bucket/mock-bundle.zip",
                        "policy": "bW9jay1wb2xpY3k=",
                        "x-amz-signature": "mock-signature",
                        "x-amz-credential": "mock-credential",
                        "x-amz-date": "20260224T000000Z"
                    }
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_da_delete_appbundle(
    state: Option<StateManager>,
    bundle_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.da.delete_app_bundle(&bundle_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_da_create_appbundle_alias(
    state: Option<StateManager>,
    bundle_id: String,
    body: Value,
) -> impl IntoResponse {
    let alias_id = body.get("id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let version = body.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    if let Some(ref sm) = state {
        let alias = try_state!(sm.da.create_alias("appbundle", &bundle_id, alias_id, version));
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": alias.id, "version": alias.version, "receiver": alias.receiver })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": alias_id,
            "version": version,
            "receiver": bundle_id
        })),
    )
        .into_response()
}

pub async fn handle_da_list_activities(state: Option<StateManager>) -> impl IntoResponse {
    let data = if let Some(ref sm) = state {
        try_state!(sm.da
            .list_activities())
            .into_iter()
            .map(serde_json::Value::String)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({ "data": data, "paginationToken": null })),
    )
        .into_response()
}

pub async fn handle_da_create_activity(
    state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("activity")
        .to_string();
    let engine = body
        .get("engine")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let desc = body
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if let Some(ref sm) = state {
        let info = try_state!(sm.da.create_activity(id, engine, desc));
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": info.id,
                "engine": info.engine,
                "version": info.version
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": "mock-activity",
                "engine": "Autodesk.Revit+2025",
                "version": 1
            })),
        )
            .into_response()
    }
}

pub async fn handle_da_delete_activity(
    state: Option<StateManager>,
    activity_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.da.delete_activity(&activity_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_da_create_activity_alias(
    state: Option<StateManager>,
    activity_id: String,
    body: Value,
) -> impl IntoResponse {
    let alias_id = body.get("id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let version = body.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    if let Some(ref sm) = state {
        let alias = try_state!(sm.da.create_alias("activity", &activity_id, alias_id, version));
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": alias.id, "version": alias.version, "receiver": alias.receiver })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": alias_id,
            "version": version,
            "receiver": activity_id
        })),
    )
        .into_response()
}

pub async fn handle_da_create_workitem(
    state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    let activity_id = body
        .get("activityId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(ref sm) = state {
        let info = try_state!(sm.da.create_work_item(activity_id));
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": info.id,
                "status": info.status,
                "progress": info.progress
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": "workitem-mock-001",
                "status": "pending",
                "progress": null
            })),
        )
            .into_response()
    }
}

pub async fn handle_da_list_workitems(state: Option<StateManager>) -> impl IntoResponse {
    let data = if let Some(ref sm) = state {
        try_state!(sm.da
            .list_work_items())
            .into_iter()
            .map(|w| {
                json!({
                    "id": w.id,
                    "status": w.status,
                    "progress": w.progress,
                    "activityId": w.activity_id
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![json!({
            "id": "demo-workitem-001",
            "status": "success",
            "progress": "100%"
        })]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({ "data": data, "paginationToken": null })),
    )
        .into_response()
}

pub async fn handle_da_get_workitem(
    state: Option<StateManager>,
    workitem_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        if let Some(w) = try_state!(sm.da.get_work_item(&workitem_id)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": w.id,
                    "status": w.status,
                    "progress": w.progress,
                    "activityId": w.activity_id
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "id": workitem_id,
                    "status": "success",
                    "progress": "100%",
                    "reportUrl": null
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": workitem_id,
                "status": "success",
                "progress": "100%",
                "reportUrl": null
            })),
        )
            .into_response()
    }
}

// ---- Reality Capture ----

pub async fn handle_reality_list_photoscenes(state: Option<StateManager>) -> impl IntoResponse {
    let scenes = if let Some(ref sm) = state {
        try_state!(sm.reality
            .list_photoscenes())
            .into_iter()
            .map(|p| {
                json!({
                    "photosceneid": p.photoscene_id,
                    "name": p.name,
                    "scenetype": p.scene_type,
                    "status": p.status,
                    "progress": p.progress,
                    "progressmsg": p.progress_msg
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![json!({
            "photosceneid": "job-demo-001",
            "name": "Demo Scene",
            "scenetype": "object",
            "status": "Done",
            "progress": "100",
            "progressmsg": null
        })]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "Photoscenes": {
                "photoscene": scenes
            }
        })),
    )
        .into_response()
}

pub async fn handle_reality_create_photoscene(
    state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    // Body may come as JSON or form-encoded (parsed into Value)
    let name = body
        .get("scenename")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let scene_type = body
        .get("scenetype")
        .and_then(|v| v.as_str())
        .unwrap_or("object")
        .to_string();
    let format = body
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("rcm")
        .to_string();

    if let Some(ref sm) = state {
        let info = try_state!(sm.reality.create_photoscene(name, scene_type, format));
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "Photoscene": {
                    "photosceneid": info.photoscene_id,
                    "name": info.name,
                    "scenetype": info.scene_type,
                    "convertformat": info.convert_format,
                    "status": info.status,
                    "progress": info.progress
                }
            })),
        )
            .into_response()
    } else {
        let id = format!("ps-{}", uuid::Uuid::new_v4());
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "Photoscene": {
                    "photosceneid": id,
                    "name": name,
                    "scenetype": scene_type,
                    "convertformat": format,
                    "status": "Created",
                    "progress": "0"
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_reality_upload_file(
    _state: Option<StateManager>,
    body: Value,
) -> impl IntoResponse {
    let photoscene_id = body
        .get("photosceneid")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "Files": {
                "file": [{
                    "filename": "uploaded-file",
                    "fileid": format!("file-{}", uuid::Uuid::new_v4()),
                    "filesize": "1024",
                    "msg": "File uploaded",
                    "photosceneid": photoscene_id
                }]
            }
        })),
    )
        .into_response()
}

pub async fn handle_reality_process_photoscene(
    state: Option<StateManager>,
    photoscene_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.reality.process_photoscene(&photoscene_id));
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "Photoscene": {
                "photosceneid": photoscene_id
            }
        })),
    )
        .into_response()
}

pub async fn handle_reality_get_progress(
    state: Option<StateManager>,
    photoscene_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state && let Some(p) = try_state!(sm.reality.get_photoscene(&photoscene_id)) {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "Photoscene": {
                    "photosceneid": p.photoscene_id,
                    "progress": p.progress,
                    "progressmsg": p.progress_msg,
                    "status": p.status
                }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "Photoscene": {
                "photosceneid": photoscene_id,
                "progress": "100",
                "progressmsg": "Complete",
                "status": "Done"
            }
        })),
    )
        .into_response()
}

pub async fn handle_reality_get_result(
    state: Option<StateManager>,
    photoscene_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state && let Some(p) = try_state!(sm.reality.get_photoscene(&photoscene_id)) {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "Photoscene": {
                    "photosceneid": p.photoscene_id,
                    "progress": p.progress,
                    "progressmsg": p.progress_msg,
                    "scenelink": p.scene_link,
                    "filesize": "5242880"
                }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "Photoscene": {
                "photosceneid": photoscene_id,
                "progress": "100",
                "progressmsg": "Complete",
                "scenelink": "https://example.com/download/model.obj",
                "filesize": "5242880"
            }
        })),
    )
        .into_response()
}

pub async fn handle_reality_delete_photoscene(
    state: Option<StateManager>,
    photoscene_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.reality.delete_photoscene(&photoscene_id));
    }
    (axum::http::StatusCode::OK, JsonResponse(json!({}))).into_response()
}

// ---- OSS Signed S3 Upload / Download ----

pub async fn handle_signed_s3_download_content(
    state: Option<StateManager>,
    bucket_key: String,
    object_key: String,
) -> impl IntoResponse {
    // Mock S3 download — return dummy bytes (capped at 10MB to avoid memory issues)
    if let Some(ref state_manager) = state {
        if let Some(obj) = try_state!(state_manager.objects.get_object(&bucket_key, &object_key)) {
            let capped_size = std::cmp::min(obj.size, 10 * 1024 * 1024) as usize;
            let dummy_content = vec![0u8; capped_size];
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, obj.content_type.as_str())],
                dummy_content,
            )
                .into_response()
        } else {
            (axum::http::StatusCode::NOT_FOUND, "Object not found").into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
            vec![0u8; 0],
        )
            .into_response()
    }
}

// ---- Admin Users ----

pub async fn handle_admin_list_users(
    state: Option<StateManager>,
    account_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let users = try_state!(sm.admin.list_users(&account_id));
        let total = users.len() as i32;
        let results: Vec<Value> = users
            .into_iter()
            .map(|u| {
                json!({
                    "id": u.id,
                    "email": u.email,
                    "name": u.name,
                    "status": u.status,
                    "role": u.role
                })
            })
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [
                {
                    "id": "user-001",
                    "email": "alice@example.com",
                    "name": "Alice Johnson",
                    "status": "active",
                    "role": "project_admin"
                },
                {
                    "id": "user-002",
                    "email": "bob@example.com",
                    "name": "Bob Smith",
                    "status": "active",
                    "role": "project_user"
                }
            ],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 2 }
        })),
    )
        .into_response()
}

pub async fn handle_admin_add_user(
    state: Option<StateManager>,
    account_id: String,
    body: Value,
) -> impl IntoResponse {
    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("new@example.com")
        .to_string();
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("New User")
        .to_string();
    let role = body
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("project_user")
        .to_string();
    if let Some(ref sm) = state {
        let user = try_state!(sm.admin.add_user(&account_id, email, name, role));
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "status": user.status,
                "role": user.role
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "id": format!("user-{}", uuid::Uuid::new_v4()),
            "email": email,
            "name": name,
            "status": "active",
            "role": role
        })),
    )
        .into_response()
}

pub async fn handle_admin_search_users(
    state: Option<StateManager>,
    account_id: String,
    body: Value,
) -> impl IntoResponse {
    // The CLI deserializes this as a single AccountUser (not paginated)
    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("alice@example.com");
    if let Some(ref sm) = state
        && let Ok(Some(user)) = sm.admin.search_user(&account_id, email)
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "firstName": user.first_name,
                "lastName": user.last_name,
                "status": user.status,
                "companyId": user.company_id
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": format!("user-{}", uuid::Uuid::new_v4()),
            "email": email,
            "name": "Mock User",
            "firstName": "Mock",
            "lastName": "User",
            "status": "active",
            "companyId": "mock-company-001"
        })),
    )
        .into_response()
}

pub async fn handle_admin_update_user(
    state: Option<StateManager>,
    account_id: String,
    user_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let role = body.get("role").and_then(|v| v.as_str()).map(|s| s.to_string());
    let status = body.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    if let Some(ref sm) = state
        && let Ok(Some(user)) = sm.admin.update_user(&account_id, &user_id, name.clone(), role.clone(), status.clone())
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "status": user.status,
                "role": user.role
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": user_id,
            "email": "user@example.com",
            "name": name.unwrap_or_else(|| "Updated User".to_string()),
            "status": status.unwrap_or_else(|| "active".to_string()),
            "role": role.unwrap_or_else(|| "project_user".to_string())
        })),
    )
        .into_response()
}

pub async fn handle_admin_delete_user(
    state: Option<StateManager>,
    account_id: String,
    user_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.admin.delete_user(&account_id, &user_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_admin_import_users(
    state: Option<StateManager>,
    account_id: String,
    body: Value,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let users_arr = body.get("users").and_then(|v| v.as_array());
        let mut imports = Vec::new();
        if let Some(arr) = users_arr {
            for u in arr {
                let email = u.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let name = u.get("name").and_then(|v| v.as_str()).unwrap_or("Imported User").to_string();
                let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("project_user").to_string();
                imports.push((email, name, role));
            }
        }
        let (success, failures) = try_state!(sm.admin.import_users(&account_id, imports));
        let success_items: Vec<Value> = success
            .iter()
            .map(|u| json!({ "email": u.email, "status": u.status }))
            .collect();
        let failure_items: Vec<Value> = failures
            .iter()
            .map(|e| json!({ "email": e }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "success": success_items.len(),
                "failure": failure_items.len(),
                "success_items": success_items,
                "failure_items": failure_items
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "success": 2,
            "failure": 0,
            "success_items": [
                { "email": "imported1@example.com", "status": "active" },
                { "email": "imported2@example.com", "status": "active" }
            ],
            "failure_items": []
        })),
    )
        .into_response()
}

// ---- Admin Projects ----

pub async fn handle_admin_list_projects(
    state: Option<StateManager>,
    account_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let projects = try_state!(sm.admin.list_projects(&account_id));
        let total = projects.len() as i32;
        let results: Vec<Value> = projects
            .into_iter()
            .map(|p| json!({ "id": p.id, "name": p.name, "status": p.status, "accountId": p.account_id }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [
                { "id": "proj-001", "name": "Mock Project Alpha", "status": "active", "accountId": "mock-account-001" },
                { "id": "proj-002", "name": "Mock Project Beta", "status": "active", "accountId": "mock-account-001" }
            ],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 2 }
        })),
    )
        .into_response()
}

pub async fn handle_admin_create_project(
    state: Option<StateManager>,
    account_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("New Project")
        .to_string();
    if let Some(ref sm) = state {
        let project = try_state!(sm.admin.create_project(&account_id, name));
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({ "id": project.id, "name": project.name, "status": project.status, "accountId": project.account_id })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "id": format!("proj-{}", uuid::Uuid::new_v4()),
            "name": name,
            "status": "active",
            "accountId": account_id
        })),
    )
        .into_response()
}

pub async fn handle_admin_get_project(
    state: Option<StateManager>,
    account_id: String,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state
        && let Ok(Some(p)) = sm.admin.get_project(&account_id, &project_id)
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": p.id, "name": p.name, "status": p.status, "accountId": p.account_id })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": project_id,
            "name": "Mock Project",
            "status": "active",
            "accountId": account_id
        })),
    )
        .into_response()
}

pub async fn handle_admin_update_project(
    state: Option<StateManager>,
    account_id: String,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let status = body.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    if let Some(ref sm) = state
        && let Ok(Some(p)) = sm.admin.update_project(&account_id, &project_id, name.clone(), status.clone())
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": p.id, "name": p.name, "status": p.status, "accountId": p.account_id })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": project_id,
            "name": name.unwrap_or_else(|| "Updated Project".to_string()),
            "status": status.unwrap_or_else(|| "active".to_string()),
            "accountId": account_id
        })),
    )
        .into_response()
}

// ---- Admin Operations ----

pub async fn handle_admin_list_project_users(
    state: Option<StateManager>,
    _account_id: String,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let users = try_state!(sm.admin.list_project_users(&project_id));
        let total = users.len() as i32;
        let results: Vec<Value> = users
            .into_iter()
            .map(|u| json!({ "id": u.id, "email": u.email, "name": u.name, "status": u.status, "role": u.role_id }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [
                { "id": "user-001", "email": "alice@example.com", "name": "Alice Johnson", "status": "active", "role": "project_admin" }
            ],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 1 }
        })),
    )
        .into_response()
}

pub async fn handle_admin_get_job(
    state: Option<StateManager>,
    job_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state
        && let Ok(Some(job)) = sm.admin.get_job(&job_id)
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": job.id,
                "status": job.status,
                "progress": job.progress,
                "result": job.result
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": job_id,
            "status": "complete",
            "progress": "100%",
            "result": "success"
        })),
    )
        .into_response()
}

// ---- Project-level User Endpoints (used by raps admin user add/remove/update) ----

pub async fn handle_admin_get_project_user(
    state: Option<StateManager>,
    project_id: String,
    user_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state
        && let Ok(Some(pu)) = sm.admin.get_project_user(&project_id, &user_id)
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": pu.id,
                "email": pu.email,
                "name": pu.name,
                "status": pu.status,
                "projectId": pu.project_id,
                "roleId": pu.role_id
            })),
        )
            .into_response();
    }
    // Return 404 to indicate user doesn't exist yet (triggers add)
    (
        axum::http::StatusCode::NOT_FOUND,
        JsonResponse(json!({
            "code": "NOT_FOUND",
            "message": format!("User {} not found in project", user_id)
        })),
    )
        .into_response()
}

pub async fn handle_admin_add_project_user(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    let user_id = body.get("userId").and_then(|v| v.as_str()).unwrap_or("mock-user-id").to_string();
    let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("user@example.com").to_string();
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("Mock User").to_string();
    let role_id = body.get("roleId").and_then(|v| v.as_str()).unwrap_or("role-default").to_string();
    if let Some(ref sm) = state {
        let pu = try_state!(sm.admin.add_project_user(&project_id, user_id, email, name, role_id));
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "id": pu.id,
                "email": pu.email,
                "name": pu.name,
                "status": pu.status,
                "projectId": pu.project_id,
                "roleId": pu.role_id
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "id": "mock-user-id",
            "email": "user@example.com",
            "name": "Mock User",
            "status": "active",
            "projectId": project_id,
            "roleId": "role-default"
        })),
    )
        .into_response()
}

pub async fn handle_admin_list_project_users_v2(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let users = try_state!(sm.admin.list_project_users(&project_id));
        let total = users.len() as i32;
        let results: Vec<Value> = users
            .into_iter()
            .map(|u| json!({ "id": u.id, "email": u.email, "name": u.name, "status": u.status, "projectId": u.project_id, "roleId": u.role_id }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 0 }
        })),
    )
        .into_response()
}

pub async fn handle_admin_update_project_user(
    state: Option<StateManager>,
    project_id: String,
    user_id: String,
    body: Value,
) -> impl IntoResponse {
    let role_id = body.get("roleId").and_then(|v| v.as_str()).map(|s| s.to_string());
    if let Some(ref sm) = state
        && let Ok(Some(pu)) = sm.admin.update_project_user(&project_id, &user_id, role_id.clone())
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": pu.id,
                "email": pu.email,
                "status": pu.status,
                "projectId": pu.project_id,
                "roleId": pu.role_id
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": user_id,
            "email": "user@example.com",
            "status": "active",
            "projectId": project_id,
            "roleId": role_id.unwrap_or_else(|| "role-default".to_string())
        })),
    )
        .into_response()
}

pub async fn handle_admin_delete_project_user(
    state: Option<StateManager>,
    project_id: String,
    user_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        try_state!(sm.admin.delete_project_user(&project_id, &user_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

// ---- HQ Companies ----

pub async fn handle_hq_list_companies(
    state: Option<StateManager>,
    account_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let companies = try_state!(sm.admin.list_companies(&account_id));
        let results: Vec<Value> = companies
            .into_iter()
            .map(|c| json!({ "id": c.id, "name": c.name, "trade": c.trade }))
            .collect();
        return (axum::http::StatusCode::OK, JsonResponse(json!(results))).into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!([
            { "id": "comp-001", "name": "Mock Construction Co", "trade": "General Contractor" },
            { "id": "comp-002", "name": "Mock Engineering Ltd", "trade": "Electrical" }
        ])),
    )
        .into_response()
}

// ---- Data Management: URN helpers ----

/// Strip the `urn:adsk.wipprod:fs.folder:co.` prefix if present, returning the raw ID.
fn strip_folder_urn(urn: &str) -> &str {
    urn.strip_prefix("urn:adsk.wipprod:fs.folder:co.")
        .unwrap_or(urn)
}

/// Strip the `urn:adsk.wipprod:dm.lineage:` prefix if present, returning the raw ID.
fn strip_item_urn(urn: &str) -> &str {
    urn.strip_prefix("urn:adsk.wipprod:dm.lineage:")
        .unwrap_or(urn)
}

// ---- Data Management: Folders ----

pub async fn handle_dm_list_folder_contents(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
) -> impl IntoResponse {
    let Some(sm) = state else {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": []
            })),
        )
            .into_response();
    };

    let child_folders = try_state!(sm.folders.list_child_folders(&project_id, &folder_id));
    let items = try_state!(sm.items.list_items_in_folder(&project_id, &folder_id));

    let mut data: Vec<Value> = Vec::new();
    for f in child_folders {
        data.push(json!({
            "type": "folders",
            "id": format!("urn:adsk.wipprod:fs.folder:co.{}", f.id),
            "attributes": {
                "name": f.name,
                "displayName": f.display_name,
                "createTime": f.created_at,
                "lastModifiedTime": f.last_modified_time,
            }
        }));
    }
    for i in items {
        data.push(json!({
            "type": "items",
            "id": format!("urn:adsk.wipprod:dm.lineage:{}", i.id),
            "attributes": {
                "displayName": i.display_name,
                "createTime": i.created_at,
                "lastModifiedTime": i.last_modified_time,
            }
        }));
    }

    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": data
        })),
    )
        .into_response()
}

pub async fn handle_dm_create_folder(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("New Folder")
        .to_string();

    let parent = body
        .get("data")
        .and_then(|d| d.get("relationships"))
        .and_then(|r| r.get("parent"))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| strip_folder_urn(s).to_string());

    let Some(sm) = state else {
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "folders",
                    "id": format!("urn:adsk.wipprod:fs.folder:co.{}", uuid::Uuid::new_v4()),
                    "attributes": { "name": name, "displayName": name, "createTime": "2026-01-01T00:00:00Z" }
                }
            })),
        )
            .into_response();
    };

    let folder = try_state!(sm.folders.create_folder(project_id, parent, name));
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": {
                "type": "folders",
                "id": format!("urn:adsk.wipprod:fs.folder:co.{}", folder.id),
                "attributes": {
                    "name": folder.name,
                    "displayName": folder.display_name,
                    "createTime": folder.created_at,
                    "lastModifiedTime": folder.last_modified_time,
                }
            }
        })),
    )
        .into_response()
}

pub async fn handle_dm_get_folder(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
) -> impl IntoResponse {
    let raw_id = strip_folder_urn(&folder_id);

    let Some(sm) = state else {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "folders",
                    "id": folder_id,
                    "attributes": { "name": "Plans", "displayName": "Plans", "createTime": "2026-01-01T00:00:00Z" }
                }
            })),
        )
            .into_response();
    };

    let folder = try_state!(sm.folders.get_folder(&project_id, raw_id));
    match folder {
        Some(f) => (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "folders",
                    "id": format!("urn:adsk.wipprod:fs.folder:co.{}", f.id),
                    "attributes": {
                        "name": f.name,
                        "displayName": f.display_name,
                        "createTime": f.created_at,
                        "lastModifiedTime": f.last_modified_time,
                    }
                }
            })),
        )
            .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({"jsonapi": {"version": "1.0"}, "errors": [{"status": "404", "detail": "Folder not found"}]})),
        )
            .into_response(),
    }
}

pub async fn handle_dm_update_folder(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
    body: Value,
) -> impl IntoResponse {
    let raw_id = strip_folder_urn(&folder_id);
    let name = body
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let Some(sm) = state else {
        let display = name.as_deref().unwrap_or("Updated Folder");
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": { "type": "folders", "id": folder_id, "attributes": { "name": display, "displayName": display } }
            })),
        )
            .into_response();
    };

    let folder = try_state!(sm.folders.update_folder(&project_id, raw_id, name));
    match folder {
        Some(f) => (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "folders",
                    "id": format!("urn:adsk.wipprod:fs.folder:co.{}", f.id),
                    "attributes": {
                        "name": f.name,
                        "displayName": f.display_name,
                        "createTime": f.created_at,
                        "lastModifiedTime": f.last_modified_time,
                    }
                }
            })),
        )
            .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({"jsonapi": {"version": "1.0"}, "errors": [{"status": "404", "detail": "Folder not found"}]})),
        )
            .into_response(),
    }
}

pub async fn handle_dm_delete_folder(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
) -> impl IntoResponse {
    let raw_id = strip_folder_urn(&folder_id);
    if let Some(sm) = state {
        try_state!(sm.folders.delete_folder(&project_id, raw_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_dm_get_folder_permissions(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
) -> impl IntoResponse {
    let folder_id = strip_folder_urn(&folder_id).to_string();
    if let Some(ref sm) = state {
        let perms = try_state!(sm.folders.get_permissions(&project_id, &folder_id));
        let data: Vec<Value> = perms
            .into_iter()
            .map(|p| {
                json!({
                    "type": "folder-permissions",
                    "id": p.id,
                    "attributes": {
                        "subjectId": p.subject_id,
                        "subjectType": p.subject_type,
                        "actions": p.actions
                    }
                })
            })
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "jsonapi": { "version": "1.0" }, "data": data })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": [
                {
                    "type": "folder-permissions",
                    "id": "perm-001",
                    "attributes": {
                        "subjectId": "user-001",
                        "subjectType": "user",
                        "actions": ["view", "download", "collaborate"]
                    }
                }
            ]
        })),
    )
        .into_response()
}

pub async fn handle_dm_batch_update_folder_permissions(
    state: Option<StateManager>,
    project_id: String,
    folder_id: String,
    body: Value,
) -> impl IntoResponse {
    let folder_id = strip_folder_urn(&folder_id).to_string();
    if let Some(ref sm) = state {
        let mut results = Vec::new();
        if let Some(arr) = body.get("data").and_then(|v| v.as_array()) {
            for entry in arr {
                let attrs = entry.get("attributes").unwrap_or(&Value::Null);
                let subject_id = attrs.get("subjectId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let subject_type = attrs.get("subjectType").and_then(|v| v.as_str()).unwrap_or("user").to_string();
                let actions: Vec<String> = attrs
                    .get("actions")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                if let Ok(perm) = sm.folders.set_permission(&project_id, &folder_id, subject_id, subject_type, actions) {
                    results.push(json!({
                        "type": "folder-permissions",
                        "id": perm.id,
                        "attributes": {
                            "subjectId": perm.subject_id,
                            "subjectType": perm.subject_type,
                            "actions": perm.actions
                        }
                    }));
                }
            }
        }
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "jsonapi": { "version": "1.0" }, "data": results })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": []
        })),
    )
        .into_response()
}

pub async fn handle_dm_list_top_folders(
    state: Option<StateManager>,
    project_id: String,
) -> impl IntoResponse {
    let Some(sm) = state else {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": []
            })),
        )
            .into_response();
    };

    let folders = try_state!(sm.folders.list_top_folders(&project_id));
    let data: Vec<Value> = folders
        .into_iter()
        .map(|f| {
            json!({
                "type": "folders",
                "id": format!("urn:adsk.wipprod:fs.folder:co.{}", f.id),
                "attributes": {
                    "name": f.name,
                    "displayName": f.display_name,
                    "createTime": f.created_at,
                    "lastModifiedTime": f.last_modified_time,
                }
            })
        })
        .collect();

    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": data
        })),
    )
        .into_response()
}

// ---- Data Management: Items ----

pub async fn handle_dm_create_item(
    state: Option<StateManager>,
    project_id: String,
    body: Value,
) -> impl IntoResponse {
    let display_name = body
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("displayName"))
        .and_then(|v| v.as_str())
        .unwrap_or("NewFile.dwg")
        .to_string();

    let folder_id = body
        .get("data")
        .and_then(|d| d.get("relationships"))
        .and_then(|r| r.get("parent"))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| strip_folder_urn(s).to_string())
        .unwrap_or_default();

    let Some(sm) = state else {
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "items",
                    "id": format!("urn:adsk.wipprod:dm.lineage:{}", uuid::Uuid::new_v4()),
                    "attributes": { "displayName": display_name, "createTime": "2026-01-01T00:00:00Z" }
                }
            })),
        )
            .into_response();
    };

    let item = try_state!(sm.items.create_item(project_id, folder_id, display_name));
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": {
                "type": "items",
                "id": format!("urn:adsk.wipprod:dm.lineage:{}", item.id),
                "attributes": {
                    "displayName": item.display_name,
                    "createTime": item.created_at,
                    "lastModifiedTime": item.last_modified_time,
                }
            }
        })),
    )
        .into_response()
}

pub async fn handle_dm_get_item(
    state: Option<StateManager>,
    project_id: String,
    item_id: String,
) -> impl IntoResponse {
    let raw_id = strip_item_urn(&item_id);

    let Some(sm) = state else {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "items", "id": item_id,
                    "attributes": { "displayName": "MockFile.dwg", "createTime": "2026-01-01T00:00:00Z" }
                }
            })),
        )
            .into_response();
    };

    let item = try_state!(sm.items.get_item(&project_id, raw_id));
    match item {
        Some(i) => (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "items",
                    "id": format!("urn:adsk.wipprod:dm.lineage:{}", i.id),
                    "attributes": {
                        "displayName": i.display_name,
                        "createTime": i.created_at,
                        "lastModifiedTime": i.last_modified_time,
                    }
                }
            })),
        )
            .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({"jsonapi": {"version": "1.0"}, "errors": [{"status": "404", "detail": "Item not found"}]})),
        )
            .into_response(),
    }
}

pub async fn handle_dm_update_item(
    state: Option<StateManager>,
    project_id: String,
    item_id: String,
    body: Value,
) -> impl IntoResponse {
    let raw_id = strip_item_urn(&item_id);
    let display_name = body
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("displayName"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let Some(sm) = state else {
        let n = display_name.as_deref().unwrap_or("UpdatedFile.dwg");
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": { "type": "items", "id": item_id, "attributes": { "displayName": n } }
            })),
        )
            .into_response();
    };

    let item = try_state!(sm.items.update_item(&project_id, raw_id, display_name));
    match item {
        Some(i) => (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": {
                    "type": "items",
                    "id": format!("urn:adsk.wipprod:dm.lineage:{}", i.id),
                    "attributes": {
                        "displayName": i.display_name,
                        "createTime": i.created_at,
                        "lastModifiedTime": i.last_modified_time,
                    }
                }
            })),
        )
            .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            JsonResponse(json!({"jsonapi": {"version": "1.0"}, "errors": [{"status": "404", "detail": "Item not found"}]})),
        )
            .into_response(),
    }
}

pub async fn handle_dm_delete_item(
    state: Option<StateManager>,
    project_id: String,
    item_id: String,
) -> impl IntoResponse {
    let raw_id = strip_item_urn(&item_id);
    if let Some(sm) = state {
        try_state!(sm.items.delete_item(&project_id, raw_id));
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_dm_list_item_versions(
    state: Option<StateManager>,
    _project_id: String,
    item_id: String,
) -> impl IntoResponse {
    let raw_id = strip_item_urn(&item_id);

    let Some(sm) = state else {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "jsonapi": { "version": "1.0" },
                "data": []
            })),
        )
            .into_response();
    };

    let versions = try_state!(sm.items.list_versions(raw_id));
    let data: Vec<Value> = versions
        .into_iter()
        .map(|v| {
            json!({
                "type": "versions",
                "id": format!("urn:adsk.wipprod:fs.file:vf.{}", v.id),
                "attributes": {
                    "name": v.display_name,
                    "displayName": v.display_name,
                    "versionNumber": v.version_number,
                    "createTime": v.created_at,
                    "storageSize": v.storage_size,
                }
            })
        })
        .collect();

    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": data
        })),
    )
        .into_response()
}

// ---- Data Management: Project Info ----

pub async fn handle_dm_get_project(
    _state: Option<StateManager>,
    _hub_id: String,
    project_id: String,
) -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "jsonapi": { "version": "1.0" },
            "data": {
                "type": "projects",
                "id": project_id,
                "attributes": {
                    "name": "Mock Project"
                }
            }
        })),
    )
        .into_response()
}

// ---- Project Templates ----

pub async fn handle_list_project_templates(
    state: Option<StateManager>,
    account_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state {
        let templates = try_state!(sm.admin.list_templates(&account_id));
        let total = templates.len() as i32;
        let results: Vec<Value> = templates
            .into_iter()
            .map(|t| json!({ "id": t.id, "name": t.name, "status": t.status }))
            .collect();
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "results": results,
                "pagination": { "limit": 50, "offset": 0, "totalResults": total }
            })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "results": [
                { "id": "tmpl-001", "name": "Default Template", "status": "active" },
                { "id": "tmpl-002", "name": "Construction Template", "status": "active" }
            ],
            "pagination": { "limit": 50, "offset": 0, "totalResults": 2 }
        })),
    )
        .into_response()
}

pub async fn handle_create_project_template(
    state: Option<StateManager>,
    account_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("New Template")
        .to_string();
    if let Some(ref sm) = state {
        let t = try_state!(sm.admin.create_template(&account_id, name));
        return (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({ "id": t.id, "name": t.name, "status": t.status })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::CREATED,
        JsonResponse(json!({
            "id": format!("tmpl-{}", uuid::Uuid::new_v4()),
            "name": name,
            "status": "active"
        })),
    )
        .into_response()
}

pub async fn handle_get_project_template(
    state: Option<StateManager>,
    account_id: String,
    template_id: String,
) -> impl IntoResponse {
    if let Some(ref sm) = state
        && let Ok(Some(t)) = sm.admin.get_template(&account_id, &template_id)
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": t.id, "name": t.name, "status": t.status })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": template_id,
            "name": "Default Template",
            "status": "active"
        })),
    )
        .into_response()
}

pub async fn handle_update_project_template(
    state: Option<StateManager>,
    account_id: String,
    template_id: String,
    body: Value,
) -> impl IntoResponse {
    let name = body.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let status = body.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    if let Some(ref sm) = state
        && let Ok(Some(t)) = sm.admin.update_template(&account_id, &template_id, name.clone(), status.clone())
    {
        return (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "id": t.id, "name": t.name, "status": t.status })),
        )
            .into_response();
    }
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "id": template_id,
            "name": name.unwrap_or_else(|| "Updated Template".to_string()),
            "status": status.unwrap_or_else(|| "active".to_string())
        })),
    )
        .into_response()
}

// ---- Webhook Events ----

pub async fn handle_list_webhook_events(_state: Option<StateManager>) -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({
            "data": [
                { "event": "dm.version.added" },
                { "event": "dm.version.copied" },
                { "event": "dm.version.deleted" },
                { "event": "dm.version.modified" },
                { "event": "dm.folder.added" },
                { "event": "dm.folder.modified" }
            ]
        })),
    )
        .into_response()
}

// ---- Model Derivative Metadata ----

pub async fn handle_get_metadata(state: Option<StateManager>, urn: String) -> impl IntoResponse {
    let decoded_urn = decode_base64_urn(&urn);

    if let Some(ref state_manager) = state {
        if let Some(metadata) = try_state!(state_manager.translations.get_metadata(&decoded_urn)) {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({ "data": metadata })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Translation not found for URN {}", decoded_urn)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": {
                    "type": "metadata",
                    "metadata": [{
                        "guid": "mock-guid-001",
                        "name": "3D View",
                        "role": "3d"
                    }]
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_object_tree(
    state: Option<StateManager>,
    urn: String,
    guid: String,
) -> impl IntoResponse {
    let decoded_urn = decode_base64_urn(&urn);

    if let Some(ref state_manager) = state {
        if let Some(tree) = try_state!(state_manager
            .translations
            .get_object_tree(&decoded_urn, &guid))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({ "data": tree })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": "Object tree not found"
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": {
                    "type": "objects",
                    "objects": [{ "objectid": 1, "name": "Model", "objects": [] }]
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_get_properties(
    state: Option<StateManager>,
    urn: String,
    guid: String,
) -> impl IntoResponse {
    let decoded_urn = decode_base64_urn(&urn);

    if let Some(ref state_manager) = state {
        if let Some(props) = try_state!(state_manager
            .translations
            .get_properties(&decoded_urn, &guid))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({ "data": props })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": "Properties not found"
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": {
                    "type": "properties",
                    "collection": []
                }
            })),
        )
            .into_response()
    }
}

pub async fn handle_query_properties(
    state: Option<StateManager>,
    urn: String,
    guid: String,
    body: Value,
) -> impl IntoResponse {
    let decoded_urn = decode_base64_urn(&urn);

    if let Some(ref state_manager) = state {
        if let Some(props) = try_state!(state_manager
            .translations
            .get_properties(&decoded_urn, &guid))
        {
            // Filter by object IDs if query contains $in operator
            let filtered = if let Some(query) = body.get("query") {
                if let Some(in_arr) = query.get("$in").and_then(|v| v.as_array()) {
                    let object_ids: Vec<i64> = in_arr
                        .iter()
                        .skip(1) // first element is field name "objectid"
                        .filter_map(|v| v.as_i64())
                        .collect();

                    if let Some(collection) = props.get("collection").and_then(|c| c.as_array()) {
                        let filtered_items: Vec<&Value> = collection
                            .iter()
                            .filter(|item| {
                                item.get("objectid")
                                    .and_then(|id| id.as_i64())
                                    .map(|id| object_ids.contains(&id))
                                    .unwrap_or(false)
                            })
                            .collect();
                        json!({
                            "type": "properties",
                            "collection": filtered_items
                        })
                    } else {
                        props
                    }
                } else {
                    props
                }
            } else {
                props
            };

            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({ "data": filtered })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": "Properties not found"
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "data": {
                    "type": "properties",
                    "collection": []
                }
            })),
        )
            .into_response()
    }
}

// ---- OSS Copy ----

pub async fn handle_copy_object(
    state: Option<StateManager>,
    dest_bucket: String,
    dest_key: String,
    copy_from: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        // Parse x-ads-copy-from: "bucket_key/objects/object_key"
        let parts: Vec<&str> = copy_from.splitn(3, '/').collect();
        let (src_bucket, src_key) = if parts.len() == 3 && parts[1] == "objects" {
            (parts[0], parts[2])
        } else {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                JsonResponse(json!({
                    "code": "BAD_REQUEST",
                    "message": format!("Invalid x-ads-copy-from header: {}", copy_from)
                })),
            )
                .into_response();
        };

        if let Some(copied) =
            try_state!(state_manager
                .objects
                .copy_object(src_bucket, src_key, &dest_bucket, &dest_key))
        {
            (
                axum::http::StatusCode::OK,
                JsonResponse(json!({
                    "bucketKey": copied.bucket_key,
                    "objectKey": copied.object_key,
                    "objectId": copied.object_id,
                    "sha1": copied.sha1,
                    "size": copied.size,
                    "contentType": copied.content_type,
                    "location": copied.location
                })),
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "code": "NOT_FOUND",
                    "message": format!("Source object not found: {}/objects/{}", src_bucket, src_key)
                })),
            )
                .into_response()
        }
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "bucketKey": dest_bucket,
                "objectKey": dest_key,
                "objectId": format!("urn:adsk.objects:os.object:{}/{}", dest_bucket, dest_key),
                "sha1": "mock-sha1",
                "size": 1024,
                "contentType": "application/octet-stream",
                "location": format!("/oss/v2/buckets/{}/objects/{}", dest_bucket, dest_key)
            })),
        )
            .into_response()
    }
}

// ---- DA App Bundle Upload ----

pub async fn handle_mock_s3_upload(
    _bundle_id: String,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Accept any multipart form data and return 200 OK
    // In a real scenario, this validates form fields; for the mock, just accept it
    if body.is_empty() {
        (
            axum::http::StatusCode::BAD_REQUEST,
            JsonResponse(json!({
                "code": "BAD_REQUEST",
                "message": "Empty upload body"
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "status": "ok" })),
        )
            .into_response()
    }
}

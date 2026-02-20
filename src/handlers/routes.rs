// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Extracted route handlers for hardcoded endpoints.
//!
//! Each handler is an async function that takes an `Option<StateManager>` plus
//! any request-specific parameters, and returns an `impl IntoResponse`.

use axum::response::{IntoResponse, Json as JsonResponse};
use base64::Engine as _;
use serde_json::{Value, json};

use crate::state::StateManager;

// ---- Auth ----

pub async fn handle_auth_token(state: Option<StateManager>, body: Value) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let client_id = body
            .get("client_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default-client");

        let scope = body
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let token = state_manager.auth.generate_token(client_id, 3600, scope);
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "access_token": token.access_token,
                "token_type": token.token_type,
                "expires_in": token.expires_in
            })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "access_token": "mock-token",
                "token_type": "Bearer",
                "expires_in": 3600
            })),
        )
            .into_response()
    }
}

// ---- OSS Buckets ----

pub async fn handle_list_buckets(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let buckets = state_manager.buckets.list_buckets();
        let items: Vec<Value> = buckets
            .into_iter()
            .map(|b| {
                json!({
                    "bucketKey": b.bucket_key,
                    "createdDate": b.created_date,
                    "policyKey": b.policy_key
                })
            })
            .collect();
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": items })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": [] })),
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

        let bucket = state_manager
            .buckets
            .create_bucket(bucket_key.to_string(), policy_key.to_string());

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
        let objects = state_manager.objects.list_objects(&bucket_key);
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
            JsonResponse(json!({ "items": items })),
        )
            .into_response()
    } else {
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({ "items": [] })),
        )
            .into_response()
    }
}

// ---- Data Management ----

pub async fn handle_list_hubs(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let hubs = state_manager.projects.list_hubs();
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
        if let Some(hub) = state_manager.projects.get_hub(&hub_id) {
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
        let projects = state_manager.projects.list_projects(&hub_id);
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

        let job = state_manager.translations.create_job(input_urn.to_string());

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
    let decoded_urn = match base64::engine::general_purpose::STANDARD.decode(&urn) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => urn.clone(),
    };

    if let Some(ref state_manager) = state {
        if let Some(job) = state_manager.translations.get_job(&decoded_urn) {
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
        let issues = state_manager.issues.list_issues(&project_id);
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

        let issue = state_manager
            .issues
            .create_issue(project_id, title, description);

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

// ---- Webhooks ----

pub async fn handle_list_webhooks(
    state: Option<StateManager>,
    system: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let subscriptions = state_manager.webhooks.list_subscriptions();
        let data: Vec<Value> = subscriptions
            .into_iter()
            .filter(|s| s.tenant == system)
            .map(|s| {
                json!({
                    "hookId": s.hook_id,
                    "tenant": s.tenant,
                    "callbackUrl": s.callback_url,
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
            project: body
                .get("scope")
                .and_then(|s| s.get("project"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        let subscription = state_manager
            .webhooks
            .create_subscription(system, callback_url, scope);

        (
            axum::http::StatusCode::CREATED,
            JsonResponse(json!({
                "hookId": subscription.hook_id,
                "tenant": subscription.tenant,
                "callbackUrl": subscription.callback_url,
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
        if state_manager.webhooks.delete_subscription(&hook_id) {
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

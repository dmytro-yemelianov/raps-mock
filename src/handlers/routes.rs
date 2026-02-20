// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Extracted route handlers for hardcoded endpoints.
//!
//! Each handler is an async function that takes an `Option<StateManager>` plus
//! any request-specific parameters, and returns an `impl IntoResponse`.

use axum::response::{IntoResponse, Json as JsonResponse};
use base64::Engine as _;  // needed for .decode() method on engine instances
use serde_json::{Value, json};

use crate::state::StateManager;

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

        // Decode the base64 URN before storing so lookups match
        let decoded_urn = decode_base64_urn(input_urn);
        let job = state_manager.translations.create_job(decoded_urn);

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

pub async fn handle_list_all_webhooks(state: Option<StateManager>) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        let subscriptions = state_manager.webhooks.list_subscriptions();
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
        let subscriptions = state_manager.webhooks.list_subscriptions();
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

        let subscription = state_manager.webhooks.create_subscription(
            system.clone(),
            callback_url,
            event.clone(),
            system,
            scope,
        );

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

// ---- OSS Bucket Details / Delete ----

pub async fn handle_get_bucket(
    state: Option<StateManager>,
    bucket_key: String,
) -> impl IntoResponse {
    if let Some(ref state_manager) = state {
        if let Some(bucket) = state_manager.buckets.get_bucket(&bucket_key) {
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
        if state_manager.buckets.delete_bucket(&bucket_key) {
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
        if let Some(obj) = state_manager.objects.get_object(&bucket_key, &object_key) {
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
        if state_manager.objects.delete_object(&bucket_key, &object_key) {
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
    let _upload_key = body
        .get("uploadKey")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if let Some(ref state_manager) = state {
        // Return existing object if the mock-s3 PUT already stored it
        let obj = state_manager
            .objects
            .get_object(&bucket_key, &object_key)
            .unwrap_or_else(|| {
                state_manager
                    .objects
                    .upload_object(bucket_key, object_key, 0, None)
            });
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
        state_manager
            .objects
            .upload_object(bucket_key, object_key, size, None);
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
        if let Some(obj) = state_manager.objects.get_object(&bucket_key, &object_key) {
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
            .map(|e| serde_json::Value::String(e))
            .collect::<Vec<_>>()
    } else {
        vec![
            json!("Autodesk.Revit+2025"),
            json!("Autodesk.AutoCAD+24"),
        ]
    };
    (
        axum::http::StatusCode::OK,
        JsonResponse(json!({ "data": data, "paginationToken": null })),
    )
        .into_response()
}

pub async fn handle_da_list_appbundles(state: Option<StateManager>) -> impl IntoResponse {
    let data = if let Some(ref sm) = state {
        sm.da
            .list_app_bundles()
            .into_iter()
            .map(|b| serde_json::Value::String(b))
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
        let info = sm.da.create_app_bundle(id, engine, desc);
        (
            axum::http::StatusCode::OK,
            JsonResponse(json!({
                "id": info.id,
                "engine": info.engine,
                "description": info.description,
                "version": info.version,
                "uploadParameters": {
                    "endpointUrl": "https://example.com/upload",
                    "formData": {}
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
                "uploadParameters": { "endpointUrl": "https://example.com/upload", "formData": {} }
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
        sm.da.delete_app_bundle(&bundle_id);
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_da_create_appbundle_alias(
    _state: Option<StateManager>,
    bundle_id: String,
    body: Value,
) -> impl IntoResponse {
    let alias_id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let version = body.get("version").and_then(|v| v.as_u64()).unwrap_or(1);
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
        sm.da
            .list_activities()
            .into_iter()
            .map(|a| serde_json::Value::String(a))
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
        let info = sm.da.create_activity(id, engine, desc);
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
        sm.da.delete_activity(&activity_id);
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn handle_da_create_activity_alias(
    _state: Option<StateManager>,
    activity_id: String,
    body: Value,
) -> impl IntoResponse {
    let alias_id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let version = body.get("version").and_then(|v| v.as_u64()).unwrap_or(1);
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
        let info = sm.da.create_work_item(activity_id);
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
        sm.da
            .list_work_items()
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
        if let Some(w) = sm.da.get_work_item(&workitem_id) {
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
        sm.reality
            .list_photoscenes()
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
        let info = sm.reality.create_photoscene(name, scene_type, format);
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
        sm.reality.process_photoscene(&photoscene_id);
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
    if let Some(ref sm) = state {
        if let Some(p) = sm.reality.get_photoscene(&photoscene_id) {
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
    if let Some(ref sm) = state {
        if let Some(p) = sm.reality.get_photoscene(&photoscene_id) {
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
        sm.reality.delete_photoscene(&photoscene_id);
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
        if let Some(obj) = state_manager.objects.get_object(&bucket_key, &object_key) {
            let capped_size = std::cmp::min(obj.size, 10 * 1024 * 1024) as usize;
            let dummy_content = vec![0u8; capped_size];
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, obj.content_type.as_str())],
                dummy_content,
            )
                .into_response()
        } else {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Object not found",
            )
                .into_response()
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

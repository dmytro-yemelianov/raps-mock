// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025

use axum::{
    Router,
    extract::{Json, Path},
    response::{IntoResponse, Json as JsonResponse},
    routing::{delete, get, patch, post, put},
};
use base64::Engine as _;
use serde_json::{Value, json};

use crate::error::Result;
use crate::middleware::{auth_middleware, cors_middleware};
use crate::openapi::types::{HttpMethod, RouteDefinition};
use crate::state::StateManager;

pub fn build_router(routes: Vec<RouteDefinition>, state: Option<StateManager>) -> Result<Router> {
    let mut router = Router::new();
    let mut registered_routes = std::collections::HashSet::new();

    // Clone state for use in closures
    let state_clone = state.clone();

    // 1. Register dynamic routes from OpenAPI specs
    for route in routes {
        let path = route.path_pattern.clone();
        let method = route.method;

        if !registered_routes.insert((path.clone(), method)) {
            tracing::debug!(
                "Skipping duplicate dynamic route: {} {}",
                method.as_str(),
                path
            );
            continue;
        }

        let handler = std::sync::Arc::new(crate::handlers::GenericHandler::new(route));
        let handler_clone = handler.clone();
        let service = move || async move { handler_clone.handle().await };

        router = match method {
            HttpMethod::Get => router.route(&path, get(service)),
            HttpMethod::Post => router.route(&path, post(service)),
            HttpMethod::Put => router.route(&path, put(service)),
            HttpMethod::Delete => router.route(&path, delete(service)),
            HttpMethod::Patch => router.route(&path, patch(service)),
        };
    }

    // 2. Register hardcoded routes (fallback for what's not in OpenAPI)
    router = register_hardcoded_routes(router, state_clone.clone(), &mut registered_routes);

    // Apply middleware
    router = router
        .layer(cors_middleware())
        .layer(axum::middleware::from_fn(auth_middleware));

    // Add state as extension for middleware access (if stateful mode)
    if let Some(state_manager) = state {
        router = router.layer(axum::Extension(state_manager));
    }

    Ok(router)
}

fn register_hardcoded_routes(
    mut router: Router,
    state: Option<StateManager>,
    registered: &mut std::collections::HashSet<(String, HttpMethod)>,
) -> Router {
    // Helper to add route only if not already registered
    let mut add_route =
        |router: Router, path: &str, method: HttpMethod, handler: axum::routing::MethodRouter| {
            if registered.insert((path.to_string(), method)) {
                router.route(path, handler)
            } else {
                tracing::debug!(
                    "Skipping hardcoded route (already covered by OpenAPI): {} {}",
                    method.as_str(),
                    path
                );
                router
            }
        };

    // Authentication endpoints
    let auth_state = state.clone();
    router = add_route(
        router,
        "/authentication/v2/token",
        HttpMethod::Post,
        post(move |Json(body_value): Json<Value>| {
            let state_inner = auth_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
                    let client_id = body_value
                        .get("client_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default-client");

                    let scope = body_value
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
        }),
    );

    // OSS endpoints
    let oss_state = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets",
        HttpMethod::Get,
        get(move || {
            let state_inner = oss_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
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
        }),
    );

    let oss_state = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets",
        HttpMethod::Post,
        post(move |Json(body_value): Json<Value>| {
            let state_inner = oss_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
                    let bucket_key = body_value
                        .get("bucketKey")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default-bucket");

                    let policy_key = body_value
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
        }),
    );

    let oss_state = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucketKey/objects",
        HttpMethod::Get,
        get(move |Path(bucket_key): Path<String>| {
            let state_inner = oss_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
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
        }),
    );

    // Data Management endpoints
    let dm_state = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs",
        HttpMethod::Get,
        get(move || {
            let state_inner = dm_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
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
        }),
    );

    let dm_state = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs/:hubId",
        HttpMethod::Get,
        get(move |Path(hub_id): Path<String>| {
            let state_inner = dm_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
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
        }),
    );

    let dm_state = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs/:hubId/projects",
        HttpMethod::Get,
        get(move |Path(hub_id): Path<String>| {
            let state_inner = dm_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
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
        }),
    );

    // Model Derivative endpoints
    let md_state = state.clone();
    router = add_route(
        router,
        "/modelderivative/v2/designdata/job",
        HttpMethod::Post,
        post(move |Json(body_value): Json<Value>| {
            let state_inner = md_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
                    let input_urn = body_value
                        .get("input")
                        .and_then(|i| i.get("urn"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let output_type = body_value
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
                            "acceptedJobs": { "type": output_type }
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
        }),
    );

    let md_state = state.clone();
    router = add_route(
        router,
        "/modelderivative/v2/designdata/:urn/manifest",
        HttpMethod::Get,
        get(move |Path(urn): Path<String>| {
            let state_inner = md_state.clone();
            async move {
                let decoded_urn = match base64::engine::general_purpose::STANDARD.decode(&urn) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => urn.clone(),
                };

                if let Some(ref state_manager) = state_inner {
                    if let Some(job) = state_manager.translations.get_job(&decoded_urn) {
                        let status_str = match job.status {
                            crate::state::translations::TranslationStatus::Pending => "pending",
                            crate::state::translations::TranslationStatus::InProgress => {
                                "inprogress"
                            }
                            crate::state::translations::TranslationStatus::Success => "success",
                            crate::state::translations::TranslationStatus::Failed => "failed",
                        };

                        let manifest = json!({
                            "type": "manifest",
                            "hasThumbnail": status_str == "success",
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
                            "hasThumbnail": false,
                            "status": "pending",
                            "progress": "0%",
                            "region": "US",
                            "urn": decoded_urn,
                            "derivatives": []
                        })),
                    )
                        .into_response()
                }
            }
        }),
    );

    // Construction/ACC Issues endpoints
    let issues_state = state.clone();
    router = add_route(
        router,
        "/construction/issues/v1/projects/:projectId/issues",
        HttpMethod::Get,
        get(move |Path(project_id): Path<String>| {
            let state_inner = issues_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
                    let issues = state_manager.issues.list_issues(&project_id);
                    let data: Vec<Value> = issues
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
                        JsonResponse(json!({ "data": data })),
                    )
                        .into_response()
                } else {
                    (
                        axum::http::StatusCode::OK,
                        JsonResponse(json!({ "data": [] })),
                    )
                        .into_response()
                }
            }
        }),
    );

    let issues_state = state.clone();
    router = add_route(
        router,
        "/construction/issues/v1/projects/:projectId/issues",
        HttpMethod::Post,
        post(
            move |Path(project_id): Path<String>, Json(body_value): Json<Value>| {
                let state_inner = issues_state.clone();
                async move {
                    if let Some(ref state_manager) = state_inner {
                        let title = body_value
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Untitled Issue")
                            .to_string();

                        let description = body_value
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let issue =
                            state_manager
                                .issues
                                .create_issue(project_id, title, description);

                        (
                            axum::http::StatusCode::CREATED,
                            JsonResponse(json!({
                                "data": {
                                    "id": issue.id,
                                    "title": issue.title,
                                    "description": issue.description,
                                    "status": issue.status,
                                    "createdAt": issue.created_at
                                }
                            })),
                        )
                            .into_response()
                    } else {
                        (
                            axum::http::StatusCode::CREATED,
                            JsonResponse(json!({
                                "data": {
                                    "id": "mock-issue-id",
                                    "title": "Mock Issue",
                                    "status": "open"
                                }
                            })),
                        )
                            .into_response()
                    }
                }
            },
        ),
    );

    // Webhooks endpoints
    let webhooks_state = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks",
        HttpMethod::Get,
        get(move |Path((system, _event)): Path<(String, String)>| {
            let state_inner = webhooks_state.clone();
            async move {
                if let Some(ref state_manager) = state_inner {
                    let subscriptions = state_manager.webhooks.list_subscriptions();
                    let hooks: Vec<Value> = subscriptions
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
                        JsonResponse(json!({ "hooks": hooks })),
                    )
                        .into_response()
                } else {
                    (
                        axum::http::StatusCode::OK,
                        JsonResponse(json!({ "hooks": [] })),
                    )
                        .into_response()
                }
            }
        }),
    );

    let webhooks_state = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks",
        HttpMethod::Post,
        post(
            move |Path((system, _event)): Path<(String, String)>, Json(body_value): Json<Value>| {
                let state_inner = webhooks_state.clone();
                async move {
                    if let Some(ref state_manager) = state_inner {
                        let callback_url = body_value
                            .get("callbackUrl")
                            .and_then(|v| v.as_str())
                            .unwrap_or("https://example.com/webhook")
                            .to_string();

                        let scope = crate::state::webhooks::WebhookScope {
                            folder: body_value
                                .get("scope")
                                .and_then(|s| s.get("folder"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            project: body_value
                                .get("scope")
                                .and_then(|s| s.get("project"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        };

                        let subscription =
                            state_manager
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
            },
        ),
    );

    let webhooks_state = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks/:hookId",
        HttpMethod::Delete,
        delete(
            move |Path((_system, _event, hook_id)): Path<(String, String, String)>| {
                let state_inner = webhooks_state.clone();
                async move {
                    if let Some(ref state_manager) = state_inner {
                        if state_manager.webhooks.delete_subscription(&hook_id) {
                            (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({})))
                                .into_response()
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
                        (axum::http::StatusCode::NO_CONTENT, JsonResponse(json!({})))
                            .into_response()
                    }
                }
            },
        ),
    );

    router
}

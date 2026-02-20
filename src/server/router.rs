// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025

use axum::{
    Router,
    extract::{Json, Path},
    routing::{delete, get, patch, post, put},
};
use serde_json::Value;

use crate::error::Result;
use crate::handlers::routes;
use crate::middleware::{auth_middleware, cors_middleware};
use crate::openapi::types::{HttpMethod, RouteDefinition};
use crate::state::StateManager;

pub fn build_router(
    routes_defs: Vec<RouteDefinition>,
    state: Option<StateManager>,
) -> Result<Router> {
    let mut router = Router::new();
    let mut registered_routes = std::collections::HashSet::new();

    // Clone state for use in closures
    let state_clone = state.clone();

    // 1. Register hardcoded routes first (stateful handlers take priority)
    router = register_hardcoded_routes(router, state_clone.clone(), &mut registered_routes);

    // Reserve paths that will be registered outside middleware (prevents OpenAPI duplicates)
    registered_routes.insert(("/userinfo".to_string(), HttpMethod::Get));

    // 2. Register dynamic routes from OpenAPI specs (fill gaps not covered above)
    for route in routes_defs {
        let path = route.path_pattern.clone();
        let method = route.method;

        if !registered_routes.insert((path.clone(), method)) {
            tracing::debug!(
                "Skipping dynamic route (already covered by hardcoded): {} {}",
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

    // Apply middleware
    router = router
        .layer(cors_middleware())
        .layer(axum::middleware::from_fn(auth_middleware));

    // Add state as extension for middleware access (if stateful mode)
    if let Some(state_manager) = state {
        router = router.layer(axum::Extension(state_manager));
    }

    // Health check (outside auth middleware so it bypasses auth)
    router = router.route(
        "/health",
        get(|| async {
            axum::Json(serde_json::json!({"status": "ok", "service": "raps-mock"}))
        }),
    );

    // User info endpoint (outside auth middleware, used by raps auth login --token)
    router = router.route("/userinfo", get(routes::handle_userinfo));

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
                    "Skipping duplicate hardcoded route: {} {}",
                    method.as_str(),
                    path
                );
                router
            }
        };

    // Authentication endpoints (accept both JSON and form-encoded bodies)
    let s = state.clone();
    router = add_route(
        router,
        "/authentication/v2/token",
        HttpMethod::Post,
        post(move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
            let state = s.clone();
            async move {
                let content_type = headers
                    .get(axum::http::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                let parsed: Value = if content_type.contains("application/x-www-form-urlencoded") {
                    let text = String::from_utf8_lossy(&body);
                    let mut map = serde_json::Map::new();
                    for pair in text.split('&') {
                        if let Some((k, v)) = pair.split_once('=') {
                            map.insert(k.to_string(), Value::String(v.to_string()));
                        }
                    }
                    Value::Object(map)
                } else {
                    serde_json::from_slice(&body).unwrap_or_default()
                };
                routes::handle_auth_token(state, parsed).await
            }
        }),
    );

    // OSS endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_list_buckets(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_create_bucket(state, body).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects",
        HttpMethod::Get,
        get(move |Path(bucket_key): Path<String>| {
            let state = s.clone();
            async move { routes::handle_list_objects(state, bucket_key).await }
        }),
    );

    // Bucket details
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/details",
        HttpMethod::Get,
        get(move |Path(bucket_key): Path<String>| {
            let state = s.clone();
            async move { routes::handle_get_bucket(state, bucket_key).await }
        }),
    );

    // Bucket delete
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key",
        HttpMethod::Delete,
        delete(move |Path(bucket_key): Path<String>| {
            let state = s.clone();
            async move { routes::handle_delete_bucket(state, bucket_key).await }
        }),
    );

    // Object details
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/details",
        HttpMethod::Get,
        get(move |Path((bucket_key, object_key)): Path<(String, String)>| {
            let state = s.clone();
            async move { routes::handle_get_object_details(state, bucket_key, object_key).await }
        }),
    );

    // Object delete
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key",
        HttpMethod::Delete,
        delete(
            move |Path((bucket_key, object_key)): Path<(String, String)>| {
                let state = s.clone();
                async move { routes::handle_delete_object(state, bucket_key, object_key).await }
            },
        ),
    );

    // Signed S3 upload URL (GET = get URL, POST = complete upload)
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/signeds3upload",
        HttpMethod::Get,
        get(
            move |Path((bucket_key, object_key)): Path<(String, String)>,
                  headers: axum::http::HeaderMap| {
                let state = s.clone();
                async move {
                    let host = headers
                        .get(axum::http::header::HOST)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("localhost:3000")
                        .to_string();
                    routes::handle_signed_s3_upload_get(state, bucket_key, object_key, host).await
                }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/signeds3upload",
        HttpMethod::Post,
        post(
            move |Path((bucket_key, object_key)): Path<(String, String)>,
                  Json(body): Json<Value>| {
                let state = s.clone();
                async move {
                    routes::handle_signed_s3_upload_complete(state, bucket_key, object_key, body)
                        .await
                }
            },
        ),
    );

    // Mock S3 PUT endpoint for actual file upload
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/signeds3upload/mock-s3",
        HttpMethod::Put,
        put(
            move |Path((bucket_key, object_key)): Path<(String, String)>,
                  body: axum::body::Bytes| {
                let state = s.clone();
                async move {
                    routes::handle_signed_s3_upload_put(state, bucket_key, object_key, body).await
                }
            },
        ),
    );

    // Signed S3 download URL
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/signeds3download",
        HttpMethod::Get,
        get(
            move |Path((bucket_key, object_key)): Path<(String, String)>,
                  headers: axum::http::HeaderMap| {
                let state = s.clone();
                async move {
                    let host = headers
                        .get(axum::http::header::HOST)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("localhost:3000")
                        .to_string();
                    routes::handle_signed_s3_download(state, bucket_key, object_key, host).await
                }
            },
        ),
    );

    // Mock S3 download content endpoint
    let s = state.clone();
    router = add_route(
        router,
        "/oss/v2/buckets/:bucket_key/objects/:object_key/signeds3download/mock-s3",
        HttpMethod::Get,
        get(move |Path((bucket_key, object_key)): Path<(String, String)>| {
            let state = s.clone();
            async move {
                routes::handle_signed_s3_download_content(state, bucket_key, object_key).await
            }
        }),
    );

    // Data Management endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_list_hubs(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs/:hub_id",
        HttpMethod::Get,
        get(move |Path(hub_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_get_hub(state, hub_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/project/v1/hubs/:hub_id/projects",
        HttpMethod::Get,
        get(move |Path(hub_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_list_hub_projects(state, hub_id).await }
        }),
    );

    // Model Derivative endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/modelderivative/v2/designdata/job",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_create_translation(state, body).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/modelderivative/v2/designdata/:urn/manifest",
        HttpMethod::Get,
        get(move |Path(urn): Path<String>| {
            let state = s.clone();
            async move { routes::handle_get_manifest(state, urn).await }
        }),
    );

    // Construction/ACC Issues endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/construction/issues/v1/projects/:project_id/issues",
        HttpMethod::Get,
        get(move |Path(project_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_list_issues(state, project_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/construction/issues/v1/projects/:project_id/issues",
        HttpMethod::Post,
        post(
            move |Path(project_id): Path<String>, Json(body): Json<Value>| {
                let state = s.clone();
                async move { routes::handle_create_issue(state, project_id, body).await }
            },
        ),
    );

    // Webhooks endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/hooks",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_list_all_webhooks(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks",
        HttpMethod::Get,
        get(move |Path((system, _event)): Path<(String, String)>| {
            let state = s.clone();
            async move { routes::handle_list_webhooks(state, system).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks",
        HttpMethod::Post,
        post(
            move |Path((system, event)): Path<(String, String)>, Json(body): Json<Value>| {
                let state = s.clone();
                async move { routes::handle_create_webhook(state, system, event, body).await }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/webhooks/v1/systems/:system/events/:event/hooks/:hook_id",
        HttpMethod::Delete,
        delete(
            move |Path((_system, _event, hook_id)): Path<(String, String, String)>| {
                let state = s.clone();
                async move { routes::handle_delete_webhook(state, hook_id).await }
            },
        ),
    );

    // Design Automation endpoints (base: /da/us-east/v3)
    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/engines",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_da_list_engines(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/appbundles",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_da_list_appbundles(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/appbundles",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_da_create_appbundle(state, body).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/appbundles/:bundle_id",
        HttpMethod::Delete,
        delete(move |Path(bundle_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_da_delete_appbundle(state, bundle_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/appbundles/:bundle_id/aliases",
        HttpMethod::Post,
        post(
            move |Path(bundle_id): Path<String>, Json(body): Json<Value>| {
                let state = s.clone();
                async move {
                    routes::handle_da_create_appbundle_alias(state, bundle_id, body).await
                }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/activities",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_da_list_activities(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/activities",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_da_create_activity(state, body).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/activities/:activity_id",
        HttpMethod::Delete,
        delete(move |Path(activity_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_da_delete_activity(state, activity_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/activities/:activity_id/aliases",
        HttpMethod::Post,
        post(
            move |Path(activity_id): Path<String>, Json(body): Json<Value>| {
                let state = s.clone();
                async move {
                    routes::handle_da_create_activity_alias(state, activity_id, body).await
                }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/workitems",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_da_create_workitem(state, body).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/workitems",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_da_list_workitems(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/da/us-east/v3/workitems/:workitem_id",
        HttpMethod::Get,
        get(move |Path(workitem_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_da_get_workitem(state, workitem_id).await }
        }),
    );

    // Reality Capture endpoints (base: /photo-to-3d/v1)
    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene",
        HttpMethod::Get,
        get(move || {
            let state = s.clone();
            async move { routes::handle_reality_list_photoscenes(state).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene",
        HttpMethod::Post,
        post(
            move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                let state = s.clone();
                async move {
                    let content_type = headers
                        .get(axum::http::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    let parsed: Value =
                        if content_type.contains("application/x-www-form-urlencoded") {
                            let text = String::from_utf8_lossy(&body);
                            let mut map = serde_json::Map::new();
                            for pair in text.split('&') {
                                if let Some((k, v)) = pair.split_once('=') {
                                    let decoded =
                                        urlencoding::decode(v).unwrap_or_default().to_string();
                                    map.insert(k.to_string(), Value::String(decoded));
                                }
                            }
                            Value::Object(map)
                        } else {
                            serde_json::from_slice(&body).unwrap_or_default()
                        };
                    routes::handle_reality_create_photoscene(state, parsed).await
                }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/file",
        HttpMethod::Post,
        post(
            move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                let state = s.clone();
                async move {
                    let content_type = headers
                        .get(axum::http::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    let parsed: Value =
                        if content_type.contains("application/x-www-form-urlencoded") {
                            let text = String::from_utf8_lossy(&body);
                            let mut map = serde_json::Map::new();
                            for pair in text.split('&') {
                                if let Some((k, v)) = pair.split_once('=') {
                                    map.insert(k.to_string(), Value::String(v.to_string()));
                                }
                            }
                            Value::Object(map)
                        } else {
                            serde_json::from_slice(&body).unwrap_or_default()
                        };
                    routes::handle_reality_upload_file(state, parsed).await
                }
            },
        ),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene/:photoscene_id",
        HttpMethod::Post,
        post(move |Path(photoscene_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_reality_process_photoscene(state, photoscene_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene/:photoscene_id/progress",
        HttpMethod::Get,
        get(move |Path(photoscene_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_reality_get_progress(state, photoscene_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene/:photoscene_id",
        HttpMethod::Get,
        get(move |Path(photoscene_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_reality_get_result(state, photoscene_id).await }
        }),
    );

    let s = state.clone();
    router = add_route(
        router,
        "/photo-to-3d/v1/photoscene/:photoscene_id",
        HttpMethod::Delete,
        delete(move |Path(photoscene_id): Path<String>| {
            let state = s.clone();
            async move { routes::handle_reality_delete_photoscene(state, photoscene_id).await }
        }),
    );

    router
}

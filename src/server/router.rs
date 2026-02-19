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

    // Authentication endpoints
    let s = state.clone();
    router = add_route(
        router,
        "/authentication/v2/token",
        HttpMethod::Post,
        post(move |Json(body): Json<Value>| {
            let state = s.clone();
            async move { routes::handle_auth_token(state, body).await }
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
            move |Path((system, _event)): Path<(String, String)>, Json(body): Json<Value>| {
                let state = s.clone();
                async move { routes::handle_create_webhook(state, system, body).await }
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

    router
}

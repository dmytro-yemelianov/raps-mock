// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::StateManager;
use axum::{
    Extension,
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

/// Middleware to validate Bearer tokens
pub async fn auth_middleware(
    state: Option<Extension<StateManager>>,
    request: Request,
    next: Next,
) -> Response {
    // Skip auth for token endpoint
    if request.uri().path() == "/authentication/v2/token" {
        return next.run(request).await;
    }

    // Extract Bearer token
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    if let Some(token) = token {
        // Validate token against state if available
        if let Some(Extension(ref state_manager)) = state {
            if state_manager.auth.validate_token(token) {
                return next.run(request).await;
            }
            // Token validation failed
            return unauthorized_response("The access token provided is invalid or has expired.");
        }
        // No state manager (stateless mode) - accept any Bearer token
        return next.run(request).await;
    }

    // Return 401 if no valid token
    unauthorized_response("Missing or malformed Authorization header. Expected: Bearer <token>")
}

fn unauthorized_response(message: &str) -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "developerMessage": message,
                "errorCode": "AUTH-001"
            })
            .to_string()
            .into(),
        )
        // Response::builder() with valid status and headers cannot fail
        .expect("Failed to build unauthorized response")
}

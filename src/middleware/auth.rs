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
    // Skip auth for token endpoint, mock S3 signed URL endpoints, and mock S3 upload
    let path = request.uri().path();
    if path == "/authentication/v2/token"
        || path.ends_with("/mock-s3")
        || path.starts_with("/mock-s3-upload/")
    {
        return next.run(request).await;
    }

    // Extract Bearer token
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    println!("DEBUG: auth_middleware path={} token={:?}", path, token);
    eprintln!("DEBUG: auth_middleware path={} token={:?}", path, token);

    if let Some(token) = token {
        // Validate token against state if available
        if let Some(Extension(ref state_manager)) = state {
            tracing::info!(token, "Validating token against state");
            if state_manager.auth.validate_token(token).unwrap_or(false) {
                return next.run(request).await;
            }
            // Token validation failed
            tracing::warn!(token, "Token validation failed in middleware");
            return unauthorized_response("The access token provided is invalid or has expired.");
        }
        // No state manager (stateless mode) - accept any Bearer token
        tracing::info!(token, "Stateless mode, accepting token");
        return next.run(request).await;
    }

    tracing::warn!(path, "Missing or malformed Authorization header");
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

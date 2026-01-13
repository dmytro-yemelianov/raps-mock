// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::openapi::types::RouteDefinition;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Generic handler that serves mock responses based on OpenAPI definitions
pub struct GenericHandler {
    route: RouteDefinition,
}

impl GenericHandler {
    pub fn new(route: RouteDefinition) -> Self {
        Self { route }
    }

    pub async fn handle(&self) -> Response {
        tracing::info!(
            "GenericHandler handling {} {}",
            self.route.method.as_str(),
            self.route.path
        );
        // Try to find a successful response (200, 201, etc.)
        let success_codes = ["200", "201", "202", "204", "default"];

        for code in success_codes {
            if let Some(response) = self.route.operation.responses.get(code) {
                // Resolve reference if needed
                let response_def = self.resolve_response(response);

                if let Some(crate::openapi::types::Response::Definition {
                    content: Some(content_map),
                    ..
                }) = response_def
                {
                    // Media types to check in order of priority
                    let media_types = ["application/json", "application/vnd.api+json"];

                    for mt in &media_types {
                        if let Some(example) = content_map
                            .get(*mt)
                            .and_then(|media_type| self.extract_example(media_type))
                        {
                            return (StatusCode::OK, Json(example)).into_response();
                        }
                    }
                }

                if response_def.is_some() {
                    // If it's 204 No Content, return empty body
                    if code == "204" {
                        return StatusCode::NO_CONTENT.into_response();
                    }

                    // Fallback for success without content
                    return StatusCode::OK.into_response();
                }
            }
        }

        // Fallback if no success response defined
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({
                "message": format!("No example response available for {} {}", self.route.method.as_str(), self.route.path),
                "operation_id": self.route.operation.operation_id
            })),
        )
            .into_response()
    }

    fn resolve_response<'a>(
        &'a self,
        response: &'a crate::openapi::types::Response,
    ) -> Option<&'a crate::openapi::types::Response> {
        match response {
            crate::openapi::types::Response::Definition { .. } => Some(response),
            crate::openapi::types::Response::Ref { ref_path } => {
                let name = ref_path.split('/').next_back()?;
                self.route
                    .components
                    .as_ref()?
                    .responses
                    .as_ref()?
                    .get(name)
            }
        }
    }

    fn extract_example(
        &self,
        media_type: &crate::openapi::types::MediaType,
    ) -> Option<serde_json::Value> {
        // 1. Try direct example
        if let Some(ref example) = media_type.example {
            return Some(example.clone());
        }

        // 2. Try first example from examples map
        if let Some(value) = media_type
            .examples
            .as_ref()
            .and_then(|examples| examples.values().next())
            .and_then(|example| example.value.as_ref())
        {
            return Some(value.clone());
        }

        // 3. Try example from schema
        media_type.schema.as_ref().and_then(|schema| {
            if let Some(crate::openapi::types::Schema::Object {
                example: Some(ex), ..
            }) = self.resolve_schema(schema)
            {
                Some(ex.clone())
            } else {
                None
            }
        })
    }

    fn resolve_schema<'a>(
        &'a self,
        schema: &'a crate::openapi::types::Schema,
    ) -> Option<&'a crate::openapi::types::Schema> {
        match schema {
            crate::openapi::types::Schema::Ref { ref_path } => {
                let name = ref_path.split('/').next_back()?;
                self.route.components.as_ref()?.schemas.as_ref()?.get(name)
            }
            _ => Some(schema),
        }
    }
}

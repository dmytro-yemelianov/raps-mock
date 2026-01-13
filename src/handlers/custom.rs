// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use axum::response::Response;
use dashmap::DashMap;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Custom handler function type
pub type HandlerFn =
    Arc<dyn Fn(Option<Value>) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// Registry for custom handlers
pub struct CustomHandlerRegistry {
    handlers: DashMap<String, HandlerFn>,
}

impl CustomHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }

    /// Register a custom handler for a route
    pub fn register(&self, route_key: String, handler: HandlerFn) {
        self.handlers.insert(route_key, handler);
    }

    /// Check if a handler exists for a route
    pub fn has(&self, route_key: &str) -> bool {
        self.handlers.contains_key(route_key)
    }

    /// Get a handler for a route
    pub fn get(&self, route_key: &str) -> Option<HandlerFn> {
        self.handlers.get(route_key).map(|h| h.clone())
    }
}

impl Default for CustomHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

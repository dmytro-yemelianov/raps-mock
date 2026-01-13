// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Webhook subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub hook_id: String,
    pub tenant: String,
    pub callback_url: String,
    pub scope: WebhookScope,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookScope {
    pub folder: Option<String>,
    pub project: Option<String>,
}

/// Webhooks state
pub struct WebhooksState {
    subscriptions: DashMap<String, WebhookSubscription>,
}

impl WebhooksState {
    pub fn new() -> Self {
        Self {
            subscriptions: DashMap::new(),
        }
    }

    /// Create a webhook subscription
    pub fn create_subscription(
        &self,
        tenant: String,
        callback_url: String,
        scope: WebhookScope,
    ) -> WebhookSubscription {
        let hook_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        let subscription = WebhookSubscription {
            hook_id: hook_id.clone(),
            tenant,
            callback_url,
            scope,
            status: "active".to_string(),
            created_at: now,
        };

        self.subscriptions.insert(hook_id, subscription.clone());
        subscription
    }

    /// Get a subscription
    pub fn get_subscription(&self, hook_id: &str) -> Option<WebhookSubscription> {
        self.subscriptions.get(hook_id).map(|s| s.clone())
    }

    /// List all subscriptions
    pub fn list_subscriptions(&self) -> Vec<WebhookSubscription> {
        self.subscriptions
            .iter()
            .map(|s| s.value().clone())
            .collect()
    }

    /// Delete a subscription
    pub fn delete_subscription(&self, hook_id: &str) -> bool {
        self.subscriptions.remove(hook_id).is_some()
    }
}

impl Default for WebhooksState {
    fn default() -> Self {
        Self::new()
    }
}

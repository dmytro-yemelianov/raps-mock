// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Webhook subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookSubscription {
    pub hook_id: String,
    pub tenant: String,
    pub callback_url: String,
    pub event: String,
    pub system: String,
    pub scope: WebhookScope,
    pub status: String,
    pub created_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookScope {
    pub folder: Option<String>,
    pub workflow: Option<String>,
}

/// Webhooks state
pub struct WebhooksState {
    db: Arc<Db>,
}

impl WebhooksState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Create a webhook subscription
    pub fn create_subscription(
        &self,
        tenant: String,
        callback_url: String,
        event: String,
        system: String,
        scope: WebhookScope,
    ) -> crate::error::Result<WebhookSubscription> {
        let hook_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let subscription = WebhookSubscription {
            hook_id: hook_id.clone(),
            tenant,
            callback_url,
            event,
            system,
            scope,
            status: "active".to_string(),
            created_date: now,
        };

        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO webhooks (hook_id, tenant, callback_url, event, system, scope_folder, scope_workflow, status, created_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                subscription.hook_id,
                subscription.tenant,
                subscription.callback_url,
                subscription.event,
                subscription.system,
                subscription.scope.folder,
                subscription.scope.workflow,
                subscription.status,
                subscription.created_date,
            ],
        )?;
        Ok(subscription)
    }

    /// Get a subscription
    pub fn get_subscription(&self, hook_id: &str) -> crate::error::Result<Option<WebhookSubscription>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT hook_id, tenant, callback_url, event, system, scope_folder, scope_workflow, status, created_date
             FROM webhooks WHERE hook_id = ?1",
            rusqlite::params![hook_id],
            Self::row_to_subscription,
        )
        .optional()?)
    }

    /// List all subscriptions
    pub fn list_subscriptions(&self) -> crate::error::Result<Vec<WebhookSubscription>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT hook_id, tenant, callback_url, event, system, scope_folder, scope_workflow, status, created_date FROM webhooks",
            )?;
        let items = stmt.query_map([], Self::row_to_subscription)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Delete a subscription
    pub fn delete_subscription(&self, hook_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn
            .execute(
                "DELETE FROM webhooks WHERE hook_id = ?1",
                rusqlite::params![hook_id],
            )?;
        Ok(rows > 0)
    }

    fn row_to_subscription(row: &rusqlite::Row<'_>) -> rusqlite::Result<WebhookSubscription> {
        Ok(WebhookSubscription {
            hook_id: row.get(0)?,
            tenant: row.get(1)?,
            callback_url: row.get(2)?,
            event: row.get(3)?,
            system: row.get(4)?,
            scope: WebhookScope {
                folder: row.get(5)?,
                workflow: row.get(6)?,
            },
            status: row.get(7)?,
            created_date: row.get(8)?,
        })
    }
}

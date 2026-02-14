// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::error::{MockError, Result};
use crate::state::{auth, buckets, issues, objects, projects, translations, webhooks};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Serializable snapshot of all state for persistence
#[derive(Serialize, Deserialize)]
struct StateSnapshot {
    buckets: Vec<buckets::BucketInfo>,
    objects: Vec<objects::ObjectInfo>,
    projects: StateProjectSnapshot,
    translations: Vec<translations::TranslationJob>,
    issues: Vec<issues::IssueInfo>,
    webhooks: Vec<webhooks::WebhookSubscription>,
}

#[derive(Serialize, Deserialize)]
struct StateProjectSnapshot {
    hubs: Vec<projects::HubInfo>,
    projects: Vec<projects::ProjectInfo>,
}

/// Central state manager for all APS resources
#[derive(Clone)]
pub struct StateManager {
    /// OAuth tokens storage
    pub auth: Arc<auth::AuthState>,
    /// OSS buckets storage
    pub buckets: Arc<buckets::BucketState>,
    /// OSS objects storage
    pub objects: Arc<objects::ObjectState>,
    /// Data Management projects storage
    pub projects: Arc<projects::ProjectState>,
    /// Model Derivative translations storage
    pub translations: Arc<translations::TranslationState>,
    /// ACC Issues storage
    pub issues: Arc<issues::IssuesState>,
    /// Webhooks storage
    pub webhooks: Arc<webhooks::WebhooksState>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            auth: Arc::new(auth::AuthState::new()),
            buckets: Arc::new(buckets::BucketState::new()),
            objects: Arc::new(objects::ObjectState::new()),
            projects: Arc::new(projects::ProjectState::new()),
            translations: Arc::new(translations::TranslationState::new()),
            issues: Arc::new(issues::IssuesState::new()),
            webhooks: Arc::new(webhooks::WebhooksState::new()),
        }
    }

    /// Load state from a file (if provided)
    pub fn load_from_file(&self, path: &std::path::Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            MockError::StatePersistence(format!("Failed to read state file: {}", e))
        })?;
        let snapshot: StateSnapshot = serde_json::from_str(&content).map_err(|e| {
            MockError::StatePersistence(format!("Failed to parse state file: {}", e))
        })?;

        // Restore buckets
        for bucket in snapshot.buckets {
            self.buckets.restore(bucket);
        }
        // Restore objects
        for object in snapshot.objects {
            self.objects.restore(object);
        }
        // Restore projects (hubs + projects)
        for hub in snapshot.projects.hubs {
            self.projects.restore_hub(hub);
        }
        for project in snapshot.projects.projects {
            self.projects.restore_project(project);
        }
        // Restore translations
        for job in snapshot.translations {
            self.translations.restore(job);
        }
        // Restore issues
        for issue in snapshot.issues {
            self.issues.restore(issue);
        }
        // Restore webhooks
        for sub in snapshot.webhooks {
            self.webhooks.restore(sub);
        }

        tracing::info!("State loaded from {}", path.display());
        Ok(())
    }

    /// Save state to a file (if provided)
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let snapshot = StateSnapshot {
            buckets: self.buckets.list_buckets(),
            objects: self.objects.list_all(),
            projects: StateProjectSnapshot {
                hubs: self.projects.list_hubs(),
                projects: self.projects.list_all_projects(),
            },
            translations: self.translations.list_all(),
            issues: self.issues.list_all(),
            webhooks: self.webhooks.list_subscriptions(),
        };
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| {
            MockError::StatePersistence(format!("Failed to serialize state: {}", e))
        })?;
        std::fs::write(path, json).map_err(|e| {
            MockError::StatePersistence(format!("Failed to write state file: {}", e))
        })?;
        tracing::info!("State saved to {}", path.display());
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::error::Result;
use crate::state::{auth, buckets, issues, objects, projects, translations, webhooks};
use std::sync::Arc;

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
    pub fn load_from_file(&self, _path: &std::path::Path) -> Result<()> {
        // TODO: Implement state persistence
        Ok(())
    }

    /// Save state to a file (if provided)
    pub fn save_to_file(&self, _path: &std::path::Path) -> Result<()> {
        // TODO: Implement state persistence
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

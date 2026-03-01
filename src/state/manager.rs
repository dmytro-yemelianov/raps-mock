// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::{
    acc, admin, auth, buckets, da, db, folders, issues, items, objects, projects, reality,
    translations, webhooks,
};
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
    /// Design Automation storage
    pub da: Arc<da::DaState>,
    /// Reality Capture storage
    pub reality: Arc<reality::RealityState>,
    /// ACC (RFIs, Assets, Submittals, Checklists) storage
    pub acc: Arc<acc::AccState>,
    /// Data Management folders storage
    pub folders: Arc<folders::FolderState>,
    /// Data Management items storage
    pub items: Arc<items::ItemState>,
    /// Admin (users, projects, companies, jobs) storage
    pub admin: Arc<admin::AdminState>,
}

impl StateManager {
    /// Create a new state manager with in-memory SQLite (default).
    pub fn new() -> Self {
        let db = Arc::new(db::Db::open_in_memory());
        Self::from_db(db)
    }

    /// Create a new state manager backed by a file-based SQLite database.
    pub fn with_db(path: &std::path::Path) -> Self {
        let db = Arc::new(db::Db::open_file(path));
        Self::from_db(db)
    }

    fn from_db(db: Arc<db::Db>) -> Self {
        Self {
            auth: Arc::new(auth::AuthState::new(db.clone())),
            buckets: Arc::new(buckets::BucketState::new(db.clone())),
            objects: Arc::new(objects::ObjectState::new(db.clone())),
            projects: Arc::new(projects::ProjectState::new(db.clone())),
            translations: Arc::new(translations::TranslationState::new(db.clone())),
            issues: Arc::new(issues::IssuesState::new(db.clone())),
            webhooks: Arc::new(webhooks::WebhooksState::new(db.clone())),
            da: Arc::new(da::DaState::new(db.clone())),
            reality: Arc::new(reality::RealityState::new(db.clone())),
            acc: Arc::new(acc::AccState::new(db.clone())),
            folders: Arc::new(folders::FolderState::new(db.clone())),
            items: Arc::new(items::ItemState::new(db.clone())),
            admin: Arc::new(admin::AdminState::new(db)),
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

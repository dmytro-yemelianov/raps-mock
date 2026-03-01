// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Fixture loading for pre-populating mock server state.
//!
//! Fixtures are JSON files that describe initial state for the mock server.
//! They can be loaded on startup via `--fixtures <path>` to create realistic
//! test scenarios without manual API calls.
//!
//! # Fixture Format
//!
//! ```json
//! {
//!   "buckets": [
//!     { "bucket_key": "my-bucket", "policy_key": "persistent" }
//!   ],
//!   "objects": [
//!     { "bucket_key": "my-bucket", "object_key": "model.rvt", "size": 1024000 }
//!   ],
//!   "projects": [
//!     { "name": "Test Project", "type": "ACC" }
//!   ],
//!   "issues": [
//!     { "project_id": "project-001", "title": "Crack in wall", "status": "open" }
//!   ]
//! }
//! ```

use crate::state::StateManager;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Top-level fixture file structure
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct FixtureData {
    /// OSS buckets to create
    pub buckets: Vec<BucketFixture>,
    /// OSS objects (metadata only — no actual file data)
    pub objects: Vec<ObjectFixture>,
    /// Data Management projects
    pub projects: Vec<ProjectFixture>,
    /// ACC issues
    pub issues: Vec<IssueFixture>,
    /// Webhooks
    pub webhooks: Vec<WebhookFixture>,
}

#[derive(Debug, Deserialize)]
pub struct BucketFixture {
    pub bucket_key: String,
    #[serde(default = "default_policy")]
    pub policy_key: String,
}

fn default_policy() -> String {
    "persistent".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ObjectFixture {
    pub bucket_key: String,
    pub object_key: String,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default = "default_content_type")]
    pub content_type: String,
}

fn default_size() -> u64 {
    1024
}
fn default_content_type() -> String {
    "application/octet-stream".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ProjectFixture {
    #[serde(default = "default_project_name")]
    pub name: String,
    #[serde(default = "default_project_type")]
    pub project_type: String,
}

fn default_project_name() -> String {
    "Test Project".to_string()
}
fn default_project_type() -> String {
    "ACC".to_string()
}

#[derive(Debug, Deserialize)]
pub struct IssueFixture {
    pub project_id: String,
    pub title: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Deserialize)]
pub struct WebhookFixture {
    #[serde(default = "default_system")]
    pub system: String,
    #[serde(default = "default_event")]
    pub event: String,
    pub callback_url: String,
}

fn default_system() -> String {
    "data".to_string()
}
fn default_event() -> String {
    "dm.version.added".to_string()
}

/// Load fixtures from a JSON file and apply them to the state manager.
pub fn load_fixtures(path: &Path, state: &StateManager) -> Result<FixtureStats> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read fixture file: {}", path.display()))?;

    let data: FixtureData =
        serde_json::from_str(&content).with_context(|| format!("Failed to parse fixture file: {}", path.display()))?;

    let mut stats = FixtureStats::default();

    // Load buckets
    for bucket in &data.buckets {
        state
            .buckets
            .create_bucket(bucket.bucket_key.clone(), bucket.policy_key.clone())?;
        stats.buckets += 1;
    }

    // Load objects
    for obj in &data.objects {
        state.objects.upload_object(
            obj.bucket_key.clone(),
            obj.object_key.clone(),
            obj.size,
            Some(obj.content_type.clone()),
        )?;
        stats.objects += 1;
    }

    // Load projects — no direct create API on ProjectState (pre-seeded in DB init).
    // Projects fixture support will be added when ProjectState gets a create method.
    stats.projects = data.projects.len();

    // Load issues
    for issue in &data.issues {
        state.issues.create_issue(
            issue.project_id.clone(),
            issue.title.clone(),
            issue.description.clone(),
        )?;
        stats.issues += 1;
    }

    // Load webhooks
    for webhook in &data.webhooks {
        state.webhooks.create_subscription(
            "mock-tenant".to_string(),
            webhook.callback_url.clone(),
            webhook.event.clone(),
            webhook.system.clone(),
            crate::state::webhooks::WebhookScope {
                folder: None,
                workflow: None,
            },
        )?;
        stats.webhooks += 1;
    }

    Ok(stats)
}

/// Load all fixture files from a directory (*.json).
pub fn load_fixtures_dir(dir: &Path, state: &StateManager) -> Result<FixtureStats> {
    let mut total = FixtureStats::default();

    let entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read fixtures directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .collect();

    if entries.is_empty() {
        tracing::warn!("No .json fixture files found in {}", dir.display());
        return Ok(total);
    }

    for entry in entries {
        let path = entry.path();
        tracing::info!("Loading fixture: {}", path.display());
        match load_fixtures(&path, state) {
            Ok(stats) => {
                total = total.merge(stats);
            }
            Err(e) => {
                tracing::error!("Failed to load fixture {}: {}", path.display(), e);
            }
        }
    }

    Ok(total)
}

/// Statistics about loaded fixtures
#[derive(Debug, Default)]
pub struct FixtureStats {
    pub buckets: usize,
    pub objects: usize,
    pub projects: usize,
    pub issues: usize,
    pub webhooks: usize,
}

impl FixtureStats {
    pub fn total(&self) -> usize {
        self.buckets + self.objects + self.projects + self.issues + self.webhooks
    }

    fn merge(self, other: FixtureStats) -> FixtureStats {
        FixtureStats {
            buckets: self.buckets + other.buckets,
            objects: self.objects + other.objects,
            projects: self.projects + other.projects,
            issues: self.issues + other.issues,
            webhooks: self.webhooks + other.webhooks,
        }
    }
}

impl std::fmt::Display for FixtureStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} buckets, {} objects, {} projects, {} issues, {} webhooks ({} total)",
            self.buckets,
            self.objects,
            self.projects,
            self.issues,
            self.webhooks,
            self.total()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture_minimal() {
        let json = r#"{"buckets": [{"bucket_key": "test"}]}"#;
        let data: FixtureData = serde_json::from_str(json).unwrap();
        assert_eq!(data.buckets.len(), 1);
        assert_eq!(data.buckets[0].bucket_key, "test");
        assert_eq!(data.buckets[0].policy_key, "persistent");
    }

    #[test]
    fn test_parse_fixture_empty() {
        let json = "{}";
        let data: FixtureData = serde_json::from_str(json).unwrap();
        assert!(data.buckets.is_empty());
        assert!(data.objects.is_empty());
        assert!(data.projects.is_empty());
    }

    #[test]
    fn test_parse_fixture_full() {
        let json = r#"{
            "buckets": [
                {"bucket_key": "b1", "policy_key": "transient"},
                {"bucket_key": "b2"}
            ],
            "objects": [
                {"bucket_key": "b1", "object_key": "model.rvt", "size": 5000000}
            ],
            "issues": [
                {"project_id": "p1", "title": "Test issue", "status": "open", "description": "A test"}
            ]
        }"#;
        let data: FixtureData = serde_json::from_str(json).unwrap();
        assert_eq!(data.buckets.len(), 2);
        assert_eq!(data.buckets[0].policy_key, "transient");
        assert_eq!(data.buckets[1].policy_key, "persistent");
        assert_eq!(data.objects.len(), 1);
        assert_eq!(data.objects[0].size, 5000000);
        assert_eq!(data.issues.len(), 1);
    }

    #[test]
    fn test_fixture_stats_display() {
        let stats = FixtureStats {
            buckets: 3,
            objects: 10,
            projects: 2,
            issues: 5,
            webhooks: 1,
        };
        let s = stats.to_string();
        assert!(s.contains("3 buckets"));
        assert!(s.contains("21 total"));
    }
}

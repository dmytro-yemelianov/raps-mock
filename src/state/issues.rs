// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// ACC Issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: i64,
}

/// ACC Issues state
pub struct IssuesState {
    /// Map of project_id -> issues
    issues: DashMap<String, DashMap<String, IssueInfo>>,
}

impl IssuesState {
    pub fn new() -> Self {
        Self {
            issues: DashMap::new(),
        }
    }

    /// Create a new issue
    pub fn create_issue(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> IssueInfo {
        let issue_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        let issue = IssueInfo {
            id: issue_id.clone(),
            project_id: project_id.clone(),
            title,
            description,
            status: "open".to_string(),
            created_at: now,
        };

        let project_issues = self.issues.entry(project_id).or_default();
        project_issues.insert(issue_id, issue.clone());
        issue
    }

    /// Get an issue
    pub fn get_issue(&self, project_id: &str, issue_id: &str) -> Option<IssueInfo> {
        self.issues
            .get(project_id)?
            .get(issue_id)
            .map(|i| i.clone())
    }

    /// List issues for a project
    pub fn list_issues(&self, project_id: &str) -> Vec<IssueInfo> {
        self.issues
            .get(project_id)
            .map(|project_issues| project_issues.iter().map(|i| i.value().clone()).collect())
            .unwrap_or_default()
    }

    /// Update issue status
    pub fn update_issue_status(&self, project_id: &str, issue_id: &str, status: String) -> bool {
        self.issues
            .get(project_id)
            .and_then(|project_issues| {
                project_issues.get_mut(issue_id).map(|mut issue| {
                    issue.status = status;
                    true
                })
            })
            .unwrap_or(false)
    }
}

impl Default for IssuesState {
    fn default() -> Self {
        Self::new()
    }
}

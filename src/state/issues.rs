// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// ACC Issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// ACC Issue comment information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentInfo {
    pub id: String,
    pub issue_id: String,
    pub body: String,
    pub created_at: String,
}

/// ACC Issues state
pub struct IssuesState {
    /// Map of project_id -> issues
    issues: DashMap<String, DashMap<String, IssueInfo>>,
    /// Map of issue_id -> comments
    comments: DashMap<String, DashMap<String, CommentInfo>>,
}

impl IssuesState {
    pub fn new() -> Self {
        let state = Self {
            issues: DashMap::new(),
            comments: DashMap::new(),
        };
        // Pre-seed well-known demo issues used by raps-examples tests
        let demo_project = "mock-project-001";
        let now = chrono::Utc::now().to_rfc3339();
        let demo_issues = vec![
            ("8d5b8b2c-3a1e-467c-9f1b-6c2d9a8e1f5b", "Demo Issue - Structural"),
            ("issue-a-demo-001", "Issue A"),
            ("issue-b-demo-002", "Issue B"),
            ("issue-c-demo-003", "Issue C"),
            ("cmt-demo-001", "Demo Issue for Comments"),
        ];
        {
            let project_issues = state.issues.entry(demo_project.to_string()).or_default();
            for (id, title) in demo_issues {
                project_issues.insert(
                    id.to_string(),
                    IssueInfo {
                        id: id.to_string(),
                        project_id: demo_project.to_string(),
                        title: title.to_string(),
                        description: None,
                        status: "open".to_string(),
                        created_at: now.clone(),
                    },
                );
            }
        }
        state
    }

    /// Create a new issue
    pub fn create_issue(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> IssueInfo {
        let issue_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
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

    /// Restore an issue from a persistence snapshot
    pub fn restore(&self, issue: IssueInfo) {
        let project_issues = self.issues.entry(issue.project_id.clone()).or_default();
        project_issues.insert(issue.id.clone(), issue);
    }

    /// List all issues across all projects
    pub fn list_all(&self) -> Vec<IssueInfo> {
        self.issues
            .iter()
            .flat_map(|proj| {
                proj.value()
                    .iter()
                    .map(|i| i.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
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

    /// Update an issue with optional fields
    pub fn update_issue(
        &self,
        project_id: &str,
        issue_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Option<IssueInfo> {
        let project_issues = self.issues.get(project_id)?;
        let mut issue = project_issues.get_mut(issue_id)?;
        if let Some(t) = title {
            issue.title = t;
        }
        if let Some(d) = description {
            issue.description = Some(d);
        }
        if let Some(s) = status {
            issue.status = s;
        }
        Some(issue.clone())
    }

    /// Delete an issue
    pub fn delete_issue(&self, project_id: &str, issue_id: &str) -> bool {
        self.issues
            .get(project_id)
            .map(|project_issues| project_issues.remove(issue_id).is_some())
            .unwrap_or(false)
    }

    /// Add a comment to an issue
    pub fn add_comment(
        &self,
        project_id: &str,
        issue_id: &str,
        body: String,
    ) -> Option<CommentInfo> {
        // Verify the issue exists
        if self.get_issue(project_id, issue_id).is_none() {
            return None;
        }

        let comment_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let comment = CommentInfo {
            id: comment_id.clone(),
            issue_id: issue_id.to_string(),
            body,
            created_at: now,
        };

        let issue_comments = self.comments.entry(issue_id.to_string()).or_default();
        issue_comments.insert(comment_id, comment.clone());
        Some(comment)
    }

    /// List comments for an issue
    pub fn list_comments(&self, _project_id: &str, issue_id: &str) -> Vec<CommentInfo> {
        self.comments
            .get(issue_id)
            .map(|issue_comments| issue_comments.iter().map(|c| c.value().clone()).collect())
            .unwrap_or_default()
    }

    /// Delete a comment from an issue
    pub fn delete_comment(&self, _project_id: &str, issue_id: &str, comment_id: &str) -> bool {
        self.comments
            .get(issue_id)
            .map(|issue_comments| issue_comments.remove(comment_id).is_some())
            .unwrap_or(false)
    }
}

impl Default for IssuesState {
    fn default() -> Self {
        Self::new()
    }
}

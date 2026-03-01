// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    db: Arc<Db>,
}

impl IssuesState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let demo_project = "mock-project-001";
        let now = chrono::Utc::now().to_rfc3339();
        let demo_issues = [
            (
                "8d5b8b2c-3a1e-467c-9f1b-6c2d9a8e1f5b",
                "Demo Issue - Structural",
            ),
            ("issue-a-demo-001", "Issue A"),
            ("issue-b-demo-002", "Issue B"),
            ("issue-c-demo-003", "Issue C"),
            ("cmt-demo-001", "Demo Issue for Comments"),
            ("demo-issue-001", "Demo Issue - General"),
            ("lc-issue-001", "Lifecycle Issue"),
        ];
        let conn = self.db.conn();
        for (id, title) in demo_issues {
            conn.execute(
                "INSERT OR IGNORE INTO issues (id, project_id, title, description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, demo_project, title, Option::<String>::None, "open", now],
            )
            .expect("failed to seed issue");
        }
    }

    /// Create a new issue
    pub fn create_issue(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> crate::error::Result<IssueInfo> {
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
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO issues (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                issue.id,
                issue.project_id,
                issue.title,
                issue.description,
                issue.status,
                issue.created_at,
            ],
        )?;
        Ok(issue)
    }

    /// Get an issue
    pub fn get_issue(&self, project_id: &str, issue_id: &str) -> crate::error::Result<Option<IssueInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, project_id, title, description, status, created_at
             FROM issues WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![issue_id, project_id],
            Self::row_to_issue,
        )
        .optional()?)
    }

    /// List issues for a project
    pub fn list_issues(&self, project_id: &str) -> crate::error::Result<Vec<IssueInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, status, created_at
                 FROM issues WHERE project_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_issue)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// List all issues across all projects
    pub fn list_all(&self) -> crate::error::Result<Vec<IssueInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, project_id, title, description, status, created_at FROM issues")?;
        let items = stmt.query_map([], Self::row_to_issue)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Update issue status
    pub fn update_issue_status(&self, project_id: &str, issue_id: &str, status: String) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "UPDATE issues SET status = ?1 WHERE id = ?2 AND project_id = ?3",
            rusqlite::params![status, issue_id, project_id],
        )?;
        Ok(rows > 0)
    }

    /// Update an issue with optional fields
    pub fn update_issue(
        &self,
        project_id: &str,
        issue_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<IssueInfo>> {
        let conn = self.db.conn();
        // Read current, apply changes, write back
        let current = match conn
            .query_row(
                "SELECT id, project_id, title, description, status, created_at
                 FROM issues WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![issue_id, project_id],
                Self::row_to_issue,
            )
            .optional()?
        {
            Some(c) => c,
            None => return Ok(None),
        };

        let new_title = title.unwrap_or(current.title);
        let new_desc = description.or(current.description);
        let new_status = status.unwrap_or(current.status);

        conn.execute(
            "UPDATE issues SET title = ?1, description = ?2, status = ?3 WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_title, new_desc, new_status, issue_id, project_id],
        )?;

        Ok(Some(IssueInfo {
            id: current.id,
            project_id: current.project_id,
            title: new_title,
            description: new_desc,
            status: new_status,
            created_at: current.created_at,
        }))
    }

    /// Delete an issue
    pub fn delete_issue(&self, project_id: &str, issue_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM issues WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![issue_id, project_id],
        )?;
        Ok(rows > 0)
    }

    /// Add a comment to an issue
    pub fn add_comment(
        &self,
        project_id: &str,
        issue_id: &str,
        body: String,
    ) -> crate::error::Result<Option<CommentInfo>> {
        // Verify the issue exists
        if self.get_issue(project_id, issue_id)?.is_none() {
            return Ok(None);
        }

        let comment_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let comment = CommentInfo {
            id: comment_id.clone(),
            issue_id: issue_id.to_string(),
            body,
            created_at: now,
        };

        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO issue_comments (id, issue_id, body, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                comment.id,
                comment.issue_id,
                comment.body,
                comment.created_at
            ],
        )?;
        Ok(Some(comment))
    }

    /// List comments for an issue
    pub fn list_comments(&self, _project_id: &str, issue_id: &str) -> crate::error::Result<Vec<CommentInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, issue_id, body, created_at FROM issue_comments WHERE issue_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![issue_id], |row| {
            Ok(CommentInfo {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                body: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Delete a comment from an issue
    pub fn delete_comment(&self, _project_id: &str, issue_id: &str, comment_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM issue_comments WHERE id = ?1 AND issue_id = ?2",
            rusqlite::params![comment_id, issue_id],
        )?;
        Ok(rows > 0)
    }

    fn row_to_issue(row: &rusqlite::Row<'_>) -> rusqlite::Result<IssueInfo> {
        Ok(IssueInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

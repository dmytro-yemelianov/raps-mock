// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Folder information (Data Management API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    pub id: String,
    pub project_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub display_name: String,
    pub created_at: String,
    pub last_modified_time: String,
}

/// Folder permission entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPermission {
    pub id: String,
    pub folder_id: String,
    pub project_id: String,
    pub subject_id: String,
    pub subject_type: String,
    pub actions: Vec<String>,
}

/// Folder state backed by SQLite
pub struct FolderState {
    db: Arc<Db>,
}

impl FolderState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.db.conn();

        // Top-level folders for the default project
        for (id, name) in [
            ("mock-top-folder-001", "Project Files"),
            ("mock-top-folder-002", "Plans"),
        ] {
            conn.execute(
                "INSERT OR IGNORE INTO folders (id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, "b.default-project", name, name, &now, &now],
            )
            .expect("failed to seed folder");
        }

        // Seed default folder permission
        conn.execute(
            "INSERT OR IGNORE INTO folder_permissions (id, folder_id, project_id, subject_id, subject_type, actions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "perm-001",
                "mock-top-folder-001",
                "b.default-project",
                "user-001",
                "user",
                r#"["view","download","collaborate"]"#,
            ],
        )
        .expect("failed to seed folder permission");

        // A subfolder inside "Project Files"
        conn.execute(
            "INSERT OR IGNORE INTO folders (id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "mock-subfolder-001",
                "b.default-project",
                "mock-top-folder-001",
                "Subfolder",
                "Subfolder",
                &now,
                &now,
            ],
        )
        .expect("failed to seed subfolder");
    }

    /// List top-level folders for a project (parent_folder_id IS NULL)
    pub fn list_top_folders(&self, project_id: &str) -> crate::error::Result<Vec<FolderInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time
             FROM folders WHERE project_id = ?1 AND parent_folder_id IS NULL",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], Self::row_to_folder)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// List child folders inside a given folder
    pub fn list_child_folders(
        &self,
        project_id: &str,
        parent_folder_id: &str,
    ) -> crate::error::Result<Vec<FolderInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time
             FROM folders WHERE project_id = ?1 AND parent_folder_id = ?2",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![project_id, parent_folder_id],
            Self::row_to_folder,
        )?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get a folder by ID
    pub fn get_folder(
        &self,
        project_id: &str,
        folder_id: &str,
    ) -> crate::error::Result<Option<FolderInfo>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time
                 FROM folders WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![folder_id, project_id],
                Self::row_to_folder,
            )
            .optional()?)
    }

    /// Create a folder
    pub fn create_folder(
        &self,
        project_id: String,
        parent_folder_id: Option<String>,
        name: String,
    ) -> crate::error::Result<FolderInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let folder = FolderInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            parent_folder_id,
            name: name.clone(),
            display_name: name,
            created_at: now.clone(),
            last_modified_time: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO folders (id, project_id, parent_folder_id, name, display_name, created_at, last_modified_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                folder.id,
                folder.project_id,
                folder.parent_folder_id,
                folder.name,
                folder.display_name,
                folder.created_at,
                folder.last_modified_time,
            ],
        )?;
        Ok(folder)
    }

    /// Update a folder's name
    pub fn update_folder(
        &self,
        project_id: &str,
        folder_id: &str,
        name: Option<String>,
    ) -> crate::error::Result<Option<FolderInfo>> {
        let current = match self.get_folder(project_id, folder_id)? {
            Some(f) => f,
            None => return Ok(None),
        };
        let new_name = name.unwrap_or(current.name);
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.db.conn();
        conn.execute(
            "UPDATE folders SET name = ?1, display_name = ?2, last_modified_time = ?3
             WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_name, new_name, now, folder_id, project_id],
        )?;
        Ok(Some(FolderInfo {
            id: current.id,
            project_id: current.project_id,
            parent_folder_id: current.parent_folder_id,
            name: new_name.clone(),
            display_name: new_name,
            created_at: current.created_at,
            last_modified_time: now,
        }))
    }

    /// Delete a folder
    pub fn delete_folder(&self, project_id: &str, folder_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM folders WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![folder_id, project_id],
        )?;
        Ok(rows > 0)
    }

    // ---- Permissions ----

    pub fn get_permissions(
        &self,
        project_id: &str,
        folder_id: &str,
    ) -> crate::error::Result<Vec<FolderPermission>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, project_id, subject_id, subject_type, actions
             FROM folder_permissions WHERE project_id = ?1 AND folder_id = ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id, folder_id], |row| {
            let actions_str: String = row.get(5)?;
            let actions: Vec<String> =
                serde_json::from_str(&actions_str).unwrap_or_default();
            Ok(FolderPermission {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                project_id: row.get(2)?,
                subject_id: row.get(3)?,
                subject_type: row.get(4)?,
                actions,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn set_permission(
        &self,
        project_id: &str,
        folder_id: &str,
        subject_id: String,
        subject_type: String,
        actions: Vec<String>,
    ) -> crate::error::Result<FolderPermission> {
        let id = format!("perm-{}", uuid::Uuid::new_v4());
        let actions_json = serde_json::to_string(&actions).unwrap_or_else(|_| "[]".to_string());
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR REPLACE INTO folder_permissions (id, folder_id, project_id, subject_id, subject_type, actions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, folder_id, project_id, subject_id, subject_type, actions_json],
        )?;
        Ok(FolderPermission {
            id,
            folder_id: folder_id.to_string(),
            project_id: project_id.to_string(),
            subject_id,
            subject_type,
            actions,
        })
    }

    fn row_to_folder(row: &rusqlite::Row<'_>) -> rusqlite::Result<FolderInfo> {
        Ok(FolderInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            parent_folder_id: row.get(2)?,
            name: row.get(3)?,
            display_name: row.get(4)?,
            created_at: row.get(5)?,
            last_modified_time: row.get(6)?,
        })
    }
}

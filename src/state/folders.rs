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

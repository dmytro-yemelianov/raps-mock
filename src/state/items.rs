// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Item (file lineage) information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInfo {
    pub id: String,
    pub project_id: String,
    pub folder_id: String,
    pub display_name: String,
    pub created_at: String,
    pub last_modified_time: String,
}

/// Item version information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemVersionInfo {
    pub id: String,
    pub item_id: String,
    pub version_number: i64,
    pub display_name: String,
    pub storage_size: i64,
    pub created_at: String,
}

/// Items state backed by SQLite
pub struct ItemState {
    db: Arc<Db>,
}

impl ItemState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.db.conn();

        // A demo item in the "Project Files" folder
        conn.execute(
            "INSERT OR IGNORE INTO items (id, project_id, folder_id, display_name, created_at, last_modified_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "mock-item-001",
                "b.default-project",
                "mock-top-folder-001",
                "Drawing.dwg",
                &now,
                &now,
            ],
        )
        .expect("failed to seed item");

        // Version 1 of that item
        conn.execute(
            "INSERT OR IGNORE INTO item_versions (id, item_id, version_number, display_name, storage_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "mock-version-001",
                "mock-item-001",
                1,
                "Drawing.dwg",
                1_024_000i64,
                &now,
            ],
        )
        .expect("failed to seed item version");
    }

    /// List items in a folder
    pub fn list_items_in_folder(
        &self,
        project_id: &str,
        folder_id: &str,
    ) -> crate::error::Result<Vec<ItemInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, folder_id, display_name, created_at, last_modified_time
             FROM items WHERE project_id = ?1 AND folder_id = ?2",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![project_id, folder_id],
            Self::row_to_item,
        )?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get an item by ID
    pub fn get_item(
        &self,
        project_id: &str,
        item_id: &str,
    ) -> crate::error::Result<Option<ItemInfo>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, project_id, folder_id, display_name, created_at, last_modified_time
                 FROM items WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![item_id, project_id],
                Self::row_to_item,
            )
            .optional()?)
    }

    /// Create an item in a folder
    pub fn create_item(
        &self,
        project_id: String,
        folder_id: String,
        display_name: String,
    ) -> crate::error::Result<ItemInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let item = ItemInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            folder_id,
            display_name: display_name.clone(),
            created_at: now.clone(),
            last_modified_time: now.clone(),
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO items (id, project_id, folder_id, display_name, created_at, last_modified_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                item.id,
                item.project_id,
                item.folder_id,
                item.display_name,
                item.created_at,
                item.last_modified_time,
            ],
        )?;

        // Auto-create version 1
        let version_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO item_versions (id, item_id, version_number, display_name, storage_size, created_at)
             VALUES (?1, ?2, 1, ?3, 0, ?4)",
            rusqlite::params![version_id, id, display_name, now],
        )?;

        Ok(item)
    }

    /// Update an item's display name
    pub fn update_item(
        &self,
        project_id: &str,
        item_id: &str,
        display_name: Option<String>,
    ) -> crate::error::Result<Option<ItemInfo>> {
        let current = match self.get_item(project_id, item_id)? {
            Some(i) => i,
            None => return Ok(None),
        };
        let new_name = display_name.unwrap_or(current.display_name);
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.db.conn();
        conn.execute(
            "UPDATE items SET display_name = ?1, last_modified_time = ?2
             WHERE id = ?3 AND project_id = ?4",
            rusqlite::params![new_name, now, item_id, project_id],
        )?;
        Ok(Some(ItemInfo {
            id: current.id,
            project_id: current.project_id,
            folder_id: current.folder_id,
            display_name: new_name,
            created_at: current.created_at,
            last_modified_time: now,
        }))
    }

    /// Delete an item and its versions
    pub fn delete_item(&self, project_id: &str, item_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM item_versions WHERE item_id = ?1",
            rusqlite::params![item_id],
        )?;
        let rows = conn.execute(
            "DELETE FROM items WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![item_id, project_id],
        )?;
        Ok(rows > 0)
    }

    /// List versions of an item
    pub fn list_versions(
        &self,
        item_id: &str,
    ) -> crate::error::Result<Vec<ItemVersionInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, version_number, display_name, storage_size, created_at
             FROM item_versions WHERE item_id = ?1 ORDER BY version_number ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![item_id], Self::row_to_version)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ItemInfo> {
        Ok(ItemInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            folder_id: row.get(2)?,
            display_name: row.get(3)?,
            created_at: row.get(4)?,
            last_modified_time: row.get(5)?,
        })
    }

    fn row_to_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<ItemVersionInfo> {
        Ok(ItemVersionInfo {
            id: row.get(0)?,
            item_id: row.get(1)?,
            version_number: row.get(2)?,
            display_name: row.get(3)?,
            storage_size: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

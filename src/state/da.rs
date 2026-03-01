// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Design Automation state: engines, app bundles, activities, work items.

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppBundleInfo {
    pub id: String,
    pub engine: String,
    pub description: String,
    pub version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityInfo {
    pub id: String,
    pub engine: String,
    pub description: Option<String>,
    pub version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkItemInfo {
    pub id: String,
    pub status: String,
    pub progress: Option<String>,
    pub activity_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AliasInfo {
    pub id: String,
    pub version: u32,
    pub receiver: String,
}

pub struct DaState {
    db: Arc<Db>,
}

impl DaState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    // ---- Engines (static list) ----

    pub fn list_engines(&self) -> Vec<String> {
        vec![
            "Autodesk.Revit+2025".to_string(),
            "Autodesk.AutoCAD+24".to_string(),
            "Autodesk.Inventor+2024".to_string(),
            "Autodesk.3dsMax+2025".to_string(),
        ]
    }

    // ---- App Bundles ----

    pub fn list_app_bundles(&self) -> crate::error::Result<Vec<String>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM app_bundles")?;
        let items = stmt.query_map([], |row| row.get(0))?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_app_bundle(
        &self,
        id: String,
        engine: String,
        description: String,
    ) -> crate::error::Result<AppBundleInfo> {
        let info = AppBundleInfo {
            id: id.clone(),
            engine,
            description,
            version: 1,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO app_bundles (id, engine, description, version) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![info.id, info.engine, info.description, info.version],
        )?;
        Ok(info)
    }

    pub fn delete_app_bundle(&self, id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM app_bundles WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }

    // ---- Activities ----

    pub fn list_activities(&self) -> crate::error::Result<Vec<String>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM activities")?;
        let items = stmt.query_map([], |row| row.get(0))?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_activity(
        &self,
        id: String,
        engine: String,
        description: Option<String>,
    ) -> crate::error::Result<ActivityInfo> {
        let info = ActivityInfo {
            id: id.clone(),
            engine,
            description,
            version: 1,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO activities (id, engine, description, version) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![info.id, info.engine, info.description, info.version],
        )?;
        Ok(info)
    }

    pub fn delete_activity(&self, id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM activities WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }

    // ---- Work Items ----

    pub fn list_work_items(&self) -> crate::error::Result<Vec<WorkItemInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, status, progress, activity_id FROM work_items")?;
        let items = stmt.query_map([], |row| {
            Ok(WorkItemInfo {
                id: row.get(0)?,
                status: row.get(1)?,
                progress: row.get(2)?,
                activity_id: row.get(3)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_work_item(&self, activity_id: String) -> crate::error::Result<WorkItemInfo> {
        let id = format!("workitem-{}", uuid::Uuid::new_v4());
        let info = WorkItemInfo {
            id: id.clone(),
            status: "pending".to_string(),
            progress: None,
            activity_id,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO work_items (id, status, progress, activity_id) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![info.id, info.status, info.progress, info.activity_id],
        )?;
        Ok(info)
    }

    pub fn get_work_item(&self, id: &str) -> crate::error::Result<Option<WorkItemInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, status, progress, activity_id FROM work_items WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(WorkItemInfo {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    progress: row.get(2)?,
                    activity_id: row.get(3)?,
                })
            },
        )
        .optional()?)
    }

    // ---- Aliases ----

    pub fn create_alias(
        &self,
        owner_type: &str,
        owner_id: &str,
        alias_id: String,
        version: u32,
    ) -> crate::error::Result<AliasInfo> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR REPLACE INTO da_aliases (alias_id, owner_type, owner_id, version)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![alias_id, owner_type, owner_id, version],
        )?;
        Ok(AliasInfo {
            id: alias_id,
            version,
            receiver: owner_id.to_string(),
        })
    }

    pub fn list_aliases(&self, owner_type: &str, owner_id: &str) -> crate::error::Result<Vec<AliasInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT alias_id, version, owner_id FROM da_aliases WHERE owner_type = ?1 AND owner_id = ?2",
        )?;
        let items = stmt.query_map(rusqlite::params![owner_type, owner_id], |row| {
            Ok(AliasInfo {
                id: row.get(0)?,
                version: row.get(1)?,
                receiver: row.get(2)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }
}

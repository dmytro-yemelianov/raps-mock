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

    pub fn list_app_bundles(&self) -> Vec<String> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM app_bundles")
            .expect("failed to prepare list app_bundles");
        stmt.query_map([], |row| row.get(0))
            .expect("failed to list app_bundles")
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn create_app_bundle(
        &self,
        id: String,
        engine: String,
        description: String,
    ) -> AppBundleInfo {
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
        )
        .expect("failed to create app bundle");
        info
    }

    pub fn delete_app_bundle(&self, id: &str) -> bool {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM app_bundles WHERE id = ?1",
            rusqlite::params![id],
        )
        .expect("failed to delete app bundle")
            > 0
    }

    // ---- Activities ----

    pub fn list_activities(&self) -> Vec<String> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM activities")
            .expect("failed to prepare list activities");
        stmt.query_map([], |row| row.get(0))
            .expect("failed to list activities")
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn create_activity(
        &self,
        id: String,
        engine: String,
        description: Option<String>,
    ) -> ActivityInfo {
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
        )
        .expect("failed to create activity");
        info
    }

    pub fn delete_activity(&self, id: &str) -> bool {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM activities WHERE id = ?1",
            rusqlite::params![id],
        )
        .expect("failed to delete activity")
            > 0
    }

    // ---- Work Items ----

    pub fn list_work_items(&self) -> Vec<WorkItemInfo> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, status, progress, activity_id FROM work_items")
            .expect("failed to prepare list work_items");
        stmt.query_map([], |row| {
            Ok(WorkItemInfo {
                id: row.get(0)?,
                status: row.get(1)?,
                progress: row.get(2)?,
                activity_id: row.get(3)?,
            })
        })
        .expect("failed to list work_items")
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn create_work_item(&self, activity_id: String) -> WorkItemInfo {
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
        )
        .expect("failed to create work item");
        info
    }

    pub fn get_work_item(&self, id: &str) -> Option<WorkItemInfo> {
        let conn = self.db.conn();
        conn.query_row(
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
        .optional()
        .expect("failed to get work item")
    }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// RFI information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RfiInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Submittal information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmittalInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Checklist information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistInfo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Checklist template information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// Unified ACC state for RFIs, Assets, Submittals, and Checklists
pub struct AccState {
    db: Arc<Db>,
}

impl AccState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let demo_project = "mock-project-001";
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.db.conn();

        // RFIs
        for (id, title) in [
            ("rfi-demo-001", "Demo RFI - MEP Routing"),
            ("demo-struct-eng-001", "Structural RFI"),
            ("lc-rfi-001", "Lifecycle RFI"),
        ] {
            conn.execute(
                "INSERT OR IGNORE INTO rfis (id, project_id, title, description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, demo_project, title, Option::<String>::None, "open", now],
            )
            .expect("failed to seed rfi");
        }

        // Assets
        for (id, title) in [
            ("ast-demo-001", "Demo Asset - HVAC Unit"),
            ("ast-chiller-01", "Chiller CH-01"),
            ("ast-chiller-02", "Chiller CH-02"),
        ] {
            conn.execute(
                "INSERT OR IGNORE INTO assets (id, project_id, title, description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, demo_project, title, Some(title), "active", now],
            )
            .expect("failed to seed asset");
        }

        // Submittals
        for (id, title) in [
            ("sub-demo-001", "Demo Submittal - Concrete Mix"),
            ("lc-sub-001", "Lifecycle Submittal"),
        ] {
            conn.execute(
                "INSERT OR IGNORE INTO submittals (id, project_id, title, description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, demo_project, title, Option::<String>::None, "waiting", now],
            )
            .expect("failed to seed submittal");
        }

        // Checklists
        conn.execute(
            "INSERT OR IGNORE INTO checklists (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "chk-demo-001",
                demo_project,
                "Demo Checklist - Pre-Pour Inspection",
                Option::<String>::None,
                "not_started",
                now,
            ],
        )
        .expect("failed to seed checklist");
    }

    // ---- RFIs ----

    pub fn list_rfis(&self, project_id: &str) -> crate::error::Result<Vec<RfiInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, status, created_at
                 FROM rfis WHERE project_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_rfi)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn get_rfi(&self, project_id: &str, rfi_id: &str) -> crate::error::Result<Option<RfiInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, project_id, title, description, status, created_at
             FROM rfis WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![rfi_id, project_id],
            Self::row_to_rfi,
        )
        .optional()?)
    }

    pub fn create_rfi(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> crate::error::Result<RfiInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let rfi = RfiInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            title,
            description,
            status: "open".to_string(),
            created_at: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO rfis (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                rfi.id,
                rfi.project_id,
                rfi.title,
                rfi.description,
                rfi.status,
                rfi.created_at
            ],
        )?;
        Ok(rfi)
    }

    pub fn update_rfi(
        &self,
        project_id: &str,
        rfi_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<RfiInfo>> {
        let current = match self.get_rfi(project_id, rfi_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_title = title.unwrap_or(current.title);
        let new_desc = description.or(current.description);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE rfis SET title = ?1, description = ?2, status = ?3 WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_title, new_desc, new_status, rfi_id, project_id],
        )?;
        Ok(Some(RfiInfo {
            id: current.id,
            project_id: current.project_id,
            title: new_title,
            description: new_desc,
            status: new_status,
            created_at: current.created_at,
        }))
    }

    pub fn delete_rfi(&self, project_id: &str, rfi_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM rfis WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![rfi_id, project_id],
        )?;
        Ok(rows > 0)
    }

    // ---- Assets ----

    pub fn list_assets(&self, project_id: &str) -> crate::error::Result<Vec<AssetInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, status, created_at
                 FROM assets WHERE project_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_asset)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn get_asset(&self, project_id: &str, asset_id: &str) -> crate::error::Result<Option<AssetInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, project_id, title, description, status, created_at
             FROM assets WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![asset_id, project_id],
            Self::row_to_asset,
        )
        .optional()?)
    }

    pub fn create_asset(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> crate::error::Result<AssetInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let asset = AssetInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            title,
            description,
            status: "active".to_string(),
            created_at: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO assets (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                asset.id,
                asset.project_id,
                asset.title,
                asset.description,
                asset.status,
                asset.created_at
            ],
        )?;
        Ok(asset)
    }

    pub fn update_asset(
        &self,
        project_id: &str,
        asset_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<AssetInfo>> {
        let current = match self.get_asset(project_id, asset_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_title = title.unwrap_or(current.title);
        let new_desc = description.or(current.description);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE assets SET title = ?1, description = ?2, status = ?3 WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_title, new_desc, new_status, asset_id, project_id],
        )?;
        Ok(Some(AssetInfo {
            id: current.id,
            project_id: current.project_id,
            title: new_title,
            description: new_desc,
            status: new_status,
            created_at: current.created_at,
        }))
    }

    pub fn delete_asset(&self, project_id: &str, asset_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM assets WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![asset_id, project_id],
        )?;
        Ok(rows > 0)
    }

    // ---- Submittals ----

    pub fn list_submittals(&self, project_id: &str) -> crate::error::Result<Vec<SubmittalInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, status, created_at
                 FROM submittals WHERE project_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_submittal)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn get_submittal(&self, project_id: &str, submittal_id: &str) -> crate::error::Result<Option<SubmittalInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, project_id, title, description, status, created_at
             FROM submittals WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![submittal_id, project_id],
            Self::row_to_submittal,
        )
        .optional()?)
    }

    pub fn create_submittal(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> crate::error::Result<SubmittalInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let submittal = SubmittalInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            title,
            description,
            status: "waiting".to_string(),
            created_at: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO submittals (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                submittal.id,
                submittal.project_id,
                submittal.title,
                submittal.description,
                submittal.status,
                submittal.created_at
            ],
        )?;
        Ok(submittal)
    }

    pub fn update_submittal(
        &self,
        project_id: &str,
        submittal_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<SubmittalInfo>> {
        let current = match self.get_submittal(project_id, submittal_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_title = title.unwrap_or(current.title);
        let new_desc = description.or(current.description);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE submittals SET title = ?1, description = ?2, status = ?3 WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_title, new_desc, new_status, submittal_id, project_id],
        )?;
        Ok(Some(SubmittalInfo {
            id: current.id,
            project_id: current.project_id,
            title: new_title,
            description: new_desc,
            status: new_status,
            created_at: current.created_at,
        }))
    }

    pub fn delete_submittal(&self, project_id: &str, submittal_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM submittals WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![submittal_id, project_id],
        )?;
        Ok(rows > 0)
    }

    // ---- Checklists ----

    pub fn list_checklists(&self, project_id: &str) -> crate::error::Result<Vec<ChecklistInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, status, created_at
                 FROM checklists WHERE project_id = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_checklist)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn get_checklist(&self, project_id: &str, checklist_id: &str) -> crate::error::Result<Option<ChecklistInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT id, project_id, title, description, status, created_at
             FROM checklists WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![checklist_id, project_id],
            Self::row_to_checklist,
        )
        .optional()?)
    }

    pub fn create_checklist(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> crate::error::Result<ChecklistInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let checklist = ChecklistInfo {
            id: id.clone(),
            project_id: project_id.clone(),
            title,
            description,
            status: "not_started".to_string(),
            created_at: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO checklists (id, project_id, title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                checklist.id,
                checklist.project_id,
                checklist.title,
                checklist.description,
                checklist.status,
                checklist.created_at
            ],
        )?;
        Ok(checklist)
    }

    pub fn update_checklist(
        &self,
        project_id: &str,
        checklist_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<ChecklistInfo>> {
        let current = match self.get_checklist(project_id, checklist_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_title = title.unwrap_or(current.title);
        let new_desc = description.or(current.description);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE checklists SET title = ?1, description = ?2, status = ?3 WHERE id = ?4 AND project_id = ?5",
            rusqlite::params![new_title, new_desc, new_status, checklist_id, project_id],
        )?;
        Ok(Some(ChecklistInfo {
            id: current.id,
            project_id: current.project_id,
            title: new_title,
            description: new_desc,
            status: new_status,
            created_at: current.created_at,
        }))
    }

    /// Return a static list of checklist templates
    pub fn list_templates(&self, _project_id: &str) -> Vec<ChecklistTemplate> {
        vec![
            ChecklistTemplate {
                id: "tpl-demo-001".to_string(),
                title: "Safety Inspection".to_string(),
                description: "Standard safety inspection checklist".to_string(),
            },
            ChecklistTemplate {
                id: "tpl-demo-002".to_string(),
                title: "Quality Assurance".to_string(),
                description: "Quality assurance review checklist".to_string(),
            },
            ChecklistTemplate {
                id: "tpl-demo-003".to_string(),
                title: "Commissioning".to_string(),
                description: "Building commissioning checklist".to_string(),
            },
        ]
    }

    // ---- Row mappers ----

    fn row_to_rfi(row: &rusqlite::Row<'_>) -> rusqlite::Result<RfiInfo> {
        Ok(RfiInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }

    fn row_to_asset(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssetInfo> {
        Ok(AssetInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }

    fn row_to_submittal(row: &rusqlite::Row<'_>) -> rusqlite::Result<SubmittalInfo> {
        Ok(SubmittalInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }

    fn row_to_checklist(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChecklistInfo> {
        Ok(ChecklistInfo {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

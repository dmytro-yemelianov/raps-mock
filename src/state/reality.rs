// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Reality Capture state: photoscenes.

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhotosceneInfo {
    pub photoscene_id: String,
    pub name: String,
    pub scene_type: String,
    pub convert_format: String,
    pub status: String,
    pub progress: String,
    pub progress_msg: Option<String>,
    pub scene_link: Option<String>,
}

pub struct RealityState {
    db: Arc<Db>,
}

impl RealityState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn list_photoscenes(&self) -> crate::error::Result<Vec<PhotosceneInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT photoscene_id, name, scene_type, convert_format, status, progress, progress_msg, scene_link FROM photoscenes",
            )?;
        let items = stmt.query_map([], Self::row_to_photoscene)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_photoscene(
        &self,
        name: String,
        scene_type: String,
        convert_format: String,
    ) -> crate::error::Result<PhotosceneInfo> {
        let id = format!("ps-{}", uuid::Uuid::new_v4());
        let info = PhotosceneInfo {
            photoscene_id: id.clone(),
            name,
            scene_type,
            convert_format,
            status: "Created".to_string(),
            progress: "0".to_string(),
            progress_msg: None,
            scene_link: None,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO photoscenes (photoscene_id, name, scene_type, convert_format, status, progress, progress_msg, scene_link)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                info.photoscene_id,
                info.name,
                info.scene_type,
                info.convert_format,
                info.status,
                info.progress,
                info.progress_msg,
                info.scene_link,
            ],
        )?;
        Ok(info)
    }

    pub fn get_photoscene(&self, id: &str) -> crate::error::Result<Option<PhotosceneInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT photoscene_id, name, scene_type, convert_format, status, progress, progress_msg, scene_link
             FROM photoscenes WHERE photoscene_id = ?1",
            rusqlite::params![id],
            Self::row_to_photoscene,
        )
        .optional()?)
    }

    pub fn process_photoscene(&self, id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn
            .execute(
                "UPDATE photoscenes SET status = 'Done', progress = '100', progress_msg = 'Complete', scene_link = 'https://example.com/download/model.obj'
                 WHERE photoscene_id = ?1",
                rusqlite::params![id],
            )?;
        Ok(rows > 0)
    }

    pub fn delete_photoscene(&self, id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM photoscenes WHERE photoscene_id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }

    fn row_to_photoscene(row: &rusqlite::Row<'_>) -> rusqlite::Result<PhotosceneInfo> {
        Ok(PhotosceneInfo {
            photoscene_id: row.get(0)?,
            name: row.get(1)?,
            scene_type: row.get(2)?,
            convert_format: row.get(3)?,
            status: row.get(4)?,
            progress: row.get(5)?,
            progress_msg: row.get(6)?,
            scene_link: row.get(7)?,
        })
    }
}

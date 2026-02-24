// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

/// Translation job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "inprogress")]
    InProgress,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

impl TranslationStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "inprogress",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }

    fn from_str_val(s: &str) -> Self {
        match s {
            "inprogress" => Self::InProgress,
            "success" => Self::Success,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

/// Translation job information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationJob {
    pub urn: String,
    pub status: TranslationStatus,
    pub progress: String,
    pub created_at: i64,
}

/// Model Derivative translation state
pub struct TranslationState {
    db: Arc<Db>,
}

impl TranslationState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Create a new translation job
    pub fn create_job(&self, urn: String) -> TranslationJob {
        let now = chrono::Utc::now().timestamp_millis();
        let job = TranslationJob {
            urn: urn.clone(),
            status: TranslationStatus::Pending,
            progress: "0%".to_string(),
            created_at: now,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO translations (urn, status, progress, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![job.urn, job.status.as_str(), job.progress, job.created_at],
        )
        .expect("failed to create translation job");
        job
    }

    /// Get a translation job
    pub fn get_job(&self, urn: &str) -> Option<TranslationJob> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT urn, status, progress, created_at FROM translations WHERE urn = ?1",
            rusqlite::params![urn],
            |row| {
                let status_str: String = row.get(1)?;
                Ok(TranslationJob {
                    urn: row.get(0)?,
                    status: TranslationStatus::from_str_val(&status_str),
                    progress: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .expect("failed to get translation job")
    }

    /// Update job status
    pub fn update_job_status(
        &self,
        urn: &str,
        status: TranslationStatus,
        progress: String,
    ) -> bool {
        let conn = self.db.conn();
        let rows = conn
            .execute(
                "UPDATE translations SET status = ?1, progress = ?2 WHERE urn = ?3",
                rusqlite::params![status.as_str(), progress, urn],
            )
            .expect("failed to update translation job");
        drop(conn);
        if status == TranslationStatus::Success {
            self.generate_mock_metadata(urn);
        }
        rows > 0
    }

    /// List all translation jobs
    pub fn list_all(&self) -> Vec<TranslationJob> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT urn, status, progress, created_at FROM translations")
            .expect("failed to prepare list translations");
        stmt.query_map([], |row| {
            let status_str: String = row.get(1)?;
            Ok(TranslationJob {
                urn: row.get(0)?,
                status: TranslationStatus::from_str_val(&status_str),
                progress: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .expect("failed to list translations")
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Simulate job progression
    pub fn simulate_progress(&self, urn: &str) {
        if let Some(job) = self.get_job(urn) {
            let (new_status, new_progress) = match job.status {
                TranslationStatus::Pending => (TranslationStatus::InProgress, "25%".to_string()),
                TranslationStatus::InProgress => {
                    let progress_num: u32 =
                        job.progress.trim_end_matches('%').parse().unwrap_or(25);
                    if progress_num < 100 {
                        (
                            TranslationStatus::InProgress,
                            format!("{}%", progress_num + 25),
                        )
                    } else {
                        (TranslationStatus::Success, "complete".to_string())
                    }
                }
                _ => return,
            };
            self.update_job_status(urn, new_status, new_progress);
        }
    }

    /// Get model metadata for a translated URN
    pub fn get_metadata(&self, urn: &str) -> Option<Value> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT metadata_json FROM translations WHERE urn = ?1 AND status = 'success'",
            rusqlite::params![urn],
            |row| {
                let json_str: Option<String> = row.get(0)?;
                Ok(json_str)
            },
        )
        .optional()
        .expect("failed to get metadata")
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Get object tree for a translated URN and view GUID
    pub fn get_object_tree(&self, urn: &str, _guid: &str) -> Option<Value> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT object_tree_json FROM translations WHERE urn = ?1 AND status = 'success'",
            rusqlite::params![urn],
            |row| {
                let json_str: Option<String> = row.get(0)?;
                Ok(json_str)
            },
        )
        .optional()
        .expect("failed to get object tree")
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Get properties for a translated URN and view GUID
    pub fn get_properties(&self, urn: &str, _guid: &str) -> Option<Value> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT properties_json FROM translations WHERE urn = ?1 AND status = 'success'",
            rusqlite::params![urn],
            |row| {
                let json_str: Option<String> = row.get(0)?;
                Ok(json_str)
            },
        )
        .optional()
        .expect("failed to get properties")
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Generate synthetic mock metadata when a translation succeeds
    fn generate_mock_metadata(&self, urn: &str) {
        let guid = format!("mock-guid-{}", &urn[..8.min(urn.len())]);

        let metadata = json!({
            "type": "metadata",
            "metadata": [{
                "guid": guid,
                "name": "3D View",
                "role": "3d",
                "mime_type": "application/autodesk-svf2",
                "has_thumbnail": "true",
                "progress": "complete"
            }]
        });

        let object_tree = json!({
            "type": "objects",
            "objects": [{
                "objectid": 1,
                "name": "Model",
                "objects": [
                    { "objectid": 2, "name": "Wall [123456]", "objects": [] },
                    { "objectid": 3, "name": "Floor [789012]", "objects": [] },
                    { "objectid": 4, "name": "Roof [345678]", "objects": [] }
                ]
            }]
        });

        let properties = json!({
            "type": "properties",
            "collection": [
                {
                    "objectid": 1,
                    "name": "Model",
                    "externalId": "ext-id-001",
                    "properties": { "Category": "Revit Model", "Area": "150.5 m²" }
                },
                {
                    "objectid": 2,
                    "name": "Wall [123456]",
                    "externalId": "ext-id-002",
                    "properties": { "Category": "Walls", "Height": "3.0 m", "Width": "0.2 m" }
                },
                {
                    "objectid": 3,
                    "name": "Floor [789012]",
                    "externalId": "ext-id-003",
                    "properties": { "Category": "Floors", "Area": "50.0 m²", "Thickness": "0.3 m" }
                },
                {
                    "objectid": 4,
                    "name": "Roof [345678]",
                    "externalId": "ext-id-004",
                    "properties": { "Category": "Roofs", "Area": "55.0 m²", "Slope": "30°" }
                }
            ]
        });

        let conn = self.db.conn();
        conn.execute(
            "UPDATE translations SET metadata_json = ?1, object_tree_json = ?2, properties_json = ?3 WHERE urn = ?4",
            rusqlite::params![
                serde_json::to_string(&metadata).unwrap(),
                serde_json::to_string(&object_tree).unwrap(),
                serde_json::to_string(&properties).unwrap(),
                urn
            ],
        )
        .expect("failed to store mock metadata");
    }
}

use rusqlite::OptionalExtension;

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Hub information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubInfo {
    pub id: String,
    pub name: String,
    pub region: String,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub hub_id: String,
    pub name: String,
}

/// Data Management state
pub struct ProjectState {
    db: Arc<Db>,
}

impl ProjectState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR IGNORE INTO hubs (id, name, region) VALUES (?1, ?2, ?3)",
            rusqlite::params!["b.default-hub", "Default Hub", "US"],
        )
        .expect("failed to seed hub");
        conn.execute(
            "INSERT OR IGNORE INTO projects (id, hub_id, name) VALUES (?1, ?2, ?3)",
            rusqlite::params!["b.default-project", "b.default-hub", "Default Project"],
        )
        .expect("failed to seed project");
    }

    /// List all hubs
    pub fn list_hubs(&self) -> Vec<HubInfo> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, name, region FROM hubs")
            .expect("failed to prepare list hubs");
        stmt.query_map([], |row| {
            Ok(HubInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                region: row.get(2)?,
            })
        })
        .expect("failed to list hubs")
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Get a hub by ID
    pub fn get_hub(&self, hub_id: &str) -> Option<HubInfo> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT id, name, region FROM hubs WHERE id = ?1",
            rusqlite::params![hub_id],
            |row| {
                Ok(HubInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    region: row.get(2)?,
                })
            },
        )
        .optional()
        .expect("failed to get hub")
    }

    /// List projects in a hub
    pub fn list_projects(&self, hub_id: &str) -> Vec<ProjectInfo> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, hub_id, name FROM projects WHERE hub_id = ?1")
            .expect("failed to prepare list projects");
        stmt.query_map(rusqlite::params![hub_id], |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                hub_id: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .expect("failed to list projects")
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Get a project by ID
    pub fn get_project(&self, project_id: &str) -> Option<ProjectInfo> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT id, hub_id, name FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| {
                Ok(ProjectInfo {
                    id: row.get(0)?,
                    hub_id: row.get(1)?,
                    name: row.get(2)?,
                })
            },
        )
        .optional()
        .expect("failed to get project")
    }

    /// List all projects across all hubs
    pub fn list_all_projects(&self) -> Vec<ProjectInfo> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT id, hub_id, name FROM projects")
            .expect("failed to prepare list all projects");
        stmt.query_map([], |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                hub_id: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .expect("failed to list all projects")
        .filter_map(|r| r.ok())
        .collect()
    }
}

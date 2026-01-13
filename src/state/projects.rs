// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

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
    hubs: DashMap<String, HubInfo>,
    projects: DashMap<String, ProjectInfo>,
    /// Map of hub_id -> project_ids
    hub_projects: DashMap<String, Vec<String>>,
}

impl ProjectState {
    pub fn new() -> Self {
        let state = Self {
            hubs: DashMap::new(),
            projects: DashMap::new(),
            hub_projects: DashMap::new(),
        };

        // Initialize with some default data
        state.init_defaults();
        state
    }

    fn init_defaults(&self) {
        let hub_id = "b.default-hub".to_string();
        let hub = HubInfo {
            id: hub_id.clone(),
            name: "Default Hub".to_string(),
            region: "US".to_string(),
        };
        self.hubs.insert(hub_id.clone(), hub);

        let project_id = "b.default-project".to_string();
        let project = ProjectInfo {
            id: project_id.clone(),
            hub_id: hub_id.clone(),
            name: "Default Project".to_string(),
        };
        self.projects.insert(project_id.clone(), project);
        self.hub_projects
            .entry(hub_id)
            .or_default()
            .push(project_id);
    }

    /// List all hubs
    pub fn list_hubs(&self) -> Vec<HubInfo> {
        self.hubs.iter().map(|h| h.value().clone()).collect()
    }

    /// Get a hub by ID
    pub fn get_hub(&self, hub_id: &str) -> Option<HubInfo> {
        self.hubs.get(hub_id).map(|h| h.clone())
    }

    /// List projects in a hub
    pub fn list_projects(&self, hub_id: &str) -> Vec<ProjectInfo> {
        self.hub_projects
            .get(hub_id)
            .map(|project_ids| {
                project_ids
                    .iter()
                    .filter_map(|id| self.projects.get(id).map(|p| p.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a project by ID
    pub fn get_project(&self, project_id: &str) -> Option<ProjectInfo> {
        self.projects.get(project_id).map(|p| p.clone())
    }
}

impl Default for ProjectState {
    fn default() -> Self {
        Self::new()
    }
}

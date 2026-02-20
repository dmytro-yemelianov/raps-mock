// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

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
    /// Map of project_id -> rfi_id -> RfiInfo
    rfis: DashMap<String, DashMap<String, RfiInfo>>,
    /// Map of project_id -> asset_id -> AssetInfo
    assets: DashMap<String, DashMap<String, AssetInfo>>,
    /// Map of project_id -> submittal_id -> SubmittalInfo
    submittals: DashMap<String, DashMap<String, SubmittalInfo>>,
    /// Map of project_id -> checklist_id -> ChecklistInfo
    checklists: DashMap<String, DashMap<String, ChecklistInfo>>,
}

impl AccState {
    pub fn new() -> Self {
        let state = Self {
            rfis: DashMap::new(),
            assets: DashMap::new(),
            submittals: DashMap::new(),
            checklists: DashMap::new(),
        };
        // Pre-seed well-known demo data used by raps-examples tests
        let demo_project = "mock-project-001";
        let now = chrono::Utc::now().to_rfc3339();

        // RFIs
        {
            let rfis = state.rfis.entry(demo_project.to_string()).or_default();
            for (id, title) in [
                ("rfi-demo-001", "Demo RFI - MEP Routing"),
                ("demo-struct-eng-001", "Structural RFI"),
                ("lc-rfi-001", "Lifecycle RFI"),
            ] {
                rfis.insert(
                    id.to_string(),
                    RfiInfo {
                        id: id.to_string(),
                        project_id: demo_project.to_string(),
                        title: title.to_string(),
                        description: None,
                        status: "open".to_string(),
                        created_at: now.clone(),
                    },
                );
            }
        }

        // Assets
        {
            let assets = state.assets.entry(demo_project.to_string()).or_default();
            for (id, title) in [
                ("ast-demo-001", "Demo Asset - HVAC Unit"),
                ("ast-chiller-01", "Chiller CH-01"),
                ("ast-chiller-02", "Chiller CH-02"),
            ] {
                assets.insert(
                    id.to_string(),
                    AssetInfo {
                        id: id.to_string(),
                        project_id: demo_project.to_string(),
                        title: title.to_string(),
                        description: Some(title.to_string()),
                        status: "active".to_string(),
                        created_at: now.clone(),
                    },
                );
            }
        }

        // Submittals
        {
            let submittals = state.submittals.entry(demo_project.to_string()).or_default();
            for (id, title) in [
                ("sub-demo-001", "Demo Submittal - Concrete Mix"),
                ("lc-sub-001", "Lifecycle Submittal"),
            ] {
                submittals.insert(
                    id.to_string(),
                    SubmittalInfo {
                        id: id.to_string(),
                        project_id: demo_project.to_string(),
                        title: title.to_string(),
                        description: None,
                        status: "waiting".to_string(),
                        created_at: now.clone(),
                    },
                );
            }
        }

        // Checklists
        {
            let checklists = state.checklists.entry(demo_project.to_string()).or_default();
            checklists.insert(
                "chk-demo-001".to_string(),
                ChecklistInfo {
                    id: "chk-demo-001".to_string(),
                    project_id: demo_project.to_string(),
                    title: "Demo Checklist - Pre-Pour Inspection".to_string(),
                    description: None,
                    status: "not_started".to_string(),
                    created_at: now.clone(),
                },
            );
        }

        state
    }

    // ---- RFIs ----

    pub fn list_rfis(&self, project_id: &str) -> Vec<RfiInfo> {
        self.rfis
            .get(project_id)
            .map(|items| items.iter().map(|r| r.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_rfi(&self, project_id: &str, rfi_id: &str) -> Option<RfiInfo> {
        self.rfis.get(project_id)?.get(rfi_id).map(|r| r.clone())
    }

    pub fn create_rfi(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> RfiInfo {
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
        let project_rfis = self.rfis.entry(project_id).or_default();
        project_rfis.insert(id, rfi.clone());
        rfi
    }

    pub fn update_rfi(
        &self,
        project_id: &str,
        rfi_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Option<RfiInfo> {
        let project_rfis = self.rfis.get(project_id)?;
        let mut rfi = project_rfis.get_mut(rfi_id)?;
        if let Some(t) = title {
            rfi.title = t;
        }
        if let Some(d) = description {
            rfi.description = Some(d);
        }
        if let Some(s) = status {
            rfi.status = s;
        }
        Some(rfi.clone())
    }

    pub fn delete_rfi(&self, project_id: &str, rfi_id: &str) -> bool {
        self.rfis
            .get(project_id)
            .map(|items| items.remove(rfi_id).is_some())
            .unwrap_or(false)
    }

    // ---- Assets ----

    pub fn list_assets(&self, project_id: &str) -> Vec<AssetInfo> {
        self.assets
            .get(project_id)
            .map(|items| items.iter().map(|a| a.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_asset(&self, project_id: &str, asset_id: &str) -> Option<AssetInfo> {
        self.assets
            .get(project_id)?
            .get(asset_id)
            .map(|a| a.clone())
    }

    pub fn create_asset(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> AssetInfo {
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
        let project_assets = self.assets.entry(project_id).or_default();
        project_assets.insert(id, asset.clone());
        asset
    }

    pub fn update_asset(
        &self,
        project_id: &str,
        asset_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Option<AssetInfo> {
        let project_assets = self.assets.get(project_id)?;
        let mut asset = project_assets.get_mut(asset_id)?;
        if let Some(t) = title {
            asset.title = t;
        }
        if let Some(d) = description {
            asset.description = Some(d);
        }
        if let Some(s) = status {
            asset.status = s;
        }
        Some(asset.clone())
    }

    pub fn delete_asset(&self, project_id: &str, asset_id: &str) -> bool {
        self.assets
            .get(project_id)
            .map(|items| items.remove(asset_id).is_some())
            .unwrap_or(false)
    }

    // ---- Submittals ----

    pub fn list_submittals(&self, project_id: &str) -> Vec<SubmittalInfo> {
        self.submittals
            .get(project_id)
            .map(|items| items.iter().map(|s| s.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_submittal(&self, project_id: &str, submittal_id: &str) -> Option<SubmittalInfo> {
        self.submittals
            .get(project_id)?
            .get(submittal_id)
            .map(|s| s.clone())
    }

    pub fn create_submittal(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> SubmittalInfo {
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
        let project_submittals = self.submittals.entry(project_id).or_default();
        project_submittals.insert(id, submittal.clone());
        submittal
    }

    pub fn update_submittal(
        &self,
        project_id: &str,
        submittal_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Option<SubmittalInfo> {
        let project_submittals = self.submittals.get(project_id)?;
        let mut submittal = project_submittals.get_mut(submittal_id)?;
        if let Some(t) = title {
            submittal.title = t;
        }
        if let Some(d) = description {
            submittal.description = Some(d);
        }
        if let Some(s) = status {
            submittal.status = s;
        }
        Some(submittal.clone())
    }

    pub fn delete_submittal(&self, project_id: &str, submittal_id: &str) -> bool {
        self.submittals
            .get(project_id)
            .map(|items| items.remove(submittal_id).is_some())
            .unwrap_or(false)
    }

    // ---- Checklists ----

    pub fn list_checklists(&self, project_id: &str) -> Vec<ChecklistInfo> {
        self.checklists
            .get(project_id)
            .map(|items| items.iter().map(|c| c.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_checklist(&self, project_id: &str, checklist_id: &str) -> Option<ChecklistInfo> {
        self.checklists
            .get(project_id)?
            .get(checklist_id)
            .map(|c| c.clone())
    }

    pub fn create_checklist(
        &self,
        project_id: String,
        title: String,
        description: Option<String>,
    ) -> ChecklistInfo {
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
        let project_checklists = self.checklists.entry(project_id).or_default();
        project_checklists.insert(id, checklist.clone());
        checklist
    }

    pub fn update_checklist(
        &self,
        project_id: &str,
        checklist_id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Option<ChecklistInfo> {
        let project_checklists = self.checklists.get(project_id)?;
        let mut checklist = project_checklists.get_mut(checklist_id)?;
        if let Some(t) = title {
            checklist.title = t;
        }
        if let Some(d) = description {
            checklist.description = Some(d);
        }
        if let Some(s) = status {
            checklist.status = s;
        }
        Some(checklist.clone())
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
}

impl Default for AccState {
    fn default() -> Self {
        Self::new()
    }
}

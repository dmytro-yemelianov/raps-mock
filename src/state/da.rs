// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Design Automation state: engines, app bundles, activities, work items.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

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
    app_bundles: DashMap<String, AppBundleInfo>,
    activities: DashMap<String, ActivityInfo>,
    work_items: DashMap<String, WorkItemInfo>,
}

impl DaState {
    pub fn new() -> Self {
        Self {
            app_bundles: DashMap::new(),
            activities: DashMap::new(),
            work_items: DashMap::new(),
        }
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
        self.app_bundles.iter().map(|r| r.key().clone()).collect()
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
        self.app_bundles.insert(id, info.clone());
        info
    }

    pub fn delete_app_bundle(&self, id: &str) -> bool {
        self.app_bundles.remove(id).is_some()
    }

    // ---- Activities ----

    pub fn list_activities(&self) -> Vec<String> {
        self.activities.iter().map(|r| r.key().clone()).collect()
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
        self.activities.insert(id, info.clone());
        info
    }

    pub fn delete_activity(&self, id: &str) -> bool {
        self.activities.remove(id).is_some()
    }

    // ---- Work Items ----

    pub fn list_work_items(&self) -> Vec<WorkItemInfo> {
        self.work_items.iter().map(|r| r.value().clone()).collect()
    }

    pub fn create_work_item(&self, activity_id: String) -> WorkItemInfo {
        let id = format!("workitem-{}", uuid::Uuid::new_v4());
        let info = WorkItemInfo {
            id: id.clone(),
            status: "pending".to_string(),
            progress: None,
            activity_id,
        };
        self.work_items.insert(id, info.clone());
        info
    }

    pub fn get_work_item(&self, id: &str) -> Option<WorkItemInfo> {
        self.work_items.get(id).map(|r| r.value().clone())
    }
}

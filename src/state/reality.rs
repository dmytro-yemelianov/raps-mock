// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Reality Capture state: photoscenes.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

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
    photoscenes: DashMap<String, PhotosceneInfo>,
}

impl RealityState {
    pub fn new() -> Self {
        Self {
            photoscenes: DashMap::new(),
        }
    }

    pub fn list_photoscenes(&self) -> Vec<PhotosceneInfo> {
        self.photoscenes.iter().map(|r| r.value().clone()).collect()
    }

    pub fn create_photoscene(
        &self,
        name: String,
        scene_type: String,
        convert_format: String,
    ) -> PhotosceneInfo {
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
        self.photoscenes.insert(id, info.clone());
        info
    }

    pub fn get_photoscene(&self, id: &str) -> Option<PhotosceneInfo> {
        self.photoscenes.get(id).map(|r| r.value().clone())
    }

    pub fn process_photoscene(&self, id: &str) -> bool {
        if let Some(mut entry) = self.photoscenes.get_mut(id) {
            // Jump straight to Done for mock purposes
            entry.status = "Done".to_string();
            entry.progress = "100".to_string();
            entry.progress_msg = Some("Complete".to_string());
            entry.scene_link = Some("https://example.com/download/model.obj".to_string());
            true
        } else {
            false
        }
    }

    pub fn delete_photoscene(&self, id: &str) -> bool {
        self.photoscenes.remove(id).is_some()
    }
}

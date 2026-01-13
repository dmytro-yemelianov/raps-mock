// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

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

/// Translation job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationJob {
    pub urn: String,
    pub status: TranslationStatus,
    pub progress: String,
    pub created_at: i64,
}

/// Model Derivative translation state
pub struct TranslationState {
    jobs: DashMap<String, TranslationJob>,
}

impl TranslationState {
    pub fn new() -> Self {
        Self {
            jobs: DashMap::new(),
        }
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
        self.jobs.insert(urn, job.clone());
        job
    }

    /// Get a translation job
    pub fn get_job(&self, urn: &str) -> Option<TranslationJob> {
        self.jobs.get(urn).map(|j| j.clone())
    }

    /// Update job status
    pub fn update_job_status(
        &self,
        urn: &str,
        status: TranslationStatus,
        progress: String,
    ) -> bool {
        if let Some(mut job) = self.jobs.get_mut(urn) {
            job.status = status;
            job.progress = progress;
            true
        } else {
            false
        }
    }

    /// Simulate job progression
    pub fn simulate_progress(&self, urn: &str) {
        if let Some(mut job) = self.jobs.get_mut(urn) {
            match job.status {
                TranslationStatus::Pending => {
                    job.status = TranslationStatus::InProgress;
                    job.progress = "25%".to_string();
                }
                TranslationStatus::InProgress => {
                    let progress_num: u32 =
                        job.progress.trim_end_matches('%').parse().unwrap_or(25);
                    if progress_num < 100 {
                        job.progress = format!("{}%", progress_num + 25);
                    } else {
                        job.status = TranslationStatus::Success;
                        job.progress = "complete".to_string();
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for TranslationState {
    fn default() -> Self {
        Self::new()
    }
}

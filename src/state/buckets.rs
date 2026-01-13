// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// OSS bucket information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub bucket_key: String,
    pub bucket_owner: String,
    pub created_date: i64,
    pub policy_key: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub auth_id: String,
    pub access: String,
}

/// OSS bucket state
pub struct BucketState {
    buckets: DashMap<String, BucketInfo>,
}

impl BucketState {
    pub fn new() -> Self {
        Self {
            buckets: DashMap::new(),
        }
    }

    /// Create a new bucket
    pub fn create_bucket(&self, bucket_key: String, policy_key: String) -> BucketInfo {
        let now = chrono::Utc::now().timestamp_millis();
        let bucket = BucketInfo {
            bucket_key: bucket_key.clone(),
            bucket_owner: "mock-owner".to_string(),
            created_date: now,
            policy_key,
            permissions: vec![],
        };
        self.buckets.insert(bucket_key, bucket.clone());
        bucket
    }

    /// Get a bucket by key
    pub fn get_bucket(&self, bucket_key: &str) -> Option<BucketInfo> {
        self.buckets.get(bucket_key).map(|b| b.clone())
    }

    /// List all buckets
    pub fn list_buckets(&self) -> Vec<BucketInfo> {
        self.buckets.iter().map(|e| e.value().clone()).collect()
    }

    /// Delete a bucket
    pub fn delete_bucket(&self, bucket_key: &str) -> bool {
        self.buckets.remove(bucket_key).is_some()
    }
}

impl Default for BucketState {
    fn default() -> Self {
        Self::new()
    }
}

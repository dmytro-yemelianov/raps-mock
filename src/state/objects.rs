// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// OSS object information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub bucket_key: String,
    pub object_key: String,
    pub object_id: String,
    pub sha1: String,
    pub size: u64,
    pub content_type: String,
    pub location: String,
}

/// OSS object state
pub struct ObjectState {
    /// Map of bucket_key -> objects
    objects: DashMap<String, DashMap<String, ObjectInfo>>,
}

impl ObjectState {
    pub fn new() -> Self {
        Self {
            objects: DashMap::new(),
        }
    }

    /// Upload an object
    pub fn upload_object(
        &self,
        bucket_key: String,
        object_key: String,
        size: u64,
        content_type: Option<String>,
    ) -> ObjectInfo {
        let object_id = format!("urn:adsk.objects:os.object:{}/{}", bucket_key, object_key);
        let object = ObjectInfo {
            bucket_key: bucket_key.clone(),
            object_key: object_key.clone(),
            object_id: object_id.clone(),
            sha1: format!("sha1_{}", uuid::Uuid::new_v4()),
            size,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            location: format!(
                "https://developer.api.autodesk.com/oss/v2/buckets/{}/objects/{}",
                bucket_key, object_key
            ),
        };

        let bucket_objects = self.objects.entry(bucket_key).or_default();
        bucket_objects.insert(object_key, object.clone());
        object
    }

    /// Get an object
    pub fn get_object(&self, bucket_key: &str, object_key: &str) -> Option<ObjectInfo> {
        self.objects
            .get(bucket_key)?
            .get(object_key)
            .map(|o| o.clone())
    }

    /// List objects in a bucket
    pub fn list_objects(&self, bucket_key: &str) -> Vec<ObjectInfo> {
        self.objects
            .get(bucket_key)
            .map(|bucket_objects| bucket_objects.iter().map(|o| o.value().clone()).collect())
            .unwrap_or_default()
    }

    /// Delete an object
    pub fn delete_object(&self, bucket_key: &str, object_key: &str) -> bool {
        self.objects
            .get(bucket_key)
            .and_then(|bucket_objects| bucket_objects.remove(object_key))
            .is_some()
    }
}

impl Default for ObjectState {
    fn default() -> Self {
        Self::new()
    }
}

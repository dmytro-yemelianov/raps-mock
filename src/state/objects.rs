// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// OSS object information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    db: Arc<Db>,
}

impl ObjectState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Upload an object
    pub fn upload_object(
        &self,
        bucket_key: String,
        object_key: String,
        size: u64,
        content_type: Option<String>,
    ) -> crate::error::Result<ObjectInfo> {
        let object_id = format!("urn:adsk.objects:os.object:{}/{}", bucket_key, object_key);
        let object = ObjectInfo {
            bucket_key: bucket_key.clone(),
            object_key: object_key.clone(),
            object_id,
            sha1: format!("sha1_{}", uuid::Uuid::new_v4()),
            size,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            location: format!(
                "https://developer.api.autodesk.com/oss/v2/buckets/{}/objects/{}",
                bucket_key, object_key
            ),
        };

        let conn = self.db.conn();
        conn.execute(
            "INSERT OR REPLACE INTO objects (bucket_key, object_key, object_id, sha1, size, content_type, location)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                object.bucket_key,
                object.object_key,
                object.object_id,
                object.sha1,
                object.size as i64,
                object.content_type,
                object.location,
            ],
        )?;
        Ok(object)
    }

    /// Get an object
    pub fn get_object(&self, bucket_key: &str, object_key: &str) -> crate::error::Result<Option<ObjectInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT bucket_key, object_key, object_id, sha1, size, content_type, location
             FROM objects WHERE bucket_key = ?1 AND object_key = ?2",
            rusqlite::params![bucket_key, object_key],
            Self::row_to_object,
        )
        .optional()?)
    }

    /// List objects in a bucket
    pub fn list_objects(&self, bucket_key: &str) -> crate::error::Result<Vec<ObjectInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT bucket_key, object_key, object_id, sha1, size, content_type, location
                 FROM objects WHERE bucket_key = ?1",
            )?;
        let items = stmt.query_map(rusqlite::params![bucket_key], Self::row_to_object)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Delete an object
    pub fn delete_object(&self, bucket_key: &str, object_key: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM objects WHERE bucket_key = ?1 AND object_key = ?2",
            rusqlite::params![bucket_key, object_key],
        )?;
        Ok(rows > 0)
    }

    /// List all objects across all buckets
    pub fn list_all(&self) -> crate::error::Result<Vec<ObjectInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT bucket_key, object_key, object_id, sha1, size, content_type, location FROM objects",
            )?;
        let items = stmt.query_map([], Self::row_to_object)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Copy an object from source to destination (server-side copy)
    pub fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        dest_bucket: &str,
        dest_key: &str,
    ) -> crate::error::Result<Option<ObjectInfo>> {
        let source = match self.get_object(src_bucket, src_key)? {
            Some(s) => s,
            None => return Ok(None),
        };
        Ok(Some(self.upload_object(
            dest_bucket.to_string(),
            dest_key.to_string(),
            source.size,
            Some(source.content_type),
        )?))
    }

    fn row_to_object(row: &rusqlite::Row<'_>) -> rusqlite::Result<ObjectInfo> {
        Ok(ObjectInfo {
            bucket_key: row.get(0)?,
            object_key: row.get(1)?,
            object_id: row.get(2)?,
            sha1: row.get(3)?,
            size: row.get::<_, i64>(4)? as u64,
            content_type: row.get(5)?,
            location: row.get(6)?,
        })
    }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// OSS bucket information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketInfo {
    pub bucket_key: String,
    pub bucket_owner: String,
    pub created_date: i64,
    pub policy_key: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub auth_id: String,
    pub access: String,
}

/// OSS bucket state
pub struct BucketState {
    db: Arc<Db>,
}

impl BucketState {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Create a new bucket
    pub fn create_bucket(&self, bucket_key: String, policy_key: String) -> crate::error::Result<BucketInfo> {
        let now = chrono::Utc::now().timestamp_millis();
        let bucket = BucketInfo {
            bucket_key: bucket_key.clone(),
            bucket_owner: "mock-owner".to_string(),
            created_date: now,
            policy_key,
            permissions: vec![],
        };
        let permissions_json = serde_json::to_string(&bucket.permissions)?;
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO buckets (bucket_key, bucket_owner, created_date, policy_key, permissions)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                bucket.bucket_key,
                bucket.bucket_owner,
                bucket.created_date,
                bucket.policy_key,
                permissions_json,
            ],
        )?;
        Ok(bucket)
    }

    /// Get a bucket by key
    pub fn get_bucket(&self, bucket_key: &str) -> crate::error::Result<Option<BucketInfo>> {
        let conn = self.db.conn();
        Ok(conn.query_row(
            "SELECT bucket_key, bucket_owner, created_date, policy_key, permissions
             FROM buckets WHERE bucket_key = ?1",
            rusqlite::params![bucket_key],
            |row| {
                let perms_json: String = row.get(4)?;
                Ok(BucketInfo {
                    bucket_key: row.get(0)?,
                    bucket_owner: row.get(1)?,
                    created_date: row.get(2)?,
                    policy_key: row.get(3)?,
                    permissions: serde_json::from_str(&perms_json).unwrap_or_default(),
                })
            },
        )
        .optional()?)
    }

    /// List all buckets
    pub fn list_buckets(&self) -> crate::error::Result<Vec<BucketInfo>> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT bucket_key, bucket_owner, created_date, policy_key, permissions FROM buckets",
            )?;
        let items = stmt.query_map([], |row| {
            let perms_json: String = row.get(4)?;
            Ok(BucketInfo {
                bucket_key: row.get(0)?,
                bucket_owner: row.get(1)?,
                created_date: row.get(2)?,
                policy_key: row.get(3)?,
                permissions: serde_json::from_str(&perms_json).unwrap_or_default(),
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    /// Delete a bucket
    pub fn delete_bucket(&self, bucket_key: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn
            .execute(
                "DELETE FROM buckets WHERE bucket_key = ?1",
                rusqlite::params![bucket_key],
            )?;
        Ok(rows > 0)
    }
}

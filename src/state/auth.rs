// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::state::db::Db;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub client_id: String,
}

/// OAuth authentication state
pub struct AuthState {
    db: Arc<Db>,
}

impl AuthState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let now = Self::current_timestamp();
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR IGNORE INTO tokens (client_id, access_token, token_type, expires_in, expires_at, refresh_token, scope)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "mock-3leg-client",
                "mock-3leg-token",
                "Bearer",
                86400_i64 * 365,
                (now + 86400 * 365) as i64,
                Option::<String>::None,
                "data:read data:write data:create data:search bucket:create bucket:read bucket:update bucket:delete account:read account:write user:read user:write user-profile:read viewables:read code:all openid",
            ],
        ).expect("failed to seed auth token");
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Generate a new access token
    pub fn generate_token(
        &self,
        client_id: &str,
        expires_in: u64,
        scope: Option<String>,
    ) -> TokenInfo {
        let now = Self::current_timestamp();
        let expires_at = now + expires_in;

        let token = TokenInfo {
            access_token: format!("mock_token_{}_{}", client_id, now),
            token_type: "Bearer".to_string(),
            expires_in,
            expires_at,
            refresh_token: Some(format!("mock_refresh_{}_{}", client_id, now)),
            scope,
            client_id: client_id.to_string(),
        };

        let conn = self.db.conn();
        // Upsert: replace old token for this client
        conn.execute(
            "INSERT INTO tokens (client_id, access_token, token_type, expires_in, expires_at, refresh_token, scope)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(client_id) DO UPDATE SET
                access_token = excluded.access_token,
                token_type = excluded.token_type,
                expires_in = excluded.expires_in,
                expires_at = excluded.expires_at,
                refresh_token = excluded.refresh_token,
                scope = excluded.scope",
            rusqlite::params![
                token.client_id,
                token.access_token,
                token.token_type,
                token.expires_in as i64,
                token.expires_at as i64,
                token.refresh_token,
                token.scope,
            ],
        ).expect("failed to insert token");

        token
    }

    /// Get token info for a client
    pub fn get_token(&self, client_id: &str) -> Option<TokenInfo> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT client_id, access_token, token_type, expires_in, expires_at, refresh_token, scope
             FROM tokens WHERE client_id = ?1",
            rusqlite::params![client_id],
            |row| {
                Ok(TokenInfo {
                    client_id: row.get(0)?,
                    access_token: row.get(1)?,
                    token_type: row.get(2)?,
                    expires_in: row.get::<_, i64>(3)? as u64,
                    expires_at: row.get::<_, i64>(4)? as u64,
                    refresh_token: row.get(5)?,
                    scope: row.get(6)?,
                })
            },
        )
        .optional()
        .expect("failed to query token")
    }

    /// Validate an access token - O(1) lookup via UNIQUE index
    pub fn validate_token(&self, token: &str) -> bool {
        let now = Self::current_timestamp();
        let conn = self.db.conn();
        conn.query_row(
            "SELECT expires_at FROM tokens WHERE access_token = ?1",
            rusqlite::params![token],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .expect("failed to validate token")
        .map(|expires_at| (expires_at as u64) > now)
        .unwrap_or(false)
    }

    /// Revoke a token
    pub fn revoke_token(&self, token: &str) {
        let conn = self.db.conn();
        conn.execute(
            "DELETE FROM tokens WHERE access_token = ?1",
            rusqlite::params![token],
        )
        .expect("failed to revoke token");
    }
}

use rusqlite::OptionalExtension;

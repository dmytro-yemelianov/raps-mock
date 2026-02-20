// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
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
    /// Map of client_id -> token info
    tokens_by_client: DashMap<String, TokenInfo>,
    /// Index: access_token -> client_id for O(1) token validation
    token_index: DashMap<String, String>,
}

impl AuthState {
    pub fn new() -> Self {
        let state = Self {
            tokens_by_client: DashMap::new(),
            token_index: DashMap::new(),
        };

        // Pre-seed well-known 3-legged mock token so Docker/test environments
        // can use `raps auth login --token mock-3leg-token` and have all
        // 3-legged-auth endpoints accept it without a real OAuth flow.
        let now = Self::current_timestamp();
        let three_leg = TokenInfo {
            access_token: "mock-3leg-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 86400 * 365,
            expires_at: now + 86400 * 365,
            refresh_token: None,
            scope: Some("data:read data:write data:create data:search bucket:create bucket:read bucket:update bucket:delete account:read account:write user:read user:write user-profile:read viewables:read code:all openid".to_string()),
            client_id: "mock-3leg-client".to_string(),
        };
        state
            .token_index
            .insert(three_leg.access_token.clone(), three_leg.client_id.clone());
        state
            .tokens_by_client
            .insert(three_leg.client_id.clone(), three_leg);

        state
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

        // Remove old token from index if exists
        if let Some(old_token) = self.tokens_by_client.get(client_id) {
            self.token_index.remove(&old_token.access_token);
        }

        let token = TokenInfo {
            access_token: format!("mock_token_{}_{}", client_id, now),
            token_type: "Bearer".to_string(),
            expires_in,
            expires_at,
            refresh_token: Some(format!("mock_refresh_{}_{}", client_id, now)),
            scope,
            client_id: client_id.to_string(),
        };

        // Update both maps
        self.token_index
            .insert(token.access_token.clone(), client_id.to_string());
        self.tokens_by_client
            .insert(client_id.to_string(), token.clone());
        token
    }

    /// Get token info for a client
    pub fn get_token(&self, client_id: &str) -> Option<TokenInfo> {
        self.tokens_by_client.get(client_id).map(|t| t.clone())
    }

    /// Validate an access token - O(1) lookup
    pub fn validate_token(&self, token: &str) -> bool {
        let now = Self::current_timestamp();

        self.token_index
            .get(token)
            .and_then(|client_id| self.tokens_by_client.get(client_id.value()))
            .map(|token_info| token_info.expires_at > now)
            .unwrap_or(false)
    }

    /// Revoke a token
    pub fn revoke_token(&self, token: &str) {
        if let Some((_, client_id)) = self.token_index.remove(token) {
            self.tokens_by_client.remove(&client_id);
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::config::{MockMode, MockServerConfig};
use crate::error::Result;
use crate::openapi::OpenApiParser;
use crate::state::StateManager;
use axum::Router;
use tokio::net::TcpListener;

mod router;

/// Mock server for APS APIs
pub struct MockServer {
    #[allow(dead_code)]
    config: MockServerConfig,
    #[allow(dead_code)]
    state: Option<StateManager>,
    router: Router,
}

impl MockServer {
    /// Create a new mock server with the given configuration
    pub async fn new(config: MockServerConfig) -> Result<Self> {
        // Parse OpenAPI specs
        let specs = OpenApiParser::parse_directory(&config.openapi_dir)?;
        tracing::info!("Parsed {} OpenAPI specifications", specs.len());

        // Extract all routes
        let mut all_routes = Vec::new();
        for (name, spec) in specs {
            let routes = OpenApiParser::extract_routes(&spec);
            tracing::debug!("Extracted {} routes from {}", routes.len(), name);
            all_routes.extend(routes);
        }

        // Create state manager if in stateful mode
        let state = if config.mode == MockMode::Stateful {
            let state_manager = StateManager::new();
            if let Some(ref state_file) = config.state_file {
                state_manager.load_from_file(state_file)?;
            }
            Some(state_manager)
        } else {
            None
        };

        // Build router using submodule
        let router = crate::server::router::build_router(all_routes, state.clone())?;

        Ok(Self {
            config,
            state,
            router,
        })
    }

    /// Start the server and listen on the given address
    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Server listening on {}", addr);

        axum::serve(listener, self.router.clone())
            .await
            .map_err(|e| crate::error::MockError::Io(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    /// Expose a clone of the router for embedding or tests
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

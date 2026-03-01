// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::middleware::SimulationConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Mock server operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MockMode {
    /// Stateless mode: return fixed responses from OpenAPI examples
    Stateless,
    /// Stateful mode: maintain in-memory state and return dynamic responses
    #[default]
    Stateful,
}

impl std::str::FromStr for MockMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stateless" => Ok(MockMode::Stateless),
            "stateful" => Ok(MockMode::Stateful),
            _ => Err(format!(
                "Invalid mode: {}. Use 'stateless' or 'stateful'",
                s
            )),
        }
    }
}

/// Configuration for the mock server
#[derive(Debug, Clone)]
pub struct MockServerConfig {
    /// Server operation mode
    pub mode: MockMode,
    /// Path to OpenAPI specifications directory
    pub openapi_dir: PathBuf,
    /// Optional path to SQLite database file for persistent state
    pub db_path: Option<PathBuf>,
    /// Enable verbose logging
    pub verbose: bool,
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Simulation configuration for chaos testing
    pub simulation: SimulationConfig,
    /// Path to fixture file or directory to preload on startup
    pub fixtures: Option<PathBuf>,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        Self {
            mode: MockMode::default(),
            openapi_dir: PathBuf::from("../aps-sdk-openapi"),
            db_path: None,
            verbose: false,
            host: "0.0.0.0".to_string(),
            port: 3000,
            simulation: SimulationConfig::default(),
            fixtures: None,
        }
    }
}

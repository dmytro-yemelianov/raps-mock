// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use thiserror::Error;

/// Errors that can occur in the mock server
#[derive(Error, Debug)]
pub enum MockError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid OpenAPI specification: {0}")]
    InvalidSpec(String),

    #[error("State persistence error: {0}")]
    StatePersistence(String),
}

pub type Result<T> = std::result::Result<T, MockError>;

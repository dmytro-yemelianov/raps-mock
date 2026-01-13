// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! raps-mock: Mock server for Autodesk Platform Services (APS) APIs
//!
//! This library provides a mock server that can automatically generate routes
//! from OpenAPI 3.0 specifications and serve mock responses.

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod openapi;
pub mod server;
pub mod state;

pub use config::{MockMode, MockServerConfig};
pub use error::{MockError, Result};
pub use server::MockServer;

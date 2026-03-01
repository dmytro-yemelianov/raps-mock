// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Simulation middleware for chaos testing: latency injection, error rates,
//! and configurable per-endpoint fault injection.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Simulation configuration for chaos testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Global latency to add to every request (milliseconds)
    pub latency_ms: u64,
    /// Random jitter range added on top of latency (0 to jitter_ms)
    pub jitter_ms: u64,
    /// Probability of returning an error (0.0 = never, 1.0 = always)
    pub error_rate: f64,
    /// HTTP status code to return on simulated errors
    pub error_status: u16,
    /// Per-path overrides (path prefix -> config override)
    pub overrides: Vec<PathOverride>,
}

/// Per-path simulation override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathOverride {
    /// Path prefix to match (e.g., "/modelderivative" or "/oss/v2")
    pub path_prefix: String,
    /// Override latency (None = use global)
    pub latency_ms: Option<u64>,
    /// Override jitter (None = use global)
    pub jitter_ms: Option<u64>,
    /// Override error rate (None = use global)
    pub error_rate: Option<f64>,
    /// Override error status (None = use global)
    pub error_status: Option<u16>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            error_rate: 0.0,
            error_status: 500,
            overrides: Vec::new(),
        }
    }
}

impl SimulationConfig {
    /// Check if simulation has any effect
    pub fn is_active(&self) -> bool {
        self.latency_ms > 0 || self.jitter_ms > 0 || self.error_rate > 0.0
    }

    /// Resolve effective config for a given path (override > global)
    fn resolve(&self, path: &str) -> ResolvedConfig {
        // Find the most specific (longest) matching override
        let best_override = self
            .overrides
            .iter()
            .filter(|o| path.starts_with(&o.path_prefix))
            .max_by_key(|o| o.path_prefix.len());

        match best_override {
            Some(ov) => ResolvedConfig {
                latency_ms: ov.latency_ms.unwrap_or(self.latency_ms),
                jitter_ms: ov.jitter_ms.unwrap_or(self.jitter_ms),
                error_rate: ov.error_rate.unwrap_or(self.error_rate),
                error_status: ov.error_status.unwrap_or(self.error_status),
            },
            None => ResolvedConfig {
                latency_ms: self.latency_ms,
                jitter_ms: self.jitter_ms,
                error_rate: self.error_rate,
                error_status: self.error_status,
            },
        }
    }
}

struct ResolvedConfig {
    latency_ms: u64,
    jitter_ms: u64,
    error_rate: f64,
    error_status: u16,
}

/// Axum middleware that injects latency and errors based on SimulationConfig
pub async fn simulation_middleware(
    config: Option<axum::Extension<Arc<SimulationConfig>>>,
    request: Request,
    next: Next,
) -> Response {
    let config = match config {
        Some(axum::Extension(ref c)) if c.is_active() => c,
        _ => return next.run(request).await,
    };

    let path = request.uri().path().to_string();

    // Skip simulation for health checks
    if path == "/health" {
        return next.run(request).await;
    }

    let resolved = config.resolve(&path);

    // Inject latency
    let total_delay = if resolved.latency_ms > 0 || resolved.jitter_ms > 0 {
        let jitter = if resolved.jitter_ms > 0 {
            rand::thread_rng().gen_range(0..=resolved.jitter_ms)
        } else {
            0
        };
        resolved.latency_ms + jitter
    } else {
        0
    };

    if total_delay > 0 {
        tokio::time::sleep(Duration::from_millis(total_delay)).await;
    }

    // Inject errors
    if resolved.error_rate > 0.0 {
        let roll: f64 = rand::thread_rng().gen_range(0.0..1.0);
        if roll < resolved.error_rate {
            let status =
                StatusCode::from_u16(resolved.error_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            tracing::debug!(
                path = %path,
                status = %status,
                "Simulation: injecting error"
            );

            return Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .header("X-Raps-Mock-Simulated", "true")
                .body(
                    serde_json::json!({
                        "developerMessage": format!(
                            "Simulated {} error (error_rate={:.0}%)",
                            status.as_u16(),
                            resolved.error_rate * 100.0
                        ),
                        "errorCode": "MOCK-SIM-001",
                        "simulated": true
                    })
                    .to_string()
                    .into(),
                )
                .expect("Failed to build simulated error response");
        }
    }

    next.run(request).await
}

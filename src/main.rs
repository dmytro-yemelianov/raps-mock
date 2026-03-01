// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use clap::Parser;
use raps_mock::{MockMode, MockServer, MockServerConfig, SimulationConfig};
use std::path::PathBuf;
use tracing::{Level, info};

#[derive(Parser)]
#[command(name = "raps-mock")]
#[command(about = "Mock server for Autodesk Platform Services (APS) APIs")]
#[command(version)]
struct Cli {
    /// Server port
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Server host
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// Operation mode: stateless or stateful
    #[arg(short, long, default_value = "stateful")]
    mode: MockMode,

    /// Path to OpenAPI specifications directory
    #[arg(long, default_value = "../aps-sdk-openapi")]
    openapi_dir: PathBuf,

    /// Path to SQLite database file for persistent state (omit for in-memory)
    #[arg(long)]
    db: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Add simulated latency to every request (milliseconds)
    #[arg(long, default_value = "0")]
    latency: u64,

    /// Add random jitter on top of latency (0 to N milliseconds)
    #[arg(long, default_value = "0")]
    jitter: u64,

    /// Probability of returning a simulated error (0.0 to 1.0)
    #[arg(long, default_value = "0.0")]
    error_rate: f64,

    /// HTTP status code for simulated errors
    #[arg(long, default_value = "500")]
    error_status: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    info!("Starting raps-mock server");
    info!("Mode: {:?}", cli.mode);
    info!("OpenAPI directory: {}", cli.openapi_dir.display());
    if let Some(ref db) = cli.db {
        info!("Database: {}", db.display());
    }

    let simulation = SimulationConfig {
        latency_ms: cli.latency,
        jitter_ms: cli.jitter,
        error_rate: cli.error_rate,
        error_status: cli.error_status,
        overrides: Vec::new(),
    };

    if simulation.is_active() {
        info!("Simulation enabled: latency={}ms jitter={}ms error_rate={:.0}% error_status={}",
            simulation.latency_ms, simulation.jitter_ms,
            simulation.error_rate * 100.0, simulation.error_status);
    }

    let config = MockServerConfig {
        mode: cli.mode,
        openapi_dir: cli.openapi_dir,
        db_path: cli.db,
        verbose: cli.verbose,
        host: cli.host.clone(),
        port: cli.port,
        simulation,
    };

    let server = MockServer::new(config).await?;
    let addr = format!("{}:{}", cli.host, cli.port);
    server.start(&addr).await?;

    Ok(())
}

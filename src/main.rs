// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use clap::Parser;
use raps_mock::{MockMode, MockServer, MockServerConfig};
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

    /// Path to state persistence file (optional)
    #[arg(long)]
    state_file: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
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

    let config = MockServerConfig {
        mode: cli.mode,
        openapi_dir: cli.openapi_dir,
        state_file: cli.state_file,
        verbose: cli.verbose,
        host: cli.host.clone(),
        port: cli.port,
    };

    let server = MockServer::new(config).await?;
    let addr = format!("{}:{}", cli.host, cli.port);
    info!("Server listening on {}", addr);
    server.start(&addr).await?;

    Ok(())
}

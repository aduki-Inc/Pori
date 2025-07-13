use anyhow::Result;
use clap::Parser;
use tracing::info;

use pori::{config::cli::CliArgs, config::settings::AppSettings, logging, run_application};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli_args = CliArgs::parse();

    // Validate arguments
    cli_args.validate()?;

    // Initialize logging
    logging::init(&cli_args.log_level)?;

    // Print startup banner
    println!("Starting Pori v{}", env!("CARGO_PKG_VERSION"));
    info!("Starting pori application");
    if let Some(ref url) = cli_args.url {
        info!("WebSocket URL: {}", url);
    }
    info!(
        "Local server: {}://localhost:{}",
        cli_args.protocol, cli_args.port
    );
    if !cli_args.no_dashboard {
        info!(
            "Dashboard server: http://localhost:{}",
            cli_args.dashboard_port
        );
    }
    info!("Log level: {}", cli_args.log_level);

    // Create application settings from CLI arguments
    let settings = AppSettings::from_cli(cli_args)?;

    // Run application
    run_application(settings).await?;

    Ok(())
}

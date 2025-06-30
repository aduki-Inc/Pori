use anyhow::Result;
use clap::Parser;
use tracing::info;

use pori::{
    config::cli::CliArgs, config::settings::AppSettings, logging, run_application,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli_args = CliArgs::parse();

    // Validate arguments
    cli_args.validate()?;

    // Initialize logging
    logging::init(&cli_args.log_level)?;

    info!("Starting pori");
    if let Some(ref url) = cli_args.url {
        info!("URL: {}", url);
    }
    info!(
        "Local server: {}://localhost:{}",
        cli_args.protocol, cli_args.port
    );
    if !cli_args.no_dashboard {
        info!("Dashboard port: {}", cli_args.dashboard_port);
    }

    // Create application settings from CLI arguments
    let settings = AppSettings::from_cli(cli_args)?;

    // Run application
    run_application(settings).await?;

    Ok(())
}

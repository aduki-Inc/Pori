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

    // Create application settings from CLI arguments
    let settings = AppSettings::from_cli(cli_args)?;

    // Initialize logging with show_context from settings
    logging::init_with_context(&settings.logging.level, settings.logging.show_context)?;

    // Print startup banner
    println!("Starting Pori v{}", env!("CARGO_PKG_VERSION"));
    info!("Starting pori application");
    info!("WebSocket URL: {}", settings.websocket.url);
    info!("Local server: {}", settings.local_server.url);
    if !settings.no_dashboard {
        info!("Dashboard server: {}", settings.dashboard_url());
    }
    info!("Log level: {}", settings.logging.level);

    // ...existing code...

    // Run application
    run_application(settings).await?;

    Ok(())
}

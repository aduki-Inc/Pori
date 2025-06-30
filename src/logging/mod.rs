use anyhow::Result;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging system
pub fn init(log_level: &str) -> Result<()> {
    // Parse log level
    let level = parse_log_level(log_level)?;

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("tunnel_client={level},tower=warn,hyper=warn")));

    // Setup tracing subscriber
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .with_ansi(atty::is(atty::Stream::Stdout))
                .compact(),
        )
        .with(env_filter)
        .init();

    Ok(())
}

/// Parse log level string to tracing Level
fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "error" => Ok(Level::ERROR),
        "warn" => Ok(Level::WARN),
        "info" => Ok(Level::INFO),
        "debug" => Ok(Level::DEBUG),
        "trace" => Ok(Level::TRACE),
        _ => anyhow::bail!("Invalid log level: {}", level),
    }
}

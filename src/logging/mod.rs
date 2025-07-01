use anyhow::Result;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::Writer, time::FormatTime, FormatEvent, FormatFields},
    prelude::*,
    registry::LookupSpan,
    EnvFilter,
};

/// Custom time formatter for human-readable timestamps
pub struct HumanTime;

impl FormatTime for HumanTime {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

/// Custom event formatter that replaces INFO with PROXY/LOCAL when appropriate
pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();

        // Write timestamp
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m")?; // Dim
        }
        HumanTime.format_time(&mut writer)?;
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[0m")?; // Reset
        }
        write!(writer, "  ")?;

        // Determine what to show as the level indicator
        let level_indicator = match metadata.target() {
            "PROXY" => "PROXY",
            "LOCAL" => "LOCAL",
            _ => {
                // For regular logs, show the level
                match *metadata.level() {
                    Level::ERROR => "ERROR",
                    Level::WARN => " WARN",
                    Level::INFO => " INFO",
                    Level::DEBUG => "DEBUG",
                    Level::TRACE => "TRACE",
                }
            }
        };

        // Write level with color if supported
        if writer.has_ansi_escapes() {
            let level_color = match metadata.target() {
                "PROXY" => "\x1b[35m", // Magenta for PROXY
                "LOCAL" => "\x1b[36m", // Cyan for LOCAL
                _ => match *metadata.level() {
                    Level::ERROR => "\x1b[31m", // Red
                    Level::WARN => "\x1b[33m",  // Yellow
                    Level::INFO => "\x1b[32m",  // Green
                    Level::DEBUG => "\x1b[34m", // Blue
                    Level::TRACE => "\x1b[37m", // White
                },
            };
            write!(writer, "{}{}\x1b[0m", level_color, level_indicator)?;
        } else {
            write!(writer, "{}", level_indicator)?;
        }

        // For non-PROXY/LOCAL logs, show the target
        if !matches!(metadata.target(), "PROXY" | "LOCAL") {
            write!(writer, " {}", metadata.target())?;
        }

        write!(writer, ": ")?;

        // Write span context
        ctx.visit_spans(|span| {
            write!(writer, "{}: ", span.name()).unwrap();
            Ok(())
        })?;

        // Write the actual message
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)?;
        Ok(())
    }
}

/// Initialize logging system
pub fn init(log_level: &str) -> Result<()> {
    // Parse log level
    let level = parse_log_level(log_level)?;

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| match level {
        Level::INFO => EnvFilter::new("pori=info,PROXY=info,LOCAL=info,tower=warn,hyper=warn"),
        Level::DEBUG => EnvFilter::new("pori=debug,PROXY=debug,LOCAL=debug,tower=warn,hyper=warn"),
        Level::TRACE => EnvFilter::new("pori=trace,PROXY=trace,LOCAL=trace,tower=warn,hyper=warn"),
        Level::WARN => EnvFilter::new("pori=warn,PROXY=warn,LOCAL=warn,tower=warn,hyper=warn"),
        Level::ERROR => EnvFilter::new("pori=error,PROXY=error,LOCAL=error,tower=warn,hyper=warn"),
    });

    // Setup tracing subscriber with custom formatter
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .event_format(CustomFormatter)
                .with_ansi(atty::is(atty::Stream::Stdout)),
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
        "proxy" => Ok(Level::INFO), // PROXY maps to INFO level
        "local" => Ok(Level::INFO), // LOCAL maps to INFO level
        "debug" => Ok(Level::DEBUG),
        "trace" => Ok(Level::TRACE),
        _ => anyhow::bail!("Invalid log level: {}", level),
    }
}

/// Macro for proxy-related logging (WebSocket server/client)
#[macro_export]
macro_rules! proxy_log {
    ($($arg:tt)*) => {
        tracing::info!(target: "PROXY", $($arg)*)
    };
}

/// Macro for local HTTP server logging
#[macro_export]
macro_rules! local_log {
    ($($arg:tt)*) => {
        tracing::info!(target: "LOCAL", $($arg)*)
    };
}

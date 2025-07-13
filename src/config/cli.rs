use anyhow::{Context, Result};
use clap::Parser;
use url::Url;

/// Command line interface arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// WebSocket URL for cloud/proxy connection
    #[arg(long, env = "PORI_URL")]
    pub url: Option<Url>,

    /// Access token for authentication
    #[arg(long, env = "PORI_TOKEN")]
    pub token: Option<String>,

    /// Protocol for local server (http or https)
    #[arg(long, default_value = "http", env = "PORI_PROTOCOL")]
    pub protocol: String,

    /// Port for local server
    #[arg(long, default_value_t = 3000, env = "PORI_PORT")]
    pub port: u16,

    /// Port for dashboard server
    #[arg(long, default_value_t = 7616, env = "PORI_DASHBOARD_PORT")]
    pub dashboard_port: u16,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    /// Configuration file path (TOML or JSON)
    #[arg(long, env = "PORI_CONFIG")]
    pub config: Option<String>,

    /// YAML configuration file path
    #[arg(long, env = "PORI_YML")]
    pub yml: Option<String>,

    /// Disable dashboard server
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_dashboard: bool,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,

    /// Reconnection attempts (0 = infinite)
    #[arg(long, default_value_t = 0)]
    pub max_reconnects: u32,

    /// Verify SSL certificates for a local server
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub verify_ssl: bool,

    /// Maximum connections to a local server
    #[arg(long, default_value_t = 10)]
    pub max_connections: usize,

    /// HTTP version for local server communication (auto, http1, http2)
    #[arg(long, default_value = "http1", env = "PORI_HTTP_VERSION")]
    pub http_version: String,
}

impl CliArgs {
    /// Parse and validate command line arguments
    pub fn parse_and_validate() -> Result<Self> {
        let args = Self::parse();
        args.validate()?;
        Ok(args)
    }

    /// Validate CLI arguments
    pub fn validate(&self) -> Result<()> {
        // If no config file is specified, url and token are required
        if self.config.is_none() && self.yml.is_none() {
            if self.url.is_none() {
                anyhow::bail!("WebSocket URL is required when not using a configuration file");
            }
            if self.token.is_none() {
                anyhow::bail!("Access token is required when not using a configuration file");
            }
        }

        // Validate WebSocket URL scheme if provided
        if let Some(ref url) = self.url {
            if !matches!(url.scheme(), "ws" | "wss") {
                anyhow::bail!(
                    "WebSocket URL must use ws:// or wss:// scheme, got: {}",
                    url.scheme()
                );
            }
        }

        // Validate local server protocol
        if !matches!(self.protocol.to_lowercase().as_str(), "http" | "https") {
            anyhow::bail!("Protocol must be 'http' or 'https', got: {}", self.protocol);
        }

        // Validate local server port
        if self.port == 0 {
            anyhow::bail!("Local server port must be greater than 0");
        }

        // Validate access token if provided
        if let Some(ref token) = self.token {
            if token.trim().is_empty() {
                anyhow::bail!("Access token cannot be empty");
            }
        }

        // Validate dashboard port
        if self.dashboard_port == 0 {
            anyhow::bail!("Dashboard port must be greater than 0");
        }

        // Validate timeout
        if self.timeout == 0 {
            anyhow::bail!("Timeout must be greater than 0 seconds");
        }

        // Validate max connections
        if self.max_connections == 0 {
            anyhow::bail!("Max connections must be greater than 0");
        }

        // Validate HTTP version
        if !matches!(
            self.http_version.to_lowercase().as_str(),
            "auto" | "http1" | "http2"
        ) {
            anyhow::bail!(
                "Invalid HTTP version: {}. Must be one of: auto, http1, http2",
                self.http_version
            );
        }

        // Validate log level
        if !matches!(
            self.log_level.to_lowercase().as_str(),
            "error" | "warn" | "info" | "debug" | "trace"
        ) {
            anyhow::bail!(
                "Invalid log level: {}. Must be one of: error, warn, info, debug, trace",
                self.log_level
            );
        }

        Ok(())
    }

    /// Get the host part of the WebSocket URL
    pub fn websocket_host(&self) -> Result<String> {
        if let Some(ref url) = self.url {
            url.host_str()
                .map(|h| h.to_string())
                .context("WebSocket URL must have a valid host")
        } else {
            anyhow::bail!("WebSocket URL isn't configured")
        }
    }

    /// Get the host part of the local server URL
    pub fn local_host(&self) -> Result<String> {
        Ok("localhost".to_string())
    }

    /// Get the complete local server URL
    pub fn local_url(&self) -> Result<Url> {
        let url_string = format!("{}://localhost:{}", self.protocol, self.port);
        Url::parse(&url_string).context("Failed to construct local server URL")
    }

    /// Check if TLS is required for WebSocket connection
    pub fn requires_tls(&self) -> bool {
        self.url.as_ref().is_some_and(|url| url.scheme() == "wss")
    }

    /// Check if local server uses HTTPS
    pub fn local_uses_https(&self) -> bool {
        self.protocol.to_lowercase() == "https"
    }

    /// Get the WebSocket URL (panics if not set)
    pub fn get_url(&self) -> &Url {
        self.url
            .as_ref()
            .expect("URL should be validated before use")
    }

    /// Get the access token (panics if not set)
    pub fn get_token(&self) -> &str {
        self.token
            .as_ref()
            .expect("Token should be validated before use")
    }
}

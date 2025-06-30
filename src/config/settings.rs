use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use super::cli::CliArgs;

/// Application settings derived from CLI arguments and configuration files
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub websocket: WebSocketSettings,
    pub local_server: LocalServerSettings,
    pub dashboard: DashboardSettings,
    pub logging: LoggingSettings,
    pub no_dashboard: bool,
}

/// WebSocket connection settings
#[derive(Debug, Clone)]
pub struct WebSocketSettings {
    pub url: Url,
    pub token: String,
    pub timeout: Duration,
    pub max_reconnects: u32,
    pub requires_tls: bool,
    pub ping_interval: Duration,
    pub pong_timeout: Duration,
}

/// Local server configuration
#[derive(Debug, Clone)]
pub struct LocalServerSettings {
    pub url: Url,
    pub timeout: Duration,
    pub verify_ssl: bool,
    pub max_connections: usize,
    pub keep_alive: Duration,
    pub connect_timeout: Duration,
}

/// Dashboard server settings
#[derive(Debug, Clone)]
pub struct DashboardSettings {
    pub port: u16,
    pub bind_address: String,
    pub enable_cors: bool,
    pub static_file_cache: bool,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub format: LogFormat,
    pub enable_color: bool,
}

/// Log output format
#[derive(Debug, Clone)]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

/// Configuration file structure (optional)
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigFile {
    pub websocket: Option<WebSocketConfig>,
    pub local_server: Option<LocalServerConfig>,
    pub dashboard: Option<DashboardConfig>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebSocketConfig {
    pub url: Option<String>,
    pub token: Option<String>,
    pub timeout: Option<u64>,
    pub max_reconnects: Option<u32>,
    pub ping_interval: Option<u64>,
    pub pong_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalServerConfig {
    pub url: Option<String>,
    pub timeout: Option<u64>,
    pub verify_ssl: Option<bool>,
    pub max_connections: Option<usize>,
    pub keep_alive: Option<u64>,
    pub connect_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardConfig {
    pub port: Option<u16>,
    pub bind_address: Option<String>,
    pub enable_cors: Option<bool>,
    pub static_file_cache: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub format: Option<String>,
    pub enable_color: Option<bool>,
}

impl AppSettings {
    /// Create settings from CLI arguments
    pub fn from_cli(cli: CliArgs) -> Result<Self> {
        // Load config file if specified
        let config_file = if let Some(yml_path) = &cli.yml {
            Self::load_config_file(yml_path)?
        } else if let Some(config_path) = &cli.config {
            Self::load_config_file(config_path)?
        } else {
            Self::try_load_default_config()?
        };

        // Get URL from CLI or config file
        let url = cli
            .url
            .clone()
            .or_else(|| {
                config_file
                    .websocket
                    .as_ref()
                    .and_then(|ws| ws.url.as_ref())
                    .and_then(|url_str| url_str.parse().ok())
            })
            .context("WebSocket URL must be provided via CLI arguments or configuration file")?;

        // Get token from CLI or config file
        let token = cli
            .token
            .clone()
            .or_else(|| {
                config_file
                    .websocket
                    .as_ref()
                    .and_then(|ws| ws.token.clone())
            })
            .context("Access token must be provided via CLI arguments or configuration file")?;

        // Determine if TLS is required based on the final URL
        let requires_tls = url.scheme() == "wss";

        Ok(Self {
            websocket: WebSocketSettings {
                url,
                token,
                timeout: Duration::from_secs(
                    config_file
                        .websocket
                        .as_ref()
                        .and_then(|ws| ws.timeout)
                        .unwrap_or(cli.timeout),
                ),
                max_reconnects: config_file
                    .websocket
                    .as_ref()
                    .and_then(|ws| ws.max_reconnects)
                    .unwrap_or(cli.max_reconnects),
                requires_tls,
                ping_interval: Duration::from_secs(
                    config_file
                        .websocket
                        .as_ref()
                        .and_then(|ws| ws.ping_interval)
                        .unwrap_or(30),
                ),
                pong_timeout: Duration::from_secs(
                    config_file
                        .websocket
                        .as_ref()
                        .and_then(|ws| ws.pong_timeout)
                        .unwrap_or(10),
                ),
            },
            local_server: LocalServerSettings {
                url: config_file
                    .local_server
                    .as_ref()
                    .and_then(|ls| ls.url.as_ref())
                    .map(|url| url.parse())
                    .transpose()
                    .context("Invalid local server URL in config file")?
                    .unwrap_or_else(|| cli.local_url().expect("Failed to construct local URL")),
                timeout: Duration::from_secs(
                    config_file
                        .local_server
                        .as_ref()
                        .and_then(|ls| ls.timeout)
                        .unwrap_or(cli.timeout),
                ),
                verify_ssl: config_file
                    .local_server
                    .as_ref()
                    .and_then(|ls| ls.verify_ssl)
                    .unwrap_or(cli.verify_ssl),
                max_connections: config_file
                    .local_server
                    .as_ref()
                    .and_then(|ls| ls.max_connections)
                    .unwrap_or(cli.max_connections),
                keep_alive: Duration::from_secs(
                    config_file
                        .local_server
                        .as_ref()
                        .and_then(|ls| ls.keep_alive)
                        .unwrap_or(60),
                ),
                connect_timeout: Duration::from_secs(
                    config_file
                        .local_server
                        .as_ref()
                        .and_then(|ls| ls.connect_timeout)
                        .unwrap_or(10),
                ),
            },
            dashboard: DashboardSettings {
                port: config_file
                    .dashboard
                    .as_ref()
                    .and_then(|d| d.port)
                    .unwrap_or(cli.dashboard_port),
                bind_address: config_file
                    .dashboard
                    .as_ref()
                    .and_then(|d| d.bind_address.clone())
                    .unwrap_or_else(|| "127.0.0.1".to_string()),
                enable_cors: config_file
                    .dashboard
                    .as_ref()
                    .and_then(|d| d.enable_cors)
                    .unwrap_or(true),
                static_file_cache: config_file
                    .dashboard
                    .as_ref()
                    .and_then(|d| d.static_file_cache)
                    .unwrap_or(true),
            },
            logging: LoggingSettings {
                level: config_file
                    .logging
                    .as_ref()
                    .and_then(|l| l.level.clone())
                    .unwrap_or(cli.log_level),
                format: config_file
                    .logging
                    .as_ref()
                    .and_then(|l| l.format.as_ref())
                    .map(|f| match f.as_str() {
                        "json" => LogFormat::Json,
                        "compact" => LogFormat::Compact,
                        _ => LogFormat::Pretty,
                    })
                    .unwrap_or(LogFormat::Pretty),
                enable_color: config_file
                    .logging
                    .as_ref()
                    .and_then(|l| l.enable_color)
                    .unwrap_or(true),
            },
            no_dashboard: cli.no_dashboard,
        })
    }

    /// Load configuration from specified file
    fn load_config_file(path: &str) -> Result<ConfigFile> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {path}"))?;

        if path.ends_with(".toml") {
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config file: {path}"))
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config file: {path}"))
        } else if path.ends_with(".yml") || path.ends_with(".yaml") {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML config file: {path}"))
        } else {
            // Try YAML first, then TOML, then JSON
            serde_yaml::from_str(&content)
                .or_else(|_| toml::from_str(&content))
                .or_else(|_| serde_json::from_str(&content))
                .with_context(|| {
                    format!("Failed to parse config file (tried YAML, TOML, and JSON): {path}")
                })
        }
    }

    /// Try to load default configuration files
    fn try_load_default_config() -> Result<ConfigFile> {
        // Try common config file locations
        let possible_paths = [
            "./pori.yml",
            "./pori.yaml",
            "./pori.toml",
            "./pori.json",
            "~/.pori.yml",
            "~/.pori.yaml",
            "~/.pori.toml",
            "~/.pori.json",
            "~/.config/pori/config.yml",
            "~/.config/pori/config.yaml",
            "~/.config/pori/config.toml",
            "~/.config/pori/config.json",
        ];

        for path in &possible_paths {
            let expanded_path = if path.starts_with("~/") {
                if let Some(home_dir) = dirs::home_dir() {
                    home_dir.join(&path[2..]).to_string_lossy().to_string()
                } else {
                    continue;
                }
            } else {
                path.to_string()
            };

            if std::path::Path::new(&expanded_path).exists() {
                if let Ok(config) = Self::load_config_file(&expanded_path) {
                    return Ok(config);
                }
            }
        }

        // Return empty config if no file found
        Ok(ConfigFile {
            websocket: None,
            local_server: None,
            dashboard: None,
            logging: None,
        })
    }

    /// Get dashboard bind address including port
    pub fn dashboard_address(&self) -> String {
        format!("{}:{}", self.dashboard.bind_address, self.dashboard.port)
    }

    /// Get dashboard URL for local access
    pub fn dashboard_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.dashboard.bind_address, self.dashboard.port
        )
    }

    /// Validate all settings
    pub fn validate(&self) -> Result<()> {
        // Validate WebSocket URL
        if self.websocket.url.host_str().is_none() {
            anyhow::bail!("WebSocket URL must have a valid host");
        }

        // Validate local server URL
        if self.local_server.url.host_str().is_none() {
            anyhow::bail!("Local server URL must have a valid host");
        }

        // Validate token
        if self.websocket.token.trim().is_empty() {
            anyhow::bail!("WebSocket authentication token cannot be empty");
        }

        // Validate dashboard port
        if self.dashboard.port == 0 {
            anyhow::bail!("Dashboard port must be greater than 0");
        }

        Ok(())
    }
}

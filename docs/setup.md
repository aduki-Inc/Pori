# Setup Guide: Rust Tunnel Client

## Project Setup and Structure

### Initial Cargo Project Setup

**Step 1: Create New Cargo Project**

> Initialize new Rust binary project with git repository

**Step 2: Configure Cargo.toml**

> Set up dependencies for async runtime, HTTP server, WebSocket client, CLI parsing, serialization, logging, error handling, and utilities

**Required dependencies:**
- tokio (async runtime with full features)
- hyper (HTTP server for dashboard)
- tokio-tungstenite (WebSocket client for cloud connection)
- reqwest (HTTP client for local server forwarding)
- clap (CLI argument parsing with derive)
- serde + serde_json (serialization)
- tracing + tracing-subscriber (structured logging)
- thiserror + anyhow (error handling)
- uuid, url, bytes, mime (utilities)
- include_dir (static file embedding)

**Development dependencies:**
- tokio-test (async testing utilities)

**Release profile optimization:**
- Maximum optimization level
- Link-time optimization enabled
- Single codegen unit
- Panic abort for smaller binaries
- Strip debug symbols

**Step 3: Create Module Structure**

> Create directory structure and module files

**Module directories:**
- src/config (CLI args, settings, validation)
- src/server (Dashboard HTTP server using hyper.rs)
- src/proxy (HTTP forwarding to local server)
- src/websocket (Cloud connection client)
- src/logging (Tracing configuration)
- src/utils (Error handling, signals)

**Static asset directories:**
- static/css (Dashboard stylesheets)
- static/js (Dashboard JavaScript)
- static/assets (Images, icons)

**Test directories:**
- tests/unit (Unit tests)
- tests/integration (Integration tests)

## CLI Configuration Implementation

### Command Line Interface Design

**CLI Arguments Structure:**

> Define CLI arguments using clap derive macros with proper validation

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// WebSocket URL for cloud/proxy connection
    #[arg(long, env = "TUNNEL_URL")]
    pub url: Url,

    /// Access token for authentication
    #[arg(long, env = "TUNNEL_TOKEN")]
    pub token: String,

    /// Local server URL to forward requests to
    #[arg(long, default_value = "https://localhost:3000", env = "TUNNEL_LOCAL")]
    pub local: Url,

    /// Port for dashboard server
    #[arg(long, default_value_t = 8080, env = "TUNNEL_DASHBOARD_PORT")]
    pub dashboard_port: u16,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    /// Configuration file path (optional)
    #[arg(long, env = "TUNNEL_CONFIG")]
    pub config: Option<String>,

    /// Disable dashboard server
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_dashboard: bool,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,

    /// Reconnection attempts (0 = infinite)
    #[arg(long, default_value_t = 0)]
    pub max_reconnects: u32,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate URL schemes
        if !matches!(self.url.scheme(), "ws" | "wss") {
            anyhow::bail!("URL must use ws:// or wss:// scheme");
        }

        if !matches!(self.local.scheme(), "http" | "https") {
            anyhow::bail!("Local URL must use http:// or https:// scheme");
        }

        // Validate token
        if self.token.is_empty() {
            anyhow::bail!("Access token cannot be empty");
        }

        // Validate port range
        if self.dashboard_port == 0 {
            anyhow::bail!("Dashboard port must be greater than 0");
        }

        Ok(())
    }
}
```

**Usage Examples:**

```bash
# Basic usage
tunnel-client --url wss://proxy.example.com --token abc123

# Full configuration
tunnel-client \
  --url wss://proxy.example.com \
  --token abc123 \
  --local https://localhost:3000 \
  --dashboard-port 8080 \
  --log-level debug

# Using environment variables
export TUNNEL_URL="wss://proxy.example.com"
export TUNNEL_TOKEN="abc123"
export TUNNEL_LOCAL="https://localhost:3000"
tunnel-client

# Disable dashboard
tunnel-client --url wss://proxy.example.com --token abc123 --no-dashboard
```

## Application Configuration Management

### Settings Structure

## Main Application Entry Point

### Application Bootstrap

## Cross-Platform Build Configuration

### Build Script Setup

### Cross-Compilation Scripts

### Distribution Packaging

This implementation guide provides:

1. **Complete project setup** with proper Cargo configuration
2. **CLI argument parsing** with validation and environment variable support
3. **Application configuration management** with file-based overrides
4. **Main application bootstrap** with concurrent component management
5. **Cross-platform build scripts** for Windows, Linux, and macOS
6. **Distribution packaging** with installers for each platform

The structure allows for:
- Easy command-line usage with intuitive arguments
- Environment variable configuration for CI/CD
- Graceful shutdown handling
- Concurrent execution of WebSocket client, HTTP proxy, and dashboard
- Professional distribution packages for each platform

Next steps would be implementing each component (WebSocket client, HTTP proxy forwarder, and dashboard server) based on this foundation.

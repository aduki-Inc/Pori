# Pori

A proxy client built in Rust that creates connections between local### Basic Usage

```bash
pori --url wss://proxy.example.com --token your-auth-token
```

### Using Configuration Files (Recommended)

Like ngrok, Pori supports configuration files for easier management:

```bash
# Create a YAML config file
cat > pori.yml << EOF
websocket:
  url: "wss://proxy.example.com"
  token: "your-auth-token"
local_server:
  url: "http://localhost:3000"
dashboard:
  port: 7616
EOF

# Run with config file
pori --yml pori.yml

# Or place config in default location and just run
mv pori.yml ~/.config/pori/config.yml
pori
```

### Advanced Usage Examples

```bash
# Connect to HTTPS local service on custom port
pori --url wss://proxy.example.com --token your-token --protocol https --port 8443

# Use HTTP with custom port and disable dashboard
pori --url wss://proxy.example.com --token your-token --protocol http --port 8080 --no-dashboard

# With custom dashboard port and debug logging
pori --url wss://proxy.example.com --token your-token --dashboard-port 9090 --log-level debug

# Mix config file with CLI overrides
pori --yml pori.yml --dashboard-port 8080
```te proxy servers through WebSocket tunnels. "Pori" means wild, open.

## Overview

Pori establishes and maintains WebSocket connections to proxy servers, forwarding HTTP requests to local services while providing monitoring through a dashboard. It handles connection failures with automatic reconnection and exponential backoff strategies.

## Features

### Core Functionality

- **WebSocket Tunneling**: Persistent connections to remote proxy servers
- **HTTP Request Forwarding**: Proxying of incoming requests to local services
- **Automatic Reconnection**: Connection management with configurable retry policies
- **Dashboard**: Web-based monitoring interface with statistics
- **Configuration Management**: Configuration via CLI arguments, environment variables, or configuration files

### Performance & Reliability

- **Asynchronous Architecture**: Built on Tokio for concurrent scenarios
- **Connection Pooling**: Connection reuse for local server communication
- **Resource Management**: Configurable connection limits and timeouts
- **Error Handling**: Error recovery and logging
- **Signal Handling**: Shutdown on system signals

### Security

- **TLS/SSL Support**: Connections with certificate validation options
- **Token Authentication**: Configurable authentication for proxy connections
- **Request Validation**: Input sanitization and validation

## Installation

### Quick Install (Linux/macOS)

Install Pori globally like ngrok - accessible from anywhere in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Proxy/main/release/install.sh | bash
```

After installation, simply run:

```bash
pori --url wss://proxy.example.com --token your-token
```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/aduki-Inc/Proxy/releases) page.

For Linux systems, use the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/aduki-Inc/Proxy/main/release/install.sh | bash
```

### Building from Source

Requirements:

- Rust 1.70 or later
- Cargo (included with Rust)

```bash
git clone https://github.com/aduki-Inc/Proxy.git
cd Proxy
cargo build --release
```

The compiled binary will be available at `target/release/pori`.

## Usage

### Basic Usage

```bash
pori --url wss://proxy.example.com --token your-auth-token
```

### Advanced Usage Examples

```bash
# Connect to HTTPS local service on custom port
pori --url wss://proxy.example.com --token your-token --protocol https --port 8443

# Use HTTP with custom port and disable dashboard
pori --url wss://proxy.example.com --token your-token --protocol http --port 7616 --no-dashboard

# With custom dashboard port and debug logging
pori --url wss://proxy.example.com --token your-token --dashboard-port 9090 --log-level debug
```

### Configuration Options

```bash
pori [OPTIONS]

Options:
      --url <URL>                    WebSocket URL for cloud/proxy connection [env: PORI_URL=]
      --token <TOKEN>                Access token for authentication [env: PORI_TOKEN=]
      --protocol <PROTOCOL>          Protocol for local server (http or https) [env: PORI_PROTOCOL=] [default: http]
      --port <PORT>                  Port for local server [env: PORI_PORT=] [default: 3000]
      --dashboard-port <PORT>        Dashboard server port [env: PORI_DASHBOARD_PORT=] [default: 7616]
      --log-level <LEVEL>            Log level [env: RUST_LOG=] [default: info]
      --config <CONFIG>              Configuration file path (TOML or JSON) [env: PORI_CONFIG=]
      --yml <YML>                    YAML configuration file path [env: PORI_YML=]
      --no-dashboard                 Disable dashboard server
      --timeout <TIMEOUT>            Connection timeout in seconds [default: 30]
      --max-reconnects <MAX>         Reconnection attempts (0 = infinite) [default: 0]
      --verify-ssl                   Verify SSL certificates for local server
      --max-connections <MAX>        Maximum connections to local server [default: 10]
  -h, --help                         Print help
  -V, --version                      Print version
```

### Environment Variables

All command-line options can be configured via environment variables:

- `PORI_URL`: WebSocket URL
- `PORI_TOKEN`: Authentication token
- `PORI_PROTOCOL`: Local server protocol (http or https)
- `PORI_PORT`: Local server port
- `PORI_DASHBOARD_PORT`: Dashboard port
- `RUST_LOG`: Log level
- `PORI_CONFIG`: Configuration file path

### Configuration File

Create a TOML, JSON, or YAML configuration file for persistent settings:

#### YAML Configuration (pori.yml)

```yaml
# WebSocket connection settings
websocket:
  url: "wss://proxy.example.com"
  token: "your-auth-token"
  timeout: 30
  max_reconnects: 0

# Local server configuration  
local_server:
  url: "http://localhost:3000"
  verify_ssl: false
  timeout: 30
  max_connections: 10

# Dashboard settings
dashboard:
  port: 7616
  bind_address: "127.0.0.1"
  enable_cors: true

# Logging configuration
logging:
  level: "info"
  format: "pretty"
  enable_color: true
```

#### TOML Configuration (pori.toml)

```toml
[websocket]
url = "wss://proxy.example.com"
token = "your-auth-token"
timeout = 30
max_reconnects = 0

[local_server]
url = "http://localhost:7616"
verify_ssl = false
timeout = 30
max_connections = 10

[dashboard]
enabled = true
port = 7616

[logging]
level = "info"
```

Use configuration files:

```bash
# Use YAML configuration
pori --yml pori.yml

# Use TOML configuration
pori --config pori.toml

# Use default locations (pori tries these automatically):
# ./pori.yml, ./pori.yaml, ./pori.toml, ./pori.json
# ~/.pori.yml, ~/.pori.yaml, ~/.pori.toml, ~/.pori.json  
# ~/.config/pori/config.yml, ~/.config/pori/config.yaml, ~/.config/pori/config.toml
pori

# Override config values with CLI arguments
pori --yml pori.yml --dashboard-port 8080
```

## Dashboard

The dashboard provides monitoring at `http://localhost:7616` (or your configured port):

- **Connection Status**: Current WebSocket connection state
- **Request Statistics**: Processed, successful, and failed request counts
- **Performance Metrics**: Average response times and throughput
- **Activity Log**: Recent connection events and errors
- **System Information**: Uptime, memory usage, and configuration details

## Architecture

### Components

- **WebSocket Client**: Manages persistent connections to proxy servers
- **HTTP Proxy**: Forwards incoming requests to local services
- **Dashboard Server**: Provides web-based monitoring interface
- **Configuration Manager**: Handles settings from multiple sources
- **Reconnection Manager**: Implements retry logic with exponential backoff

### Request Flow

1. Proxy server sends HTTP request via WebSocket
2. Pori receives and validates the request
3. Request is forwarded to the configured local service
4. Response is captured and sent back through the WebSocket
5. Statistics and logs are updated

## Development

### Prerequisites

- Rust 1.70+
- Git

### Building

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Project Structure

```
src/
├── config/          # Configuration management
├── logging/         # Logging setup and utilities
├── proxy/           # HTTP request forwarding
├── server/          # Dashboard server implementation
├── utils/           # Shared utilities and error handling
├── websocket/       # WebSocket client and connection management
├── lib.rs           # Application state and coordination
└── main.rs          # Entry point and CLI setup

static/              # Dashboard web assets
docs/                # Project documentation
release/             # Release artifacts and scripts
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Style

- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` to catch common issues
- Follow Rust naming conventions
- Add documentation for public APIs
- Include tests for new features

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Support

For questions, bug reports, or feature requests:

- Create an issue on GitHub
- Check existing documentation in the `docs/` directory
- Review the troubleshooting section below

## Troubleshooting

### Common Issues

**Connection Refused**

- Verify the WebSocket URL is correct
- Check network connectivity
- Ensure the proxy server is running

## Authentication Failed

- Verify the authentication token is valid
- Check token expiration
- Ensure proper token format

## Local Service Unreachable

- Confirm the local service is running
- Verify the local URL configuration
- Check SSL/TLS settings if using HTTPS

## High Memory Usage

- Reduce `max_connections` setting
- Enable connection pooling timeouts
- Monitor request patterns for issues

### Logging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug pori --url wss://proxy.example.com --token your-token
```

### Performance Tuning

For high-throughput scenarios:

- Increase `max_connections` for local server
- Adjust connection timeout values
- Monitor system resources
- Consider running multiple instances with load balancing

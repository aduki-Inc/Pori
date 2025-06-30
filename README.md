# Tunnel Client

A high-performance tunnel client built in Rust that creates secure connections between local services and remote proxy servers through WebSocket tunnels.

## Overview

The tunnel client establishes and maintains persistent WebSocket connections to proxy servers, forwarding HTTP requests to local services while providing real-time monitoring through an integrated dashboard. It handles connection failures gracefully with automatic reconnection and exponential backoff strategies.

## Features

### Core Functionality

- **WebSocket Tunneling**: Secure, persistent connections to remote proxy servers
- **HTTP Request Forwarding**: Efficient proxying of incoming requests to local services
- **Automatic Reconnection**: Robust connection management with configurable retry policies
- **Real-time Dashboard**: Web-based monitoring interface with live statistics
- **Configuration Management**: Flexible configuration via CLI arguments, environment variables, or configuration files

### Performance & Reliability

- **Asynchronous Architecture**: Built on Tokio for high-concurrency scenarios
- **Connection Pooling**: Efficient connection reuse for local server communication
- **Resource Management**: Configurable connection limits and timeouts
- **Error Handling**: Comprehensive error recovery and logging
- **Signal Handling**: Graceful shutdown on system signals

### Security

- **TLS/SSL Support**: Secure connections with certificate validation options
- **Token Authentication**: Configurable authentication for proxy connections
- **Request Validation**: Input sanitization and validation

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/your-org/tunnel-client/releases) page.

For Linux systems, use the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/your-org/tunnel-client/main/release/install.sh | bash
```

### Building from Source

Requirements:

- Rust 1.70 or later
- Cargo (included with Rust)

```bash
git clone https://github.com/your-org/tunnel-client.git
cd tunnel-client
cargo build --release
```

The compiled binary will be available at `target/release/tunnel-client`.

## Usage

### Basic Usage

```bash
tunnel-client --url wss://proxy.example.com --token your-auth-token
```

### Advanced Usage Examples

```bash
# Connect to HTTPS local service on custom port
tunnel-client --url wss://proxy.example.com --token your-token --protocol https --port 8443

# Use HTTP with custom port and disable dashboard
tunnel-client --url wss://proxy.example.com --token your-token --protocol http --port 8080 --no-dashboard

# With custom dashboard port and debug logging
tunnel-client --url wss://proxy.example.com --token your-token --dashboard-port 9090 --log-level debug
```

### Configuration Options

```bash
tunnel-client [OPTIONS] --url <URL> --token <TOKEN>

Options:
      --url <URL>                    WebSocket URL for cloud/proxy connection
      --token <TOKEN>                Access token for authentication
      --protocol <PROTOCOL>          Protocol for local server (http or https) [default: http]
      --port <PORT>                  Port for local server [default: 3000]
      --dashboard-port <PORT>        Dashboard server port [default: 8080]
      --log-level <LEVEL>            Log level [default: info]
      --config <CONFIG>              Configuration file path
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

- `TUNNEL_URL`: WebSocket URL
- `TUNNEL_TOKEN`: Authentication token
- `TUNNEL_PROTOCOL`: Local server protocol (http or https)
- `TUNNEL_PORT`: Local server port
- `TUNNEL_DASHBOARD_PORT`: Dashboard port
- `RUST_LOG`: Log level
- `TUNNEL_CONFIG`: Configuration file path

### Configuration File

Create a TOML configuration file for persistent settings:

```toml
[websocket]
url = "wss://proxy.example.com"
token = "your-auth-token"
timeout = 30
max_reconnects = 0

[local_server]
url = "http://localhost:8080"
verify_ssl = false
timeout = 30
max_connections = 10

[dashboard]
enabled = true
port = 8080

[logging]
level = "info"
```

Use the configuration file:

```bash
tunnel-client --config config.toml
```

## Dashboard

The integrated dashboard provides real-time monitoring at `http://localhost:8080` (or your configured port):

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
2. Tunnel client receives and validates the request
3. Request is forwarded to the configured local service
4. Response is captured and sent back through the WebSocket
5. Statistics and logs are updated in real-time

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
RUST_LOG=debug tunnel-client --url wss://proxy.example.com --token your-token
```

### Performance Tuning

For high-throughput scenarios:

- Increase `max_connections` for local server
- Adjust connection timeout values
- Monitor system resources
- Consider running multiple instances with load balancing

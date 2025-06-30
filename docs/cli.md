# CLI Guide: Command Line Interface

## Overview

Command-line interface for tunnel client configuration using clap derive macros. Supports both CLI arguments and environment variables for flexible deployment.

## Command Structure

### Basic Usage

pori --url wss://tunnel.example.com --token my-token --local https://localhost:7616 Run tunnel client with minimal required arguments

```bash
pori --url wss://proxy.example.com --token abc123
```

### Full Configuration

> Complete argument specification

```bash
pori \
  --url wss://proxy.example.com \
  --token abc123 \
  --local https://localhost:3000 \
  --dashboard-port 7616 \
  --log-level debug \
  --timeout 30 \
  --max-reconnects 5
```

### Environment Variables

> Use environment variables for configuration

```bash
export TUNNEL_URL="wss://proxy.example.com"
export PORI_TOKEN="abc123"
export PORI_LOCAL="https://localhost:3000"
export PORI_DASHBOARD_PORT="7616"
export RUST_LOG="debug"

pori
```

## Argument Specifications

### Required Arguments

**--url \<websocket-url\>**

- Purpose: WebSocket URL for cloud/proxy connection
- Format: ws:// or wss:// scheme required
- Environment: TUNNEL_URL
- Example: wss://proxy.example.com/tunnel

**--token \<access-token\>**

- Purpose: Authentication token for cloud/proxy
- Format: Non-empty string
- Environment: TUNNEL_TOKEN
- Security: Keep confidential, use environment variables in production

### Optional Arguments

**--local \<local-url\>**

- Purpose: Local server URL to forward requests to
- Default: https://localhost:3000
- Format: http:// or https:// scheme required
- Environment: TUNNEL_LOCAL
- Example: https://localhost:7616

**--dashboard-port \<port\>**

- Purpose: Port for local dashboard server
- Default: 7616
- Range: 1-65535
- Environment: TUNNEL_DASHBOARD_PORT
- Conflicts: Automatically handled if port in use

**--log-level \<level\>**

- Purpose: Logging verbosity level
- Default: info
- Options: error, warn, info, debug, trace
- Environment: RUST_LOG
- Format: Supports complex filters (module::path=level)

**--timeout \<seconds\>**

- Purpose: Connection timeout for WebSocket
- Default: 30
- Range: 1-300 seconds
- Environment: TUNNEL_TIMEOUT
- Use case: Adjust for slow networks

**--max-reconnects \<count\>**

- Purpose: Maximum reconnection attempts
- Default: 0 (infinite)
- Range: 0-999
- Environment: TUNNEL_MAX_RECONNECTS
- Behavior: 0 means infinite attempts

### Flags

**--no-dashboard**

- Purpose: Disable dashboard server
- Default: false
- Use case: Headless operation, CI/CD environments
- Effect: Saves resources when monitoring not needed

**--help (-h)**

- Purpose: Display help information
- Shows all available options and examples

**--version (-V)**

- Purpose: Display version information
- Shows current build version and commit

## Configuration Validation

### URL Validation

> Validate WebSocket and local URLs at startup

**WebSocket URL checks:**
- Must use ws:// or wss:// scheme
- Must be valid URL format
- Must include host (domain or IP)
- Port specification optional

**Local URL checks:**
- Must use http:// or https:// scheme
- Must be valid URL format
- Must include host and port if not default

### Token Validation

> Ensure access token meets requirements

**Token checks:**
- Cannot be empty string
- No whitespace validation (tokens may contain spaces)
- Length validation if required by server

### Port Validation

> Validate dashboard port availability

**Port checks:**
- Must be in valid range (1-65535)
- Must not be reserved system port (if not root)
- Availability check at startup
- Fallback behavior if port occupied

## Environment Variable Priority

### Precedence Order

1. Command-line arguments (highest priority)
2. Environment variables
3. Default values (lowest priority)

### Environment Mapping

- TUNNEL_URL → --url
- TUNNEL_TOKEN → --token
- TUNNEL_LOCAL → --local
- TUNNEL_DASHBOARD_PORT → --dashboard-port
- RUST_LOG → --log-level
- TUNNEL_TIMEOUT → --timeout
- TUNNEL_MAX_RECONNECTS → --max-reconnects

## Configuration File Support

### Optional Configuration File

> Support TOML configuration file for complex setups

**File format:**
```toml
url = "wss://proxy.example.com"
token = "abc123"
local = "https://localhost:3000"
dashboard_port = 7616
log_level = "info"
timeout = 30
max_reconnects = 0
no_dashboard = false
```

**File location precedence:**
1. --config argument
2. TUNNEL_CONFIG environment variable
3. ./pori.toml
4. ~/.pori.toml

## Error Handling

### Validation Errors

> Clear error messages for configuration issues

**Common errors:**
- Invalid URL schemes
- Empty required fields
- Out-of-range port numbers
- Unreachable local server

**Error format:**
- Clear problem description
- Suggested fixes
- Exit code 1 for configuration errors

### Runtime Errors

> Handle runtime configuration issues

**Connection errors:**
- WebSocket connection failures
- Authentication rejections
- Local server unavailable
- Dashboard port conflicts

## Help Documentation

### Built-in Help

> Comprehensive help generated by clap

**Help sections:**
- Usage examples
- Argument descriptions
- Environment variable mapping
- Common configuration patterns

### Examples Section

**Basic tunnel:**
```bash
pori --url wss://tunnel.example.com --token my-secret-token
```

**Custom local server:**
```bash
tunnel-client --url wss://tunnel.example.com --token my-token --local https://localhost:7616
```

**Headless operation:**
```bash
pori --url wss://tunnel.example.com --token my-token --no-dashboard
```

**Debug mode:**
```bash
pori --url wss://tunnel.example.com --token my-token --log-level debug
```

## Production Deployment

### Systemd Service

> Example systemd service configuration

**Service file:**
```ini
[Unit]
Description=Tunnel Client
After=network.target

[Service]
Type=simple
User=tunnel
Environment=TUNNEL_URL=wss://proxy.example.com
Environment=TUNNEL_TOKEN=production-token
Environment=TUNNEL_LOCAL=https://localhost:3000
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/pori
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Docker Configuration

> Environment variable configuration for containers

**Docker run:**
```bash
docker run -d \
  -e TUNNEL_URL=wss://proxy.example.com \
  -e TUNNEL_TOKEN=abc123 \
  -e TUNNEL_LOCAL=https://app:3000 \
  -p 7616:7616 \
  pori
```

This CLI implementation provides flexible configuration options while maintaining security and ease of use for both development and production environments.

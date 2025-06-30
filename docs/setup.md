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

## Disable dashboard

```bash
pori --url wss://proxy.example.com --token abc123 --no-dashboard
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

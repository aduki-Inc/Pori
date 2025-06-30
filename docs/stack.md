# Technology Stack Guide: Rust Tunnel Client with hyper.rs

## Core Dependencies Overview

This guide covers the technology choices for building a high-performance tunnel client that:

1. **Connects to cloud/proxy via WebSocket** for bidirectional communication
2. **Forwards HTTP requests to local HTTPS server** (e.g., localhost:3000)
3. **Serves a local dashboard** for monitoring using hyper.rs
4. **Compiles to multiple platforms** (Windows, Linux, macOS)

## Primary Technology Stack

### HTTP Server: hyper.rs (Dashboard & API)

**Why hyper.rs over Axum?**

- **Lower-level control**: Direct access to HTTP internals and connection management
- **Performance**: Minimal overhead, no framework abstractions for proxy operations
- **Memory efficiency**: Zero-cost abstractions with fine-grained control
- **Flexibility**: Custom routing exactly suited for dashboard + API endpoints
- **WebSocket upgrade**: Native support for upgrading HTTP to WebSocket
- **Binary size**: Smaller final executable compared to higher-level frameworks

**Cargo.toml entry**:
```toml
hyper = { version = "1.0", features = ["full"] }
hyper-util = "0.1"
http-body-util = "0.1"
```

**Key features used**:

- Static file serving for dashboard (HTML/CSS/JS)
- REST API endpoints for configuration and status
- WebSocket upgrade for real-time dashboard updates
- CORS handling for browser compatibility
- Custom service implementation for routing

### WebSocket Client: tokio-tungstenite

**Purpose**: WebSocket client implementation

**Cargo.toml entry**:
```toml
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
```

**Features**:

- Async WebSocket client
- TLS/SSL support
- Message compression
- Ping/pong handling
- Connection close handling

### Async Runtime: Tokio

**Purpose**: Asynchronous runtime for Rust

**Cargo.toml entry**:
```toml
tokio = { version = "1.0", features = ["full"] }
```

**Why Tokio**:

- Industry standard async runtime
- Excellent performance
- Rich ecosystem
- Built-in timers and I/O
- Signal handling support

### CLI Parsing: clap

**Purpose**: Command-line argument parsing

**Cargo.toml entry**:
```toml
clap = { version = "4.0", features = ["derive"] }
```

**Features used**:

- Derive macros for easy CLI definition
- Automatic help generation
- Input validation
- Environment variable support

### Serialization: serde

**Purpose**: JSON/TOML serialization and deserialization

**Cargo.toml entry**:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
```

**Use cases**:

- WebSocket message serialization
- Configuration file parsing
- API request/response handling

### Logging: tracing

**Purpose**: Structured logging and diagnostics

**Cargo.toml entry**:
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Features**:

- Structured logging with spans
- Multiple output formats
- Log level filtering
- Performance tracing

### Error Handling

**Libraries**:
```toml
thiserror = "1.0"  # Error derive macros
anyhow = "1.0"     # Error context and chaining
```

**Strategy**:

- Use `thiserror` for library errors
- Use `anyhow` for application errors
- Implement custom error types for domain-specific errors

## Development Dependencies

### Cross-Platform Compilation

**Tool**: cargo-cross

**Installation**:
```bash
cargo install cross
```

**Purpose**:

- Cross-compile for Windows, Linux, macOS
- Docker-based compilation environments
- Consistent builds across platforms

### Testing Framework

**Built-in testing**:
```toml
[dev-dependencies]
tokio-test = "0.4"
```

**Integration testing**:

- Use `tokio-test` for async testing
- Mock WebSocket servers for testing
- HTTP client testing with real requests

### Benchmarking

**Tool**: criterion

**Cargo.toml entry**:
```toml
[dev-dependencies]
criterion = "0.5"
```

**Purpose**:

- Performance benchmarking
- Regression testing
- Memory usage profiling

## Build Configuration

### Cargo.toml Structure

```toml
[package]
name = "tunnel-client"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]
description = "High-performance tunnel client with dashboard"
license = "MIT"
repository = "https://github.com/username/tunnel-client"

[[bin]]
name = "tunnel-client"
path = "src/main.rs"

[dependencies]
# Core async runtime
tokio = { version = "1.0", features = ["full"] }

# HTTP server
hyper = { version = "1.0", features = ["full"] }
hyper-util = "0.1"

# WebSocket client
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }

# CLI and configuration
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4"] }
url = "2.0"
mime = "0.3"
mime_guess = "2.0"

[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 0
debug = true
```

### Cross-Platform Targets

**Supported platforms**:

- Linux: `x86_64-unknown-linux-gnu`
- Windows: `x86_64-pc-windows-gnu`
- macOS: `x86_64-apple-darwin`
- ARM Linux: `aarch64-unknown-linux-gnu`

**Cross.toml configuration**:
```toml
[build.env]
passthrough = [
    "RUST_LOG",
]

[target.x86_64-pc-windows-gnu]
pre-build = [
    "apt-get update && apt-get install -y mingw-w64"
]

[target.x86_64-apple-darwin]
pre-build = [
    "apt-get update && apt-get install -y clang"
]
```

## Feature Flags

### Optional Features

```toml
[features]
default = ["tls"]
tls = ["tokio-tungstenite/native-tls"]
rustls = ["tokio-tungstenite/rustls-tls-native-roots"]
compression = ["tokio-tungstenite/deflate"]
```

**Feature descriptions**:

- `tls`: Native TLS support (default)
- `rustls`: Pure Rust TLS implementation
- `compression`: WebSocket compression support

## Static Assets

### Dashboard Files

**Directory structure**:
```text
static/
├── index.html
├── css/
│   ├── main.css
│   └── dashboard.css
├── js/
│   ├── main.js
│   ├── websocket.js
│   └── dashboard.js
└── assets/
    ├── logo.png
    └── favicon.ico
```

**Embedding strategy**:
```toml
[build-dependencies]
include_dir = "0.7"
```

**Purpose**:

- Embed static files in binary
- No external file dependencies
- Single executable distribution

## Memory and Performance Optimization

### Compile-time Optimizations

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit for better optimization
panic = "abort"     # Smaller binary size
strip = true        # Remove debug symbols
```

### Runtime Optimizations

**Memory management**:

- Use `Arc` for shared immutable data
- Use `Mutex` sparingly, prefer channels
- Implement bounded channels to prevent memory leaks
- Use `bytes::Bytes` for zero-copy operations

**Performance features**:

- Connection pooling for HTTP
- WebSocket compression
- Async I/O throughout
- Minimal allocations in hot paths

## Development Workflow

### Local Development

```bash
# Install dependencies
cargo build

# Run with logging
RUST_LOG=debug cargo run -- --url wss://example.com --token abc123

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Cross-Platform Builds

```bash
# Install cross
cargo install cross

# Build for all platforms
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
```

### Production Builds

```bash
# Optimized release build
cargo build --release

# Strip debug symbols (Linux/macOS)
strip target/release/tunnel-client

# Create distribution archives
tar -czf tunnel-client-linux.tar.gz -C target/release tunnel-client
```

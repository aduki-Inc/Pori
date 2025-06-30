# Project Overview: Rust Tunnel Client

## Project Description

A high-performance Rust client application that connects to a cloud/proxy WebSocket server for tunnel functionality. This client will serve a local web dashboard using hyper.rs and establish WebSocket connections to remote servers.

## Key Features

- **WebSocket Client**: Connect to cloud/proxy WebSocket servers for bidirectional communication
- **Local Dashboard**: Serve a web interface using hyper.rs for monitoring and control
- **Cross-Platform**: Compile and distribute for Windows, Linux, and macOS
- **Command-Line Interface**: Configure URL and access token via command-line arguments
- **Standalone Binary**: Single executable with no external dependencies

## Architecture Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Browser       │    │  Local Client   │    │  Cloud/Proxy    │
│   Dashboard     │───▶│  (hyper.rs)     │◀──▶│  WebSocket      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │ Local Web       │
                       │ Dashboard       │
                       │ (Static Files)  │
                       └─────────────────┘
```

## Technology Stack

- **HTTP Server**: hyper.rs (instead of Axum)
- **WebSocket Client**: tokio-tungstenite
- **Async Runtime**: Tokio
- **CLI Parsing**: clap
- **Configuration**: serde + toml
- **Logging**: tracing + tracing-subscriber
- **Error Handling**: thiserror + anyhow
- **Cross-compilation**: cargo-cross

## Project Goals

1. Create a lightweight, fast tunnel client
2. Provide real-time dashboard for monitoring
3. Support cross-platform deployment
4. Easy command-line configuration
5. Production-ready error handling and logging

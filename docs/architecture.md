# Architecture Guide: Rust Tunnel Client with hyper.rs

## System Architecture

### Component Overview

The Rust tunnel client consists of several kepori --url wss://proxy.example.com --token abc123 --local https://localhost:3000 --dashboard-port 8080 components working together:

1. **Dashboard HTTP Server (hyper.rs)** - Serves the local dashboard interface
2. **WebSocket Client** - Connects to cloud/proxy servers for bidirectional communication
3. **HTTP Proxy Forwarder** - Forwards incoming requests to local HTTPS server
4. **Message Router** - Routes messages between WebSocket and local server
5. **Configuration Manager** - Handles CLI arguments (URL, token, local server details)

### Data Flow Architecture

```text
Command Line Arguments (URL, Token, Local Server)
                    ↓
               Configuration
                    ↓
        ┌─────────────────────┐
        │    Main Process     │
        └─────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
Dashboard Server  WebSocket   HTTP Proxy
  (hyper.rs)      Client      Forwarder
        │            │            │
        ▼            ▼            ▼
   Browser      Cloud/Proxy   Local HTTPS
  Dashboard   ←─────────────→    Server
                              (localhost:3000)

Message Flow:
Internet → Cloud/Proxy → WebSocket → HTTP Proxy → Local Server
Local Server → HTTP Proxy → WebSocket → Cloud/Proxy → Internet
```

## Module Structure

```text
src/
├── main.rs              # Entry point and CLI handling
├── lib.rs               # Library exports
├── config/
│   ├── mod.rs          # Configuration management
│   ├── cli.rs          # Command-line argument parsing
│   └── settings.rs     # Application settings
├── server/
│   ├── mod.rs          # Dashboard HTTP server using hyper.rs
│   ├── dashboard.rs    # Dashboard route handlers
│   ├── static_files.rs # Static file serving
│   └── api.rs          # API endpoints
├── proxy/
│   ├── mod.rs          # HTTP proxy forwarding logic
│   ├── forwarder.rs    # Request forwarding to local server
│   ├── response.rs     # Response handling from local server
│   └── client.rs       # HTTP client for local server communication
├── websocket/
│   ├── mod.rs          # WebSocket client management
│   ├── client.rs       # WebSocket connection logic
│   ├── messages.rs     # Message types and serialization
│   ├── reconnect.rs    # Reconnection logic
│   └── tunnel.rs       # Tunnel message handling
├── logging/
│   ├── mod.rs          # Logging configuration
│   └── formatters.rs   # Custom log formatters
└── utils/
    ├── mod.rs          # Utility functions
    ├── error.rs        # Error types
    └── signals.rs      # Signal handling (graceful shutdown)
```

## Component Responsibilities

### Dashboard HTTP Server (hyper.rs)

**Purpose**: Serve local dashboard interface for monitoring and control

**Responsibilities**:

- Serve static HTML/CSS/JS files for dashboard
- Provide REST API for dashboard interactions
- Handle CORS for browser compatibility
- Manage HTTP keep-alive connections

**Key Features**:

- Low-level HTTP control with hyper.rs
- Custom routing without framework overhead
- Static file serving with proper MIME types
- WebSocket upgrade handling for dashboard real-time updates

### HTTP Proxy Forwarder

**Purpose**: Forward HTTP requests from cloud to local HTTPS server

**Responsibilities**:

- Receive HTTP requests via WebSocket from cloud proxy
- Forward requests to local HTTPS server (e.g., localhost:3000)
- Handle request/response transformation
- Manage connection pooling to local server
- Support HTTPS/TLS connections to local server

**Key Features**:

- Efficient request forwarding with minimal latency
- Connection reuse for local server communication
- Request/response streaming for large payloads
- Error handling and retry logic

### WebSocket Client

**Purpose**: Maintain connection to cloud/proxy servers for tunnel communication

**Responsibilities**:

- Establish WebSocket connections using provided URL
- Authenticate using access token
- Handle bidirectional message sending/receiving (tunnel traffic)
- Implement reconnection logic with exponential backoff
- Monitor connection health with ping/pong

**Key Features**:

- Automatic reconnection on connection loss
- Message queuing during disconnections
- Heartbeat monitoring
- Secure WebSocket (WSS) support
- Binary message support for HTTP request/response data

### Configuration Manager

**Purpose**: Handle all application configuration

**Responsibilities**:

- Parse command-line arguments (URL, token, local server details)
- Validate URL and access token
- Configure local server target (host:port)
- Provide default values
- Environment variable support

**Command-line Interface**:

```bash
tunnel-client --url wss://proxy.example.com --token abc123 --local https://localhost:3000 --dashboard-port 7616
```

### Message Router

**Purpose**: Route messages between WebSocket, HTTP proxy, and dashboard

**Responsibilities**:

- Forward HTTP requests from WebSocket to local server
- Send HTTP responses from local server back via WebSocket
- Broadcast tunnel statistics to dashboard
- Handle message transformation/filtering
- Maintain connection state and metrics

## Communication Patterns

### Dashboard ↔ Local Dashboard Server

- **Protocol**: HTTP/WebSocket
- **Format**: JSON messages
- **Transport**: Local network (127.0.0.1)
- **Purpose**: Monitor tunnel status and statistics

### Local Client ↔ Cloud/Proxy

- **Protocol**: WebSocket (WSS)
- **Format**: Binary/JSON messages (HTTP requests/responses)
- **Authentication**: Token-based
- **Reconnection**: Automatic with backoff
- **Purpose**: Tunnel HTTP traffic bidirectionally

### Local Client ↔ Local HTTPS Server

- **Protocol**: HTTP/HTTPS
- **Format**: Standard HTTP requests/responses
- **Transport**: Local network or localhost
- **Purpose**: Forward incoming tunnel requests to actual local application

## Error Handling Strategy

### Connection Errors

- WebSocket disconnections: Automatic reconnection with exponential backoff
- HTTP server errors: Graceful error responses
- Authentication failures: Clear error messages and exit

### Configuration Errors

- Invalid URLs: Validation at startup
- Missing tokens: Required parameter validation
- Port conflicts: Alternative port selection

### Runtime Errors

- Message parsing errors: Log and continue
- Dashboard connection issues: Non-blocking operation
- Resource exhaustion: Graceful degradation

## Security Considerations

### WebSocket Security

- Use WSS (secure WebSocket) for cloud connections
- Validate SSL certificates
- Implement proper token authentication
- Rate limiting for reconnection attempts

### Local Dashboard Security

- Bind only to localhost (127.0.0.1)
- No external network access
- Input validation for all API endpoints
- CSRF protection for state-changing operations

## Performance Considerations

### Memory Management

- Use `Arc<Mutex<>>` for shared state
- Implement bounded channels to prevent memory leaks
- Lazy loading of static dashboard files
- Connection pooling for HTTP keep-alive

### CPU Optimization

- Async/await for non-blocking I/O
- Minimize serialization/deserialization overhead
- Use zero-copy operations where possible
- Efficient message routing without unnecessary copies

### Network Optimization

- WebSocket compression support
- HTTP/2 support in hyper.rs
- Connection keep-alive
- Bandwidth monitoring and throttling

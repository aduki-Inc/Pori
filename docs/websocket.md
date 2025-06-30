# WebSocket Client Implementation Guide

## WebSocket Client Architecture

### Purpose and Responsibilities

The WebSocket client is the core component that:
1. **Connects to cloud/proxy servers** using provided WSS URL and token
2. **Receives HTTP requests** from the cloud to forward to local server
3. **Sends HTTP responses** back to cloud after processing by local server
4. **Maintains persistent connection** with automatic reconnection
5. **Handles authentication** using provided access token

### Component Structure

```text
websocket/
├── mod.rs          # Public module interface
├── client.rs       # Main WebSocket client logic
├── messages.rs     # Message types and serialization
├── reconnect.rs    # Reconnection logic with backoff
└── tunnel.rs       # HTTP tunnel message handling
```

## Implementation Strategy

### Step 1: Create Message Protocol Structure

> Create `src/websocket/messages.rs` file

**Define TunnelMessage enum with these message types:**
- `Auth` - Send authentication token to cloud
- `AuthSuccess` - Receive successful authentication with session ID
- `AuthError` - Handle authentication failures
- `HttpRequest` - Receive HTTP requests from cloud to forward to local server
- `HttpResponse` - Send HTTP responses back to cloud after local server processing
- `Ping/Pong` - Connection health monitoring
- `Error` - Handle error messages
- `Stats` - Send tunnel statistics

**Implement message serialization methods:**
- `to_json()` - Convert message to JSON string
- `from_json()` - Parse JSON string to message
- `to_binary()` - Convert message to binary format
- `from_binary()` - Parse binary data to message

**Add message helper constructors:**
- `auth(token)` - Create authentication message
- `http_request(id, method, url, headers, body)` - Create HTTP request message
- `http_response(id, status, headers, body)` - Create HTTP response message
- `error(request_id, error, code)` - Create error message
- `ping()` / `pong()` - Create health check messages

### Step 2: Implement WebSocket Client Core

> Create `src/websocket/client.rs` file

**Define WebSocketClient struct with fields:**
- `url` - WebSocket server URL from CLI
- `token` - Access token for authentication
- `app_state` - Shared application state
- `reconnect_manager` - Handle reconnection logic
- `message_tx/rx` - Channels for message passing

**Implement client lifecycle methods:**
- `new()` - Initialize client with URL, token, and app state
- `run()` - Main client loop with reconnection handling
- `connect_and_run()` - Single connection attempt and message handling
- `handle_incoming_message()` - Process messages from cloud
- `get_message_sender()` - Provide channel for sending messages

**Setup message handling flow:**
1. Establish WebSocket connection using `tokio_tungstenite::connect_async()`
2. Send authentication message immediately after connection
3. Split WebSocket stream for concurrent read/write
4. Create separate tasks for ping/pong and message processing
5. Use `tokio::select!` for handling multiple async operations
6. Forward HTTP requests to proxy module via app state channels
7. Send HTTP responses back to cloud via WebSocket

### Step 3: Add Reconnection Logic

> Create `src/websocket/reconnect.rs` file

**Implement ReconnectManager struct:**
- `max_attempts` - Maximum reconnection attempts (0 = infinite)
- `current_attempt` - Track current attempt number
- `base_delay` - Initial delay between attempts
- `max_delay` - Maximum delay cap
- `backoff_multiplier` - Exponential backoff factor

**Add reconnection methods:**
- `should_reconnect()` - Check if more attempts allowed
- `next_delay()` - Calculate next delay with exponential backoff
- `reset()` - Reset counter on successful connection
- `with_max_attempts()` - Builder pattern for configuration
- `with_base_delay()` - Configure initial delay

**Implement exponential backoff strategy:**
- Start with 1 second delay
- Double delay on each attempt (exponential)
- Cap maximum delay at 5 minutes
- Reset counter on successful connection
- Allow infinite attempts by default


## Connection Health Monitoring

### Health Check Implementation

## Integration with Application State

### State Management for WebSocket

This WebSocket client implementation provides:

1. **Robust connection management** with automatic reconnection
2. **Comprehensive message protocol** for tunnel communication
3. **Health monitoring** with ping/pong and timeout detection
4. **Exponential backoff** for reconnection attempts
5. **Statistics tracking** for monitoring and dashboard
6. **Error handling** with proper error propagation
7. **Concurrent message processing** for high throughput
8. **Authentication handling** with token-based auth
9. **Binary and text message support** for flexibility
10. **Integration with application state** for coordinated operation

The client is designed to be resilient, performant, and maintainable while providing all the functionality needed for the tunnel proxy system.

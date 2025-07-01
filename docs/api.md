# API Documentation

This document provides comprehensive information about Pori's WebSocket and REST API endpoints.

## WebSocket Endpoints

Pori provides two primary WebSocket endpoints for different purposes:

### Main Proxy Endpoint

**Endpoint:** `ws://localhost:7616/` or `wss://server_url/`

**Purpose:** Primary tunnel endpoint for HTTP request forwarding

**Authentication:** Token-based via query parameter

**Usage:**

```websocket
wss://server_url/?token=your-auth-token
```

**Message Protocol:**

- **Authentication:** Initial token validation
- **HTTP Requests:** Bidirectional HTTP traffic tunneling
- **Health Monitoring:** Ping/pong for connection health
- **Error Handling:** Graceful error reporting

**Example Flow:**

1. Client connects with authentication token
2. Server validates token and establishes tunnel
3. HTTP requests are forwarded through the tunnel
4. Responses are sent back through the same connection

### Metrics Dashboard Endpoint

**Endpoint:** `ws://localhost:7616/metrics` or `wss://server_url/metrics`

**Purpose:** Real-time analytics and metrics streaming

**Authentication:** Token-based via query parameter

**Usage:**

```websocket
wss://server_url/metrics?token=your-auth-token
```

**Real-time Data:**

- Connection status updates
- Request/response statistics
- Performance metrics
- Error notifications
- Configuration changes

**Message Format:**

```json
{
  "type": "stats_update",
  "timestamp": "2025-06-30T10:30:00Z",
  "data": {
    "requests_processed": 1247,
    "requests_successful": 1230,
    "requests_failed": 17,
    "bytes_forwarded": 2045678,
    "uptime_seconds": 3600,
    "connection_status": "connected"
  }
}
```

## REST API Endpoints

All REST API endpoints are available at `http://localhost:7616/api/` (or your configured dashboard port).

### Status Endpoint

**GET /api/status**

Get current connection status and basic information.

**Response:**

```json
{
  "status": "ok",
  "connection_status": "connected",
  "uptime_seconds": 3600,
  "requests_processed": 1247,
  "websocket_reconnects": 2
}
```

### Statistics Endpoint

**GET /api/stats**

Get detailed statistics and metrics.

**Response:**

```json
{
  "requests_processed": 1247,
  "requests_successful": 1230,
  "requests_failed": 17,
  "bytes_forwarded": 2045678,
  "uptime_seconds": 3600,
  "websocket_reconnects": 2,
  "connection_status": "connected"
}
```

### Configuration Endpoint

**GET /api/config**

Get current configuration settings.

**Response:**

```json
{
  "websocket_url": "wss://proxy.example.com",
  "local_server_url": "http://localhost:3000",
  "dashboard_port": 7616,
  "dashboard_address": "127.0.0.1",
  "max_reconnects": 0,
  "timeout_seconds": 30,
  "verify_ssl": false
}
```

### Endpoints Discovery

**GET /api/endpoints**

Get list of all available endpoints with descriptions.

**Response:**

```json
{
  "websocket_endpoints": [
    {
      "path": "/",
      "name": "Main Proxy Endpoint",
      "description": "WebSocket endpoint for HTTP request forwarding",
      "url": "wss://server_url/?token=your-token",
      "usage": "Main proxy endpoint for request forwarding"
    },
    {
      "path": "/metrics",
      "name": "Real-time Analytics Dashboard",
      "description": "WebSocket endpoint for real-time metrics and analytics",
      "url": "wss://server_url/metrics?token=your-token",
      "usage": "Real-time analytics and metrics dashboard"
    }
  ],
  "api_endpoints": [
    {
      "path": "/api/status",
      "method": "GET",
      "description": "Get current connection status and basic info"
    },
    {
      "path": "/api/stats",
      "method": "GET", 
      "description": "Get detailed statistics and metrics"
    }
  ]
}
```

### Control Endpoints

#### Reconnect

**POST /api/reconnect**

Trigger a WebSocket reconnection.

**Response:**

```json
{
  "status": "success",
  "message": "Reconnection initiated"
}
```

#### Shutdown

**POST /api/shutdown**

Initiate graceful shutdown.

**Response:**

```json
{
  "status": "success",
  "message": "Shutdown initiated"
}
```

## Authentication

### WebSocket Authentication

WebSocket endpoints use token-based authentication via query parameters:

```websocket
wss://server_url/path?token=your-auth-token
```

**Security Notes:**

- Tokens should be kept secure and not logged
- Use WSS (secure WebSocket) in production
- Tokens are validated on connection establishment

### REST API Authentication

REST API endpoints are currently accessible without authentication when accessed from localhost. In production deployments, consider implementing proper authentication mechanisms.

## Error Handling

### WebSocket Errors

**Connection Errors:**

- `401 Unauthorized`: Invalid or missing token
- `403 Forbidden`: Token validation failed
- `500 Internal Server Error`: Server-side error

**Message Errors:**

```json
{
  "type": "error",
  "request_id": "uuid-here",
  "error": "Error description",
  "code": 500
}
```

### REST API Errors

**Standard HTTP Status Codes:**

- `200 OK`: Success
- `400 Bad Request`: Invalid request
- `404 Not Found`: Endpoint not found
- `500 Internal Server Error`: Server error

**Error Response Format:**

```json
{
  "error": "Error Type",
  "message": "Detailed error message"
}
```

## Rate Limiting

Currently, no rate limiting is implemented for local dashboard access. In production deployments, consider implementing appropriate rate limiting for security.

## CORS Policy

CORS is enabled for dashboard endpoints with permissive settings suitable for local development:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: content-type
```

## Security Considerations

1. **Local Only**: Dashboard binds to localhost (127.0.0.1) by default
2. **Token Security**: Keep WebSocket tokens secure
3. **TLS**: Use WSS for production connections
4. **Validation**: All inputs are validated server-side
5. **No External Access**: Dashboard is not exposed to external networks

## Integration Examples

### JavaScript WebSocket Client

```javascript
// Connect to main proxy endpoint
const proxyWs = new WebSocket('wss://server_url/?token=your-token');

proxyWs.onopen = () => {
  console.log('Connected to proxy endpoint');
};

proxyWs.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};

// Connect to metrics endpoint
const metricsWs = new WebSocket('wss://server_url/metrics?token=your-token');

metricsWs.onmessage = (event) => {
  const metrics = JSON.parse(event.data);
  updateDashboard(metrics);
};
```

### cURL Examples

```bash
# Get status
curl http://localhost:7616/api/status

# Get statistics
curl http://localhost:7616/api/stats

# Trigger reconnection
curl -X POST http://localhost:7616/api/reconnect

# Get endpoint list
curl http://localhost:7616/api/endpoints
```

### Python Example

```python
import asyncio
import websockets
import json

async def connect_to_metrics():
    uri = "ws://localhost:7616/metrics?token=your-token"
    
    async with websockets.connect(uri) as websocket:
        print("Connected to metrics endpoint")
        
        async for message in websocket:
            data = json.loads(message)
            print(f"Metrics update: {data}")

# Run the client
asyncio.run(connect_to_metrics())
```

## Troubleshooting

### Common Issues

**Connection Refused:**

- Verify Pori is running and dashboard is enabled
- Check dashboard port configuration
- Ensure no firewall blocking localhost connections

**Authentication Failed:**

- Verify token is correct and properly URL-encoded
- Check token parameter name is exactly "token"
- Ensure WebSocket URL includes token parameter

**Metrics Not Updating:**

- Verify connection to `/metrics` endpoint
- Check for WebSocket connection errors in browser console
- Ensure token has proper permissions

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug pori --url wss://proxy.example.com --token your-token
```

This documentation covers the current API implementation. Features may be extended in future versions.

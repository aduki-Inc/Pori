# Pori Proxy Server Guide

This document provides comprehensive instructions for configuring and understanding how the Pori proxy server handles messages and forwards them between cloud servers and local applications.

## Overview

The Pori proxy server acts as a bridge between:

- **Cloud Server**: External WebSocket server that sends HTTP requests via tunnel
- **Local Applications**: Your local web servers, APIs, or applications
- **Dashboard**: Real-time monitoring and management interface

### Architecture Flow

```text
┌─────────────────┐    WebSocket     ┌─────────────────┐    HTTP      ┌─────────────────┐
│   Cloud Server  │ ◄──────────────► │  Pori Proxy     │ ◄──────────► │ Local Server    │
│                 │   TunnelMessage  │   (Forwarder)   │  HttpMessage │ (Your App)      │
└─────────────────┘                  └─────────────────┘              └─────────────────┘
                                              │
                                              ▼ HTTP
                                      ┌─────────────────┐
                                      │    Dashboard    │
                                      │  (Management)   │
                                      └─────────────────┘
```

## Message Processing Pipeline

### 1. Incoming Message Flow

```text
Cloud Server Request
        │
        ▼
WebSocket Connection (TunnelMessage)
        │
        ▼
Tunnel Handler (Authentication & Validation)
        │
        ▼
Message Parser (Extract HTTP Details)
        │
        ▼
HTTP Forwarder (Convert to Local HTTP)
        │
        ▼
Local Server (Your Application)
        │
        ▼
Response Handler (Convert Back to TunnelMessage)
        │
        ▼
WebSocket Response (Back to Cloud)
```

### 2. Message Types Handled

The proxy server processes these message types:

#### A. Authentication Messages

- **Purpose**: Establish secure connection with cloud server
- **Format**: Token-based or challenge-response authentication
- **Handler**: `TunnelHandler::handle_message()`

#### B. HTTP Request Messages

- **Purpose**: Forward HTTP requests from cloud to local server
- **Format**: Complete HTTP request with method, URL, headers, body
- **Handler**: `ProxyForwarder::forward_request()`

#### C. Control Messages

- **Purpose**: Health checks, configuration updates, statistics
- **Format**: Ping/pong, status updates, configuration changes
- **Handler**: `TunnelHandler::handle_message()`

#### D. Error Messages

- **Purpose**: Error reporting and recovery
- **Format**: Structured error with code, message, and recovery actions
- **Handler**: Global error handler

## Configuration Guide

### 1. Basic Configuration

Create a `pori.yml` configuration file:

```yaml
# Basic Proxy Configuration
proxy:
  # Local server settings
  local_server:
    host: "localhost"
    port: 3000
    protocol: "http"  # or "https"
    timeout_seconds: 30
    
  # WebSocket tunnel settings
  websocket:
    url: "wss://your-cloud-server.com/tunnel"
    token: "your-authentication-token"
    reconnect_attempts: 5
    reconnect_delay_seconds: 10
    
  # Dashboard settings
  dashboard:
    enabled: true
    port: 7616
    host: "localhost"
    
  # Logging configuration
  logging:
    level: "info"  # debug, info, warn, error
    format: "json"  # json or plain
```

### 2. Advanced Configuration

```yaml
# Advanced Proxy Configuration
proxy:
  local_server:
    host: "localhost"
    port: 3000
    protocol: "http"
    timeout_seconds: 30
    max_connections: 100
    keep_alive: true
    
    # Request transformation
    transform:
      # Add headers to all requests
      add_headers:
        "X-Forwarded-By": "pori-proxy"
        "X-Request-Source": "cloud"
        
      # Remove headers before forwarding
      remove_headers:
        - "X-Cloud-Internal"
        - "Authorization"  # if handling auth differently
        
      # URL rewriting
      url_rewrite:
        from: "/api/v1/"
        to: "/v1/"
        
  websocket:
    url: "wss://your-cloud-server.com/tunnel"
    token: "your-authentication-token"
    
    # Connection settings
    connection:
      reconnect_attempts: 5
      reconnect_delay_seconds: 10
      max_reconnect_delay_seconds: 300
      ping_interval_seconds: 30
      pong_timeout_seconds: 10
      
    # Message settings
    message:
      max_size_bytes: 1048576  # 1MB
      compression: true
      binary_mode: false
      
    # Security settings
    security:
      verify_ssl: true
      client_cert_path: "/path/to/client.crt"  # optional
      client_key_path: "/path/to/client.key"   # optional
      
  # Error handling
  error_handling:
    retry_failed_requests: true
    max_retries: 3
    retry_delay_ms: 1000
    
    # Custom error responses
    custom_errors:
      "502":
        message: "Local server unavailable"
        headers:
          "Content-Type": "application/json"
        body: '{"error": "Service temporarily unavailable"}'
        
  # Rate limiting
  rate_limiting:
    enabled: true
    requests_per_second: 100
    burst_size: 200
    
  # Monitoring
  monitoring:
    metrics_enabled: true
    stats_interval_seconds: 60
    health_check:
      endpoint: "/health"
      interval_seconds: 30
```

## Local Server Requirements

### 1. HTTP Server Compatibility

Your local server must support standard HTTP/1.1 or HTTP/2 protocols:

#### Minimum Requirements

- **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- **Status Codes**: Standard HTTP status codes (200, 404, 500, etc.)
- **Headers**: Accept and return standard HTTP headers
- **Content Types**: Support for JSON, HTML, plain text, binary data

#### Recommended Features

- **Keep-Alive**: Support connection reuse for better performance
- **Compression**: Support gzip/deflate compression
- **Large Payloads**: Handle requests/responses up to 10MB
- **Streaming**: Support chunked transfer encoding

### 2. Health Check Endpoint

Implement a health check endpoint for monitoring:

```http
GET /health HTTP/1.1
Host: localhost:3000

Response:
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "healthy",
  "timestamp": "2025-07-13T10:30:00Z",
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

### 3. Error Handling

Your local server should return appropriate HTTP status codes:

#### Success Responses

- **200 OK**: Successful request
- **201 Created**: Resource created successfully
- **204 No Content**: Successful request with no content

#### Client Error Responses

- **400 Bad Request**: Invalid request format
- **401 Unauthorized**: Authentication required
- **403 Forbidden**: Access denied
- **404 Not Found**: Resource not found
- **422 Unprocessable Entity**: Validation errors

#### Server Error Responses

- **500 Internal Server Error**: Server-side error
- **502 Bad Gateway**: Upstream service error
- **503 Service Unavailable**: Service temporarily down

## Message Handling Details

### Important: Request ID Handling

**Critical Requirement**: The cloud server includes a unique `requestId` field in every HTTP request message that must be preserved and returned in the response. This ID is used by the server to route responses back to the correct requester.

#### Request ID Format

- **Field Name**: `requestId`
- **Example Value**: `"RQT7C08F54C72A8"`
- **Location**: Required field within the HTTP payload (inside the `data` section of HTTP Request/Response messages)
- **Type**: Required String (not optional)
- **Purpose**: Links responses to their corresponding requests and enables proper routing by the cloud server

#### Request ID Flow

```text
1. Cloud Server → Proxy: HTTP request with requestId: "RQT7C08F54C72A8"
2. Proxy → Local Server: Forward request (automatically adds X-Request-ID header)
3. Local Server → Proxy: HTTP response
4. Proxy → Cloud Server: Response with same requestId: "RQT7C08F54C72A8"
```

⚠️ **Warning**: The requestId is now a required field in all HTTP payloads. The proxy automatically preserves and forwards this ID throughout the request-response cycle. Failing to maintain the requestId will cause the cloud server to be unable to route responses back to clients, resulting in timeouts and failed requests.

### 1. HTTP Request Processing

When the proxy receives an HTTP request from the cloud:

```json
{
  "step": "incoming_request",
  "requestId": "RQT7C08F54C72A8",
  "tunnel_message": {
    "envelope": {
      "tunnel_id": "tunnel_123",
      "client_id": "client_456"
    },
    "message": {
      "metadata": {
        "id": "req_789",
        "message_type": "http_request",
        "timestamp": 1673532000000
      },
      "payload": {
        "type": "Http",
        "data": {
          "type": "Request",
          "method": "POST",
          "url": "/api/users",
          "headers": {
            "content-type": "application/json",
            "authorization": "Bearer token123"
          },
          "body": {
            "name": "John Doe"
          },
          "requestId": "RQT7C08F54C72A8"
        }
      }
    }
  }
}
```

The proxy transforms this into a local HTTP request:

```http
POST /api/users HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Authorization: Bearer token123
X-Forwarded-By: pori-proxy
X-Request-ID: RQT7C08F54C72A8

{"name": "John Doe"}
```

**Implementation Details**: The proxy automatically:

- Extracts the `requestId` from the incoming HTTP payload (required field)
- Adds the `X-Request-ID` header with the `requestId` value for local server tracking
- Adds the `X-Forwarded-By: pori-proxy` header to identify the proxy
- Preserves the `requestId` in all response messages back to the cloud server

### 2. HTTP Response Processing

When your local server responds:

```http
HTTP/1.1 201 Created
Content-Type: application/json
Location: /api/users/123

{"id": 123, "name": "John Doe", "created_at": "2025-07-13T10:30:00Z"}
```

The proxy converts this back to a tunnel message:

```json
{
  "step": "outgoing_response",
  "requestId": "RQT7C08F54C72A8",
  "tunnel_message": {
    "envelope": {
      "tunnel_id": "tunnel_123",
      "client_id": "client_456"
    },
    "message": {
      "metadata": {
        "id": "resp_790",
        "correlation_id": "req_789",
        "message_type": "http_response",
        "timestamp": 1673532001000
      },
      "payload": {
        "type": "Http",
        "data": {
          "type": "Response",
          "status": 201,
          "status_text": "Created",
          "headers": {
            "content-type": "application/json",
            "location": "/api/users/123"
          },
          "body": {
            "id": 123,
            "name": "John Doe",
            "created_at": "2025-07-13T10:30:00Z"
          },
          "requestId": "RQT7C08F54C72A8"
        }
      }
    }
  }
}
```

**Critical**: Notice that the response includes the same `requestId` ("RQT7C08F54C72A8") that was in the original request. This is essential for the cloud server to route the response correctly.

### 3. Error Handling

If your local server returns an error:

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{"error": "Invalid user data", "details": "Name is required"}
```

The proxy forwards this as:

```json
{
  "step": "error_response",
  "requestId": "RQT7C08F54C72A8",
  "tunnel_message": {
    "envelope": {
      "tunnel_id": "tunnel_123",
      "client_id": "client_456"
    },
    "message": {
      "metadata": {
        "id": "err_791",
        "correlation_id": "req_789",
        "message_type": "http_response",
        "timestamp": 1673532001000
      },
      "payload": {
        "type": "Http",
        "data": {
          "type": "Response",
          "status": 400,
          "status_text": "Bad Request",
          "headers": {
            "content-type": "application/json"
          },
          "body": {
            "error": "Invalid user data",
            "details": "Name is required"
          },
          "requestId": "RQT7C08F54C72A8"
        }
      }
    }
  }
}
```

## Implementation Notes

### RequestId Handling

As of the current implementation, `requestId` is a **required field** in all HTTP Request and Response payloads. The system automatically:

- Extracts `requestId` from incoming HTTP requests within the payload data
- Preserves the `requestId` throughout the proxy forwarding process
- Automatically injects an `X-Request-ID` header into forwarded HTTP requests
- Includes the original `requestId` in all response messages back to the tunnel

This ensures complete request traceability and correlation between the cloud server, proxy, and local server components.

## Security Considerations

### 1. Authentication & Authorization

#### Token-Based Authentication

```yaml
websocket:
  token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  token_refresh_interval: 3600  # Refresh every hour
```

#### Certificate-Based Authentication

```yaml
websocket:
  security:
    client_cert_path: "/etc/pori/client.crt"
    client_key_path: "/etc/pori/client.key"
    ca_cert_path: "/etc/pori/ca.crt"
```

### 2. Request Validation

The proxy validates all incoming requests:

- **Message Format**: Ensures proper JSON structure
- **Required Fields**: Validates all required fields are present
- **Data Types**: Checks field types match expected values
- **Size Limits**: Enforces maximum message and payload sizes
- **Rate Limits**: Prevents abuse with rate limiting

### 3. Local Server Security

Protect your local server:

```yaml
proxy:
  local_server:
    # Only allow proxy connections
    allowed_hosts: ["127.0.0.1", "localhost"]
    
    # Add security headers
    transform:
      add_headers:
        "X-Forwarded-For": "proxy"
        "X-Real-IP": "127.0.0.1"
```

## Monitoring & Debugging

### 1. Dashboard Access

Access the dashboard at: `http://localhost:7616`

Features:

- **Real-time Statistics**: Request/response counts, error rates
- **Connection Status**: WebSocket connection health
- **Message Logs**: Recent message history
- **Performance Metrics**: Response times, throughput

### 2. Logging Configuration

```yaml
logging:
  level: "debug"  # For troubleshooting
  format: "json"
  outputs:
    - type: "console"
    - type: "file"
      path: "/var/log/pori/proxy.log"
      rotation:
        max_size_mb: 100
        max_files: 10
```

### 3. Health Monitoring

Monitor proxy health:

```bash
# Check proxy status
curl http://localhost:7616/api/status

# Check statistics
curl http://localhost:7616/api/stats

# Check recent errors
curl http://localhost:7616/api/errors
```

## Troubleshooting

### 1. Common Issues

#### Connection Problems

```text
Problem: WebSocket connection fails
Solutions:
- Check internet connectivity
- Verify WebSocket URL and token
- Check firewall settings
- Review SSL certificate configuration
```

#### Local Server Unreachable

```text
Problem: Cannot connect to local server
Solutions:
- Verify local server is running
- Check host and port configuration
- Test direct HTTP connection: curl http://localhost:3000
- Review local server logs
```

#### High Error Rates

```text
Problem: Many 5xx errors
Solutions:
- Check local server health
- Review local server logs
- Increase timeout values
- Monitor resource usage (CPU, memory)
```

### 2. Debug Mode

Enable debug logging for detailed troubleshooting:

```yaml
logging:
  level: "debug"
  
proxy:
  debug:
    log_requests: true
    log_responses: true
    log_websocket_frames: true
```

This will log all message details for debugging.

### 3. Testing Configuration

Test your configuration:

```bash
# Start proxy in test mode
./pori --config pori.yml --test

# Validate configuration
./pori --config pori.yml --validate

# Check connectivity
./pori --config pori.yml --check-connection
```

## Performance Optimization

### 1. Connection Pooling

```yaml
proxy:
  local_server:
    max_connections: 100
    connection_pool:
      idle_timeout: 30
      max_lifetime: 300
```

### 2. Compression

```yaml
proxy:
  websocket:
    message:
      compression: true
      compression_level: 6  # 1-9, higher = better compression
```

### 3. Caching

```yaml
proxy:
  cache:
    enabled: true
    ttl_seconds: 300
    max_size_mb: 100
    cache_headers: true
```

## Production Deployment

### 1. Systemd Service

Create `/etc/systemd/system/pori-proxy.service`:

```ini
[Unit]
Description=Pori Proxy Server
After=network.target

[Service]
Type=simple
User=pori
Group=pori
ExecStart=/usr/local/bin/pori --config /etc/pori/pori.yml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### 2. Docker Deployment

```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY pori /usr/local/bin/
COPY pori.yml /etc/pori/
EXPOSE 7616
CMD ["pori", "--config", "/etc/pori/pori.yml"]
```

### 3. Monitoring Integration

```yaml
monitoring:
  prometheus:
    enabled: true
    port: 9090
    path: "/metrics"
    
  logging:
    json_format: true
    structured_logs: true
```

This guide provides everything needed to configure your server to be compatible with the Pori proxy forwarder. The proxy handles all the complex message translation between the cloud WebSocket protocol and standard HTTP for your local applications.

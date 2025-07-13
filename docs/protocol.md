# Pori Protocol Message Structures

This document provides a comprehensive overview of all message structures used in the Pori proxy system for communication between components and external systems.

## Overview

The Pori proxy system uses a unified protocol message structure that supports various communication patterns:

- **WebSocket tunnel communication** between proxy client and cloud server
- **HTTP proxy forwarding** between local server and proxy client
- **Control and management** messages for system coordination
- **Authentication and authorization** for secure communication
- **Statistics and monitoring** for system health and performance

## Architecture

### Message Hierarchy

```text
ProtocolMessage (Base)
├── TunnelMessage (WebSocket Tunnel)
├── HttpMessage (HTTP Communication)
└── WebSocketMessage (WebSocket Communication)
```

### Component Communication Flow

```text
Cloud Server ←→ [WebSocket] ←→ Tunnel Handler ←→ HTTP Forwarder ←→ [HTTP] ←→ Local Server
                    ↓                             ↓
              TunnelMessage                 HttpMessage
                    ↓                             ↓
              ProtocolMessage ←------ Bridge ------→ ProtocolMessage
```

## Base Protocol Message Structure

### ProtocolMessage

The base message structure for all communications:

```json
{
  "metadata": {
    "id": "msg_12345",
    "message_type": "http_request",
    "version": "1.0",
    "timestamp": 1673532000000,
    "priority": "Normal",
    "delivery_mode": "AtLeastOnce",
    "encoding": "JSON",
    "source": "proxy-client",
    "destination": "local-server",
    "correlation_id": "req_67890",
    "session_id": "session_abc123",
    "headers": {
      "user-agent": "pori-proxy/1.0",
      "custom-header": "value"
    },
    "tags": ["http", "proxy"],
    "retry_count": 0,
    "max_retries": 3,
    "ttl": 300
  },
  "payload": {
    "type": "Http",
    "data": {
      // Payload-specific data structure
    }
  }
}
```

#### MessageMetadata Fields

- **id**: Unique message identifier
- **message_type**: Message type identifier
- **version**: Protocol version
- **timestamp**: Unix timestamp in milliseconds
- **priority**: Message priority (Low, Normal, High, Critical)
- **delivery_mode**: Delivery guarantees
- **encoding**: Encoding format (JSON, MessagePack, etc.)
- **source**: Source identifier (optional)
- **destination**: Destination identifier (optional)
- **correlation_id**: For request-response tracking (optional)
- **session_id**: Session identifier (optional)
- **headers**: Custom headers
- **tags**: Message tags for routing
- **retry_count**: Current retry count
- **max_retries**: Maximum retry attempts
- **ttl**: Time-to-live in seconds (optional)

#### MessagePayload Types

- **Auth**: Authentication messages
- **Http**: HTTP request/response
- **Control**: Control and management
- **Stats**: Statistics and monitoring
- **Error**: Error messages
- **Stream**: Streaming data
- **Custom**: Custom application messages

## Message Types

### 1. Authentication Messages (AuthPayload)

Used for authentication and authorization:

#### Token Authentication

```json
{
  "type": "TokenAuth",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "scopes": ["tunnel", "proxy", "admin"]
}
```

#### Challenge Authentication

```json
{
  "type": "Challenge",
  "challenge": "abc123def456",
  "method": "HMAC-SHA256"
}
```

#### Challenge Response

```json
{
  "type": "Response",
  "response": "9f8e7d6c5b4a3210fedcba0987654321",
  "proof": "additional_proof_data"
}
```

#### Authentication Success

```json
{
  "type": "Success",
  "session_id": "session_xyz789",
  "expires_at": 1673535600000,
  "permissions": ["tunnel.read", "tunnel.write", "stats.read"]
}
```

#### Authentication Failure

```json
{
  "type": "Failure",
  "error_code": "INVALID_TOKEN",
  "error_message": "The provided token is invalid or expired",
  "retry_after": 300
}
```

**Use Cases:**

- Initial authentication with cloud server
- Token refresh and validation
- Permission verification
- Session management

### 2. HTTP Messages (HttpPayload)

For HTTP request/response communication:

#### HTTP Request

```json
{
  "type": "Request",
  "method": "GET",
  "url": "/api/users/123",
  "headers": {
    "host": "api.example.com",
    "authorization": "Bearer token123",
    "content-type": "application/json",
    "user-agent": "pori-proxy/1.0"
  },
  "body": null,
  "query_params": {
    "format": "json",
    "include": "profile"
  }
}
```

#### HTTP Response

```json
{
  "type": "Response",
  "status": 200,
  "status_text": "OK",
  "headers": {
    "content-type": "application/json",
    "content-length": "256",
    "cache-control": "no-cache"
  },
  "body": "eyJpZCI6MTIzLCJuYW1lIjoiSm9obiBEb2UifQ=="
}
```

#### Protocol Upgrade

```json
{
  "type": "Upgrade",
  "protocol": "websocket",
  "headers": {
    "upgrade": "websocket",
    "connection": "upgrade",
    "sec-websocket-key": "dGhlIHNhbXBsZSBub25jZQ==",
    "sec-websocket-version": "13"
  }
}
```

#### Connection Close

```json
{
  "type": "Close",
  "reason": "Connection timeout"
}
```

**Use Cases:**

- Forwarding HTTP requests from cloud to local server
- Sending HTTP responses back to cloud
- WebSocket upgrade requests
- Connection management

### 3. Control Messages (ControlPayload)

For system control and management:

#### Ping

```json
{
  "type": "Ping",
  "timestamp": 1673532000000,
  "data": "cGluZyBkYXRh"
}
```

#### Pong

```json
{
  "type": "Pong",
  "timestamp": 1673532001000,
  "data": "cG9uZyBkYXRh"
}
```

#### Status Update

```json
{
  "type": "Status",
  "status": "Connected",
  "message": "Connection established successfully",
  "details": {
    "connection_time": "2023-01-12T10:30:00Z",
    "protocol_version": "1.0",
    "compression_enabled": "true"
  }
}
```

#### Configuration Update

```json
{
  "type": "Config",
  "config_type": "tunnel_settings",
  "config_data": {
    "compression": true,
    "timeout_seconds": 30,
    "max_retries": 3
  }
}
```

#### Command Execution

```json
{
  "type": "Command",
  "command": "restart",
  "arguments": ["--graceful", "--timeout=30"],
  "environment": {
    "LOG_LEVEL": "info",
    "CONFIG_PATH": "/etc/pori"
  }
}
```

#### Shutdown

```json
{
  "type": "Shutdown",
  "reason": "Maintenance window",
  "grace_period_seconds": 300
}
```

**Use Cases:**

- Connection health monitoring (ping/pong)
- Status updates and notifications
- Configuration updates
- Remote command execution
- Graceful shutdown coordination

### 4. Statistics Messages (StatsPayload)

For monitoring and metrics:

#### Connection Statistics

```json
{
  "type": "Connection",
  "connected_clients": 42,
  "total_connections": 1337,
  "failed_connections": 13,
  "uptime_seconds": 86400
}
```

#### Traffic Statistics

```json
{
  "type": "Traffic",
  "requests_processed": 10000,
  "requests_successful": 9850,
  "requests_failed": 150,
  "bytes_transferred": 52428800,
  "average_response_time_ms": 125.5
}
```

#### System Statistics

```json
{
  "type": "System",
  "cpu_usage_percent": 15.7,
  "memory_usage_bytes": 536870912,
  "memory_total_bytes": 8589934592,
  "disk_usage_bytes": 1073741824,
  "network_bytes_sent": 104857600,
  "network_bytes_received": 209715200
}
```

#### Custom Metrics

```json
{
  "type": "Custom",
  "metric_name": "active_tunnels",
  "metric_value": 25,
  "metric_type": "gauge",
  "labels": {
    "region": "us-east-1",
    "environment": "production"
  }
}
```

**Use Cases:**

- Real-time performance monitoring
- Health checks and alerting
- Resource usage tracking
- Custom application metrics

### 5. Error Messages (ErrorPayload)

For error reporting and handling:

```json
{
  "code": "HTTP_502",
  "message": "Bad Gateway - Unable to connect to upstream server",
  "details": "Connection timeout after 30 seconds",
  "trace": "at proxy::forwarder::forward_request:142\nat proxy::client::send_request:89",
  "related_id": "req_abc123",
  "category": "External",
  "recovery_actions": [
    "Retry the request",
    "Check upstream server status",
    "Contact system administrator"
  ]
}
```

#### Error Categories

- **Authentication**: Auth/authorization errors
- **Authorization**: Permission errors  
- **Network**: Network connectivity errors
- **Protocol**: Protocol violation errors
- **Validation**: Data validation errors
- **Internal**: Internal system errors
- **External**: External service errors
- **Timeout**: Timeout errors
- **RateLimit**: Rate limiting errors

**Use Cases:**

- Error reporting and logging
- Client error notifications
- Debugging and troubleshooting
- Error recovery guidance

### 6. Stream Messages (StreamPayload)

For streaming data communication:

#### Stream Data

```json
{
  "type": "Data",
  "stream_id": "stream_xyz789",
  "sequence": 42,
  "data": "SGVsbG8gd29ybGQgZnJvbSBzdHJlYW0gY2h1bms=",
  "is_final": false
}
```

#### Stream Control

```json
{
  "type": "Control",
  "stream_id": "stream_xyz789",
  "action": "Pause",
  "metadata": {
    "reason": "Flow control",
    "resume_after": "5000"
  }
}
```

#### Stream Actions

- **Start**: Start stream
- **Pause**: Pause stream
- **Resume**: Resume stream
- **Stop**: Stop stream
- **Reset**: Reset stream

**Use Cases:**

- Large file transfers
- Real-time data streaming
- Chunked response handling
- Flow control management

## Specialized Message Wrappers

### TunnelMessage

WebSocket tunnel communication wrapper:

```json
{
  "envelope": {
    "tunnel_id": "tunnel_abc123",
    "client_id": "client_xyz789", 
    "server_id": "server_def456",
    "protocol_version": "1.0",
    "compression": "gzip",
    "encryption": "aes256",
    "routing": {
      "priority": "high",
      "route_type": "direct"
    }
  },
  "message": {
    "metadata": {
      "id": "msg_12345",
      "message_type": "http_request",
      "version": "1.0",
      "timestamp": 1673532000000
    },
    "payload": {
      "type": "Http",
      "data": {
        "type": "Request",
        "method": "GET",
        "url": "/api/status"
      }
    }
  }
}
```

**Use Cases:**

- WebSocket communication with cloud server
- Tunnel establishment and management
- Client identification and routing
- Compression and encryption metadata

### HttpMessage

HTTP communication wrapper:

```json
{
  "envelope": {
    "connection_id": "conn_abc123",
    "http_version": "1.1",
    "keep_alive": true,
    "proxy_info": {
      "proxy_host": "proxy.example.com",
      "proxy_port": 8080,
      "upstream_host": "api.example.com",
      "upstream_port": 443
    },
    "timing": {
      "request_start": 1673532000000,
      "dns_lookup_time": 12,
      "connection_time": 45,
      "tls_handshake_time": 78
    }
  },
  "message": {
    "metadata": {
      "id": "msg_67890",
      "message_type": "http_response",
      "version": "1.0",
      "timestamp": 1673532001000
    },
    "payload": {
      "type": "Http",
      "data": {
        "type": "Response",
        "status": 200,
        "status_text": "OK"
      }
    }
  }
}
```

**Use Cases:**

- HTTP proxy forwarding
- Local server communication
- Performance monitoring
- Connection management

### WebSocketMessage

WebSocket communication wrapper:

```json
{
  "envelope": {
    "connection_id": "ws_conn_abc123",
    "ws_version": "13",
    "frame_type": "Text",
    "compression": {
      "algorithm": "deflate",
      "enabled": true,
      "compression_ratio": 0.65
    },
    "extensions": {
      "permessage-deflate": "server_max_window_bits=15",
      "x-webkit-deflate-frame": "disabled"
    }
  },
  "message": {
    "metadata": {
      "id": "msg_ws001",
      "message_type": "websocket_data",
      "version": "1.0",
      "timestamp": 1673532002000
    },
    "payload": {
      "type": "Custom",
      "data": {
        "frame_data": "SGVsbG8gV2ViU29ja2V0IQ==",
        "opcode": "text"
      }
    }
  }
}
```

**Use Cases:**

- WebSocket client communication
- Frame type management
- Compression handling
- Extension negotiation

## Configuration Structures

### Protocol Configuration

Global protocol settings:

```json
{
  "protocol_config": {
    "version": "1.0",
    "compression": {
      "enabled": true,
      "algorithm": "gzip",
      "level": 6
    },
    "security": {
      "encryption": "aes256",
      "require_auth": true,
      "session_timeout": 3600
    },
    "timeouts": {
      "connection_timeout": 30,
      "request_timeout": 60,
      "keepalive_timeout": 300
    },
    "limits": {
      "max_message_size": 1048576,
      "max_connections": 1000,
      "rate_limit_per_second": 100
    },
    "features": {
      "compression_enabled": true,
      "metrics_enabled": true,
      "debug_logging": false
    }
  }
}
```

### Tunnel Configuration

WebSocket tunnel settings:

```json
{
  "tunnel_config": {
    "settings": {
      "tunnel_id": "default-tunnel",
      "max_reconnect_attempts": 5,
      "reconnect_delay_seconds": 10
    },
    "auth": {
      "auth_type": "token",
      "token_refresh_interval": 1800,
      "require_client_cert": false
    },
    "network": {
      "compression": true,
      "keepalive_interval": 30,
      "max_frame_size": 65536
    },
    "security": {
      "tls_version": "1.3",
      "verify_hostname": true,
      "cipher_suites": ["TLS_AES_256_GCM_SHA384"]
    }
  }
}
```

### HTTP Configuration

HTTP communication settings:

```json
{
  "http_config": {
    "timeouts": {
      "connect_timeout": 10,
      "read_timeout": 30,
      "write_timeout": 30,
      "idle_timeout": 90
    },
    "retry": {
      "max_retries": 3,
      "retry_delay_ms": 1000,
      "backoff_multiplier": 2.0
    },
    "cache": {
      "enabled": true,
      "max_size_mb": 100,
      "ttl_seconds": 300
    },
    "connection": {
      "pool_size": 10,
      "keep_alive": true,
      "tcp_nodelay": true
    }
  }
}
```

## Message Flow Examples

### 1. HTTP Request Forwarding

```text
Cloud Server → WebSocket → Proxy Client → HTTP → Local Server
```

1. **Cloud to Proxy**: `TunnelMessage` with `HttpPayload::Request`
2. **Proxy Processing**: Convert to `HttpMessage` with `HttpPayload::Request`
3. **Proxy to Local**: Standard HTTP request
4. **Local to Proxy**: Standard HTTP response
5. **Proxy Processing**: Convert to `HttpMessage` with `HttpPayload::Response`
6. **Proxy to Cloud**: `TunnelMessage` with `HttpPayload::Response`

### 2. Authentication Flow

```text
Proxy Client → WebSocket → Cloud Server
```

1. **Initial Auth**: `TunnelMessage` with `AuthPayload::TokenAuth`
2. **Server Response**: `TunnelMessage` with `AuthPayload::Success`
3. **Session Established**: Normal message flow begins

### 3. Health Monitoring

```text
Both Directions: Proxy Client ↔ Cloud Server
```

1. **Ping**: `TunnelMessage` with `ControlPayload::Ping`
2. **Pong**: `TunnelMessage` with `ControlPayload::Pong`
3. **Stats**: `TunnelMessage` with `StatsPayload::Traffic`

### 4. Error Handling

```text
Any Component → Error Handler → Response
```

1. **Error Occurs**: System generates `ErrorPayload`
2. **Error Wrapped**: In appropriate message wrapper
3. **Error Sent**: To originating component or monitoring system
4. **Recovery**: Based on error category and recovery actions

## Implementation Examples

### Message Creation

```json
{
  "action": "create_http_request",
  "message": {
    "metadata": {
      "id": "req_12345",
      "message_type": "http_request",
      "version": "1.0",
      "timestamp": 1673532000000,
      "priority": "Normal"
    },
    "payload": {
      "type": "Http",
      "data": {
        "type": "Request",
        "method": "GET",
        "url": "/api/data",
        "headers": {
          "authorization": "Bearer token123"
        }
      }
    }
  },
  "tunnel_wrapper": {
    "tunnel_id": "tunnel-1",
    "client_id": "client-1"
  }
}
```

### Message Processing

```json
{
  "action": "process_incoming_message",
  "received_message": {
    "envelope": {
      "tunnel_id": "tunnel-1",
      "client_id": "client-1"
    },
    "message": {
      "payload": {
        "type": "Http",
        "data": {
          "type": "Request",
          "method": "POST",
          "url": "/api/users"
        }
      }
    }
  },
  "processing_result": {
    "action": "forward_to_local_server",
    "target_url": "http://localhost:3000/api/users",
    "method": "POST"
  }
}
```

### Error Handling

```json
{
  "action": "create_error_response",
  "error_details": {
    "code": "HTTP_502",
    "message": "Bad Gateway",
    "category": "External",
    "related_id": "req_12345"
  },
  "error_message": {
    "metadata": {
      "id": "err_67890",
      "message_type": "error",
      "version": "1.0",
      "timestamp": 1673532001000
    },
    "payload": {
      "type": "Error",
      "data": {
        "code": "HTTP_502",
        "message": "Bad Gateway",
        "category": "External",
        "related_id": "req_12345"
      }
    }
  }
}
```

## Security Considerations

### Message Validation

- All incoming messages MUST be validated against schema
- Message size limits MUST be enforced
- Malformed messages MUST be rejected with appropriate errors

### Authentication

- All tunnel connections MUST authenticate before message exchange
- Authentication tokens MUST be validated and refreshed as needed
- Unauthorized messages MUST be dropped

### Rate Limiting

- Message rate limits SHOULD be enforced per connection
- Burst limits SHOULD be configured to prevent abuse
- Rate limit violations SHOULD result in temporary blocks

### Data Sanitization

- All user-provided data MUST be sanitized
- HTTP headers MUST be validated and filtered
- Binary data MUST be scanned for malicious content

## Monitoring and Debugging

### Message Logging

- All messages SHOULD be logged with appropriate detail level
- Sensitive data MUST be redacted from logs
- Log correlation IDs SHOULD be used for tracing

### Metrics Collection

- Message counts and sizes SHOULD be tracked
- Error rates and types SHOULD be monitored
- Performance metrics SHOULD be collected

### Debug Information

- Debug builds MAY include additional message metadata
- Production builds MUST minimize debug overhead
- Debug information SHOULD be configurable

## Future Extensions

### Planned Features

- **Message Compression**: Automatic compression for large messages
- **Message Encryption**: End-to-end encryption for sensitive data
- **Message Routing**: Advanced routing based on content and metadata
- **Message Persistence**: Durable message queuing for reliability

### Extensibility

- **Custom Payloads**: Support for application-specific message types
- **Plugin System**: Message processing plugins and middlewares
- **Schema Evolution**: Backward-compatible schema versioning
- **Performance Optimization**: Zero-copy serialization and advanced encoding

This protocol specification provides a robust foundation for all communication within the Pori proxy system while maintaining flexibility for future enhancements and use cases.

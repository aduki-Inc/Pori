# Server Message Format Requirements

This document outlines the required message format for the tunnel server to ensure proper communication with the Pori proxy client.

## Current Issues

The proxy client is failing to parse incoming messages because the server's message format doesn't match the expected client protocol structure.

**Common parsing errors:**

- Missing required metadata fields (`version`, `priority`, `delivery_mode`, `encoding`)
- Incorrect field names (`type` instead of `kind`)
- Wrong case sensitivity (`Http` instead of `HTTP`)

## Required Message Structure

### HTTP Request Message

```json
{
  "envelope": {
    "tunnel_id": "tunnel_1752411737413",
    "client_id": "client_msg_e262b1fcd5f749a7"
  },
  "message": {
    "metadata": {
      "id": "msg_e262b1fcd5f749a7",
      "message_type": "http_request",
      "version": "1.0.0",
      "timestamp": 1752411737413,
      "priority": "normal",
      "delivery_mode": "at_least_once",
      "encoding": "json"
    },
    "payload": {
      "kind": "HTTP",
      "data": {
        "kind": "Request",
        "method": "GET",
        "url": "/hh",
        "headers": {
          "connection": "upgrade",
          "host": "pay.vpu.world",
          "user-agent": "Mozilla/5.0..."
        },
        "body": null,
        "requestId": "R0X39398B820AB9"
      }
    }
  }
}
```

### Authentication Message

```json
{
  "envelope": {
    "tunnel_id": "tunnel_1752412958539",
    "client_id": "client_msg_26a040a2f1491df0"
  },
  "message": {
    "metadata": {
      "id": "msg_26a040a2f1491df0",
      "message_type": "auth",
      "version": "1.0.0",
      "timestamp": 1752412958539,
      "priority": "normal",
      "delivery_mode": "at_least_once",
      "encoding": "json"
    },
    "payload": {
      "kind": "Control",
      "data": {
        "kind": "Authentication",
        "status": "authenticated",
        "message": "Connection established successfully",
        "timestamp": "2025-07-13T13:22:38.539Z"
      }
    }
  }
}
```

## Required Fields

### Message Metadata (ALL REQUIRED)

| Field | Type | Description | Example Value |
|-------|------|-------------|---------------|
| `id` | string | Unique message identifier | `"msg_e262b1fcd5f749a7"` |
| `message_type` | string | Type of message | `"http_request"`, `"auth"` |
| `version` | string | Protocol version | `"1.0.0"` |
| `timestamp` | number | Unix timestamp in milliseconds | `1752411737413` |
| `priority` | string | Message priority | `"normal"` |
| `delivery_mode` | string | Delivery guarantee | `"at_least_once"` |
| `encoding` | string | Message encoding format | `"json"` |

### Enum Values (CASE SENSITIVE)

**Priority Values:**

- `"low"`
- `"normal"` *(recommended)*
- `"high"`
- `"critical"`

**Delivery Mode Values:**

- `"fire_and_forget"`
- `"at_most_once"`
- `"at_least_once"` *(recommended)*
- `"exactly_once"`

**Encoding Values:**

- `"json"` *(recommended)*
- `"binary"`
- `"compressed"`

### Payload Structure

**HTTP Messages:**

- `payload.kind` = `"HTTP"` (uppercase)
- `payload.data.kind` = `"Request"` or `"Response"`
- `payload.data.requestId` = string (required for all HTTP messages)

**Control Messages:**

- `payload.kind` = `"Control"`
- `payload.data.kind` = `"Authentication"`, `"Ping"`, `"Pong"`, etc.

## Server Implementation Template

### JavaScript/Node.js

```javascript
function createMessage(messageType, payloadType, payloadData) {
  return {
    envelope: {
      tunnel_id: `tunnel_${Date.now()}`,
      client_id: `client_msg_${generateId()}`
    },
    message: {
      metadata: {
        id: `msg_${generateId()}`,
        message_type: messageType,
        version: "1.0.0",
        timestamp: Date.now(),
        priority: "normal",
        delivery_mode: "at_least_once",
        encoding: "json"
      },
      payload: {
        kind: payloadType,
        data: payloadData
      }
    }
  };
}

// HTTP Request Example
const httpRequest = createMessage("http_request", "HTTP", {
  kind: "Request",
  method: "GET",
  url: "/api/users",
  headers: { "host": "example.com" },
  body: null,
  requestId: generateRequestId()
});

// Authentication Example
const authMessage = createMessage("auth", "Control", {
  kind: "Authentication",
  status: "authenticated",
  message: "Connection established successfully",
  timestamp: new Date().toISOString()
});
```

### Python

```python
import time
import json
from typing import Dict, Any

def create_message(message_type: str, payload_type: str, payload_data: Dict[str, Any]) -> Dict[str, Any]:
    return {
        "envelope": {
            "tunnel_id": f"tunnel_{int(time.time() * 1000)}",
            "client_id": f"client_msg_{generate_id()}"
        },
        "message": {
            "metadata": {
                "id": f"msg_{generate_id()}",
                "message_type": message_type,
                "version": "1.0.0",
                "timestamp": int(time.time() * 1000),
                "priority": "normal",
                "delivery_mode": "at_least_once",
                "encoding": "json"
            },
            "payload": {
                "kind": payload_type,
                "data": payload_data
            }
        }
    }

# HTTP Request Example
http_request = create_message("http_request", "HTTP", {
    "kind": "Request",
    "method": "GET",
    "url": "/api/users",
    "headers": {"host": "example.com"},
    "body": None,
    "requestId": generate_request_id()
})

# Authentication Example
auth_message = create_message("auth", "Control", {
    "kind": "Authentication",
    "status": "authenticated",
    "message": "Connection established successfully",
    "timestamp": time.strftime('%Y-%m-%dT%H:%M:%S.%fZ')
})
```

## Critical Requirements Summary

**MUST FIX for proxy to work:**

1. ✅ **Add missing metadata fields:** `version`, `priority`, `delivery_mode`, `encoding`
2. ✅ **Use correct case:** `"HTTP"` (not `"Http"`), lowercase enums (`"normal"`, `"at_least_once"`)
3. ✅ **Use `kind` field:** Replace all `"type"` with `"kind"` in payload data
4. ✅ **Include requestId:** Required for all HTTP messages

## Testing

Once implemented, the proxy should:

- ✅ Parse all messages without errors
- ✅ Log `INCOMING REQUEST: GET /path` for HTTP requests
- ✅ Forward requests to local server
- ✅ Send responses back through tunnel

## Priority

🚨 **BLOCKING ISSUE** - Proxy cannot function until these changes are implemented.

## Support

For questions, refer to:

- `docs/protocol.md` - Complete protocol documentation
- `docs/proxy.md` - Proxy implementation guide
- `src/protocol/messages.rs` - Message structure definitions

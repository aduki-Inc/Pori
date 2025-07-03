# WebSocket Message Types for Proxy Client

This document outlines all WebSocket message types that the proxy client expects to receive and all message types it sends back to the proxy server.

## Messages Client EXPECTS to Receive from Server

### 1. Authentication Status

```json
{
  "type": "auth",
  "status": "authenticated",
  "timestamp": "2025-07-03T12:31:57.566Z"
}
```

### 2. HTTP Request (Most common - from server to client)

```json
{
  "type": "request",
  "requestId": "RQT7C08F54C72A8",
  "method": "GET",
  "url": "/456",
  "path": "/456",
  "query": {},
  "headers": {
    "host": "example.com",
    "user-agent": "Mozilla/5.0...",
    "accept": "text/html,application/xhtml+xml"
  },
  "body": null,
  "timestamp": "2025-07-03T12:32:16.616Z"
}
```

### 3. Server Error Messages

```json
{
  "type": "error",
  "message": "Request processing failed",
  "code": "ERROR_CODE",
  "requestId": "RQT7C08F54C72A8"
}
```

## Messages Client SENDS to Server

### 1. Response (Most common - after processing request)

```json
{
  "type": "response",
  "data": {
    "id": "RQT7C08F54C72A8",
    "status": 200,
    "status_text": "OK",
    "headers": {
      "content-type": "text/html; charset=utf-8",
      "content-length": "1234"
    },
    "body": null
  }
}
```

### 2. Error Response (when local server fails)

```json
{
  "type": "response",
  "data": {
    "id": "RQT7C08F54C72A8", 
    "status": 502,
    "status_text": "Bad Gateway",
    "headers": {
      "content-type": "application/json"
    },
    "body": null
  }
}
```

### 3. Client Error Messages

```json
{
  "type": "error",
  "data": {
    "request_id": "RQT7C08F54C72A8",
    "error": "Failed to connect to local server",
    "code": 502
  }
}
```

## Message Types NOT Supported

The following message types are **NOT expected** by the server and will cause "Unknown message type" errors:

- `ping` - Client no longer sends automatic ping messages
- `pong` - Not used for regular communication  
- `auth_success`, `auth_error` - Use simple `auth` with `status` field
- Any custom message types not listed above

## Common Error Scenarios

### Local Server Unavailable

When the local server (localhost:3000) is not running or unreachable:

**Client logs:**

```text
2025-07-03 16:39:52.581  LOCAL: handle_http_request: Forwarding request to local server: GET /456 (ID: RQT7C08F54C72A8)
2025-07-03 16:39:52.581  ERROR pori::proxy::forwarder: handle_http_request: Local server error: GET /456 -> Error: Failed to send request to local server (530.801µs)
2025-07-03 16:39:52.581  LOCAL: handle_http_request: Sending error response: GET /456 -> 502 Bad Gateway (ID: RQT7C08F54C72A8)
```

**Client response:**

```json
{
  "type": "response", 
  "data": {
    "id": "RQT7C08F54C72A8",
    "status": 502,
    "status_text": "Bad Gateway", 
    "headers": {
      "content-type": "application/json"
    },
    "body": null
  }
}
```

### Server Message Format Issues

If server sends "Unknown message type" errors, check:

1. **Message structure** - Use flat JSON format for incoming messages: `{"type": "request", "requestId": "...", ...}`
2. **Supported types** - Only use types listed in "Client EXPECTS" section
3. **Field names** - Use exact field names as shown (e.g., `requestId` not `id`)

## Message Flow Examples

### Successful Request Flow

1. **Server → Client:** HTTP request

   ```json
   {"type": "request", "requestId": "REQ123", "method": "GET", "url": "/api/users", "path": "/api/users", "headers": {"host": "example.com"}, "body": null}
   ```

2. **Client → Local Server:** Forward HTTP request to localhost:3000

3. **Local Server → Client:** HTTP response (200 OK)

4. **Client → Server:** Response  

   ```json
   {"type": "response", "data": {"id": "REQ123", "status": 200, "status_text": "OK", "headers": {"content-type": "text/html"}, "body": null}}
   ```

### Failed Request Flow (Local Server Down)

1. **Server → Client:** HTTP request

   ```json
   {"type": "request", "requestId": "REQ456", "method": "GET", "url": "/api/data", "path": "/api/data", "headers": {"host": "example.com"}, "body": null}
   ```

2. **Client:** Try to connect to localhost:3000 → Connection failed

3. **Client → Server:** Error response

   ```json
   {"type": "response", "data": {"id": "REQ456", "status": 502, "status_text": "Bad Gateway", "headers": {"content-type": "application/json"}, "body": null}}
   ```

## Implementation Notes

### Client Behavior

- **Authentication:** Handles auth messages with status field
- **Request Processing:** Forwards all HTTP requests to configured local server (default: localhost:3000)
- **Error Handling:** Returns 502 Bad Gateway when local server unavailable
- **Logging:** Structured logging with request IDs for traceability
- **No Pings:** Does not send automatic ping messages to reduce noise

### Server Requirements

1. **Use flat JSON format for requests:** `{"type": "request", "requestId": "...", "method": "GET", ...}`
2. **Handle nested responses:** Accept responses in nested format with `data` field
3. **Provide meaningful errors:** Use error messages with request context
4. **Support binary data:** Handle file uploads/downloads via body field
5. **Authentication flow:** Send auth messages with status field

This documentation ensures compatibility between the proxy client and your server implementation.

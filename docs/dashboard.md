# Dashboard Guide: hyper.rs HTTP Server

## Overview

Local HTTP server built with hyper.rs that provides:

- Web dashboard for monitoring tunnel status
- REST API for configuration and control
- Static file serving for HTML/CSS/JS
- WebSocket upgrade for real-time updates

## Server Architecture

### hyper.rs Service Implementation

> Create HTTP service using hyper.rs with custom routing

**Service responsibilities:**
- Route requests to appropriate handlers
- Serve static dashboard files
- Handle API endpoints
- Upgrade connections to WebSocket for real-time updates

### Static File Serving

> Embed dashboard assets at compile time using include_dir

**Static asset structure:**
- index.html (Main dashboard page)
- CSS files (Styling and responsive design)
- JavaScript files (Dashboard functionality and WebSocket client)
- Assets (Icons, images, fonts)

**MIME type detection:**
- Use mime_guess crate for proper Content-Type headers
- Support common web file types
- Cache-friendly headers for static assets

### API Endpoints

> Implement REST API for dashboard functionality

**Status endpoints:**
- GET /api/status (Current tunnel status)
- GET /api/stats (Connection statistics)
- GET /api/config (Current configuration)

**Control endpoints:**
- POST /api/reconnect (Force WebSocket reconnection)
- POST /api/shutdown (Graceful shutdown)

**Response format:**
- JSON responses for all API endpoints
- Consistent error handling
- Proper HTTP status codes

### WebSocket Upgrade

> Handle WebSocket upgrade for real-time dashboard updates

**Upgrade process:**
- Check for WebSocket headers
- Validate connection request
- Upgrade HTTP connection to WebSocket
- Handle bidirectional message flow

**Real-time features:**
- Live connection status updates
- Request/response statistics
- Error notifications
- Performance metrics

## Implementation Steps

### Step 1: Service Structure

> Create src/server/mod.rs with main service implementation

**Service trait implementation:**
- Handle incoming HTTP requests
- Route to appropriate handlers
- Return proper HTTP responses
- Handle errors gracefully

### Step 2: Static File Handler

> Implement src/server/static_files.rs

**File serving logic:**
- Extract file path from URL
- Look up embedded file content
- Set appropriate MIME type
- Return file content or 404

**Security considerations:**
- Path traversal protection
- Only serve files from static directory
- Validate file extensions

### Step 3: API Handler

> Create src/server/api.rs for REST endpoints

**Endpoint implementations:**
- Parse request paths and methods
- Extract query parameters
- Call appropriate business logic
- Format JSON responses

### Step 4: Dashboard Handler

> Implement src/server/dashboard.rs

**Dashboard routing:**
- Serve index.html for root path
- Handle dashboard-specific routes
- WebSocket upgrade handling
- Error page rendering

## Configuration

### Server Binding

> Bind server to localhost only for security

**Default configuration:**
- Listen on 127.0.0.1 (localhost only)
- Default port 7616 (configurable via CLI)
- No external network access

### CORS Headers

> Set proper CORS headers for browser compatibility

**CORS configuration:**
- Allow origin from localhost
- Support preflight requests
- Allow necessary headers
- Handle OPTIONS method

### Error Handling

> Implement comprehensive error handling

**Error responses:**
- 404 for missing static files
- 500 for server errors
- 400 for bad requests
- JSON error format for API

## Dashboard Frontend

### HTML Structure

> Create single-page dashboard application

**Dashboard sections:**
- Connection status indicator
- Request/response statistics
- Configuration display
- Error log display
- Control buttons

### CSS Styling

> Responsive design with modern styling

**Design principles:**
- Clean, minimal interface
- Responsive layout for different screen sizes
- Status indicators with colors
- Professional appearance

### JavaScript Functionality

> Interactive dashboard with WebSocket updates

**Core features:**
- WebSocket connection to server
- Real-time status updates
- Request statistics visualization
- Configuration display
- Manual control buttons

**WebSocket handling:**
- Automatic reconnection on disconnect
- Message parsing and display
- Error handling and user feedback
- Connection status indicators

## Integration with Application State

### Shared State Access

> Access application state from HTTP handlers

**State management:**
- Use Arc<Mutex<AppState>> for thread-safe access
- Share connection status between components
- Update statistics from WebSocket events
- Coordinate between HTTP and WebSocket components

### Real-time Updates

> Push updates to dashboard via WebSocket

**Update triggers:**
- WebSocket connection state changes
- New HTTP requests processed
- Error conditions
- Configuration changes

## Security Considerations

### Local-only Access

> Ensure dashboard is only accessible locally

**Security measures:**
- Bind only to 127.0.0.1
- No authentication required (local access only)
- Input validation for all API endpoints
- Rate limiting for API calls

### Static File Security

> Secure static file serving

**Protection measures:**
- Path traversal prevention
- Only serve embedded files
- Validate file extensions
- No directory listing

## Performance Optimization

### Efficient Serving

> Optimize static file and API performance

**Optimization techniques:**
- Cache embedded files in memory
- Compress responses where beneficial
- Use efficient routing without regex
- Minimize allocations in hot paths

### Connection Management

> Handle concurrent connections efficiently

**Connection handling:**
- Async request processing
- Connection pooling where applicable
- Graceful connection closure
- Resource cleanup on disconnect

## Testing Strategy

### Unit Tests

> Test individual handler components

**Test coverage:**
- Static file serving logic
- API endpoint responses
- Error handling scenarios
- WebSocket upgrade process

### Integration Tests

> Test full HTTP request flow

**Integration scenarios:**
- Complete request/response cycle
- Static file serving
- API functionality
- WebSocket communication

This dashboard implementation provides a professional monitoring interface while maintaining security and performance through hyper.rs's low-level HTTP control.

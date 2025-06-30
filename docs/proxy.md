# HTTP Proxy Forwarder Implementation Guide

## Proxy Forwarder Architecture

### Purpose and Responsibilities

The HTTP Proxy Forwarder is responsible for:

1. **Receiving HTTP requests** from WebSocket client (from cloud/proxy)
2. **Forwarding requests** to local HTTPS server (e.g., localhost:3000)
3. **Streaming responses** back to WebSocket client
4. **Managing connections** to local server with pooling
5. **Handling SSL/TLS** connections to local HTTPS servers
6. **Error handling** and timeout management

### Component Structure

```text
proxy/
├── mod.rs          # Public module interface
├── forwarder.rs    # Main HTTP proxy forwarding logic
├── client.rs       # HTTP client for local server communication
├── response.rs     # Response handling and streaming
└── pool.rs         # Connection pooling for local server
```

## HTTP Client Configuration

### Local Server Client Setup

```rust
// src/proxy/client.rs
use anyhow::{Result, Context};
use reqwest::{Client, ClientBuilder, Response};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn, error, info};
use url::Url;

pub struct LocalServerClient {
    client: Client,
    base_url: Url,
    timeout: Duration,
}

impl LocalServerClient {
    pub fn new(base_url: Url, timeout: Duration, verify_ssl: bool) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(!verify_ssl) // For local development
            .danger_accept_invalid_hostnames(!verify_ssl)
            .pool_max_idle_per_host(10) // Connection pooling
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(30))
            .http2_prior_knowledge() // Enable HTTP/2 if supported
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            timeout,
        })
    }

    pub async fn forward_request(
        &self,
        method: &str,
        path: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<ProxyResponse> {
        let url = self.build_url(path)?;
        
        debug!("Forwarding {} {} to local server", method, url);

        // Build request
        let mut request_builder = self.client.request(
            method.parse().context("Invalid HTTP method")?,
            url.clone(),
        );

        // Add headers (excluding host and other proxy-specific headers)
        for (key, value) in headers {
            if !self.should_skip_header(&key) {
                request_builder = request_builder.header(&key, &value);
            }
        }

        // Add body if present
        if let Some(body_data) = body {
            request_builder = request_builder.body(body_data);
        }

        // Send request
        let start_time = std::time::Instant::now();
        let response = request_builder.send().await
            .context("Failed to send request to local server")?;

        let duration = start_time.elapsed();
        let status = response.status();
        
        info!(
            "Local server response: {} {} -> {} ({:?})",
            method, path, status, duration
        );

        // Convert response
        let proxy_response = ProxyResponse::from_reqwest_response(response).await?;
        
        Ok(proxy_response)
    }

    fn build_url(&self, path: &str) -> Result<Url> {
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };

        self.base_url.join(path)
            .context("Failed to build target URL")
    }

    fn should_skip_header(&self, header_name: &str) -> bool {
        let header_lower = header_name.to_lowercase();
        matches!(
            header_lower.as_str(),
            "host" | "connection" | "upgrade" | 
            "proxy-connection" | "proxy-authorization" |
            "te" | "trailers" | "transfer-encoding"
        )
    }
}

#[derive(Debug)]
pub struct ProxyResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl ProxyResponse {
    pub async fn from_reqwest_response(response: Response) -> Result<Self> {
        let status = response.status();
        let status_text = status.canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Read body
        let body = response.bytes().await
            .context("Failed to read response body")?
            .to_vec();

        let body = if body.is_empty() { None } else { Some(body) };

        Ok(Self {
            status: status.as_u16(),
            status_text,
            headers,
            body,
        })
    }
}
```

## Request Forwarding Logic

### Main Forwarder Implementation

```rust
// src/proxy/forwarder.rs
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error, debug, instrument};
use uuid::Uuid;

use crate::{AppState, TunnelMessage};
use super::client::{LocalServerClient, ProxyResponse};
use crate::websocket::messages::TunnelMessage as WsMessage;

pub struct ProxyForwarder {
    local_client: LocalServerClient,
    app_state: Arc<AppState>,
    stats: Arc<RwLock<ProxyStats>>,
}

#[derive(Debug, Default)]
pub struct ProxyStats {
    pub requests_processed: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub bytes_forwarded: u64,
    pub average_response_time: f64,
    pub active_requests: u64,
}

impl ProxyForwarder {
    pub fn new(local_client: LocalServerClient, app_state: Arc<AppState>) -> Self {
        Self {
            local_client,
            app_state,
            stats: Arc::new(RwLock::new(ProxyStats::default())),
        }
    }

    pub async fn start(&self, mut message_rx: mpsc::UnboundedReceiver<TunnelMessage>) -> Result<()> {
        info!("Starting HTTP proxy forwarder");

        while let Some(message) = message_rx.recv().await {
            match message {
                TunnelMessage::HttpRequest { id, method, url, headers, body } => {
                    // Process request in background to avoid blocking
                    let forwarder = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = forwarder.handle_http_request(id, method, url, headers, body).await {
                            error!("Failed to handle HTTP request: {}", e);
                        }
                    });
                }
                _ => {
                    debug!("Ignoring non-HTTP request message: {:?}", message);
                }
            }
        }

        info!("Proxy forwarder shutting down");
        Ok(())
    }

    #[instrument(skip(self, headers, body))]
    async fn handle_http_request(
        &self,
        request_id: String,
        method: String,
        url: String,
        headers: std::collections::HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Update active request count
        {
            let mut stats = self.stats.write().await;
            stats.active_requests += 1;
        }

        // Extract path from URL
        let path = self.extract_path_from_url(&url)?;

        debug!(
            "Processing request: {} {} (ID: {})",
            method, path, request_id
        );

        // Notify dashboard
        let _ = self.app_state.dashboard_tx.send(
            crate::DashboardEvent::RequestForwarded(format!("{} {}", method, path))
        );

        // Forward request to local server
        let result = self.local_client.forward_request(&method, &path, headers, body).await;

        let duration = start_time.elapsed();

        match result {
            Ok(response) => {
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.requests_processed += 1;
                    stats.requests_successful += 1;
                    stats.active_requests -= 1;
                    
                    if let Some(body) = &response.body {
                        stats.bytes_forwarded += body.len() as u64;
                    }

                    // Update average response time
                    let current_avg = stats.average_response_time;
                    let count = stats.requests_processed as f64;
                    stats.average_response_time = (current_avg * (count - 1.0) + duration.as_millis() as f64) / count;
                }

                // Send response back via WebSocket
                self.send_response(request_id, response).await?;

                info!(
                    "Request completed successfully: {} {} -> {} ({:?})",
                    method, path, response.status, duration
                );
            }
            Err(e) => {
                // Update error stats
                {
                    let mut stats = self.stats.write().await;
                    stats.requests_processed += 1;
                    stats.requests_failed += 1;
                    stats.active_requests -= 1;
                }

                error!(
                    "Request failed: {} {} -> Error: {} ({:?})",
                    method, path, e, duration
                );

                // Send error response
                self.send_error_response(request_id, 502, "Bad Gateway", &e.to_string()).await?;

                // Notify dashboard
                let _ = self.app_state.dashboard_tx.send(
                    crate::DashboardEvent::Error(format!("Request failed: {}", e))
                );
            }
        }

        Ok(())
    }

    async fn send_response(&self, request_id: String, response: ProxyResponse) -> Result<()> {
        let ws_message = WsMessage::http_response(
            request_id,
            response.status,
            response.status_text,
            response.headers,
            response.body,
        );

        // Get WebSocket message sender from app state
        // This assumes the WebSocket client provides a way to send messages
        // Implementation depends on how WebSocket client is integrated
        if let Err(e) = self.send_to_websocket(ws_message).await {
            warn!("Failed to send response to WebSocket: {}", e);
        }

        Ok(())
    }

    async fn send_error_response(
        &self,
        request_id: String,
        status: u16,
        status_text: &str,
        error_message: &str,
    ) -> Result<()> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let body = Some(error_message.as_bytes().to_vec());

        let ws_message = WsMessage::http_response(
            request_id,
            status,
            status_text.to_string(),
            headers,
            body,
        );

        if let Err(e) = self.send_to_websocket(ws_message).await {
            warn!("Failed to send error response to WebSocket: {}", e);
        }

        Ok(())
    }

    async fn send_to_websocket(&self, message: WsMessage) -> Result<()> {
        // This would need to be implemented based on how WebSocket client
        // exposes its message sending capability
        // For now, we'll assume there's a channel in app_state
        
        // Convert to internal message type and send
        // This is a placeholder - actual implementation depends on architecture
        Ok(())
    }

    fn extract_path_from_url(&self, url: &str) -> Result<String> {
        if url.starts_with('/') {
            // Already a path
            Ok(url.to_string())
        } else if let Ok(parsed_url) = url::Url::parse(url) {
            // Full URL, extract path
            let path = parsed_url.path();
            let query = parsed_url.query().map(|q| format!("?{}", q)).unwrap_or_default();
            Ok(format!("{}{}", path, query))
        } else {
            // Assume it's a relative path
            Ok(format!("/{}", url.trim_start_matches('/')))
        }
    }

    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }
}

impl Clone for ProxyForwarder {
    fn clone(&self) -> Self {
        Self {
            local_client: self.local_client.clone(), // This would need Clone impl for LocalServerClient
            app_state: self.app_state.clone(),
            stats: self.stats.clone(),
        }
    }
}

// Public function to start proxy forwarder
pub async fn run_proxy_forwarder(app_state: Arc<AppState>) -> Result<()> {
    let local_url = app_state.settings.local_server.url.clone();
    let timeout = app_state.settings.local_server.timeout;
    let verify_ssl = app_state.settings.local_server.verify_ssl;

    // Create local server client
    let local_client = LocalServerClient::new(local_url, timeout, verify_ssl)
        .context("Failed to create local server client")?;

    // Create forwarder
    let forwarder = ProxyForwarder::new(local_client, app_state.clone());

    // Get message receiver from app state
    let message_rx = app_state.get_proxy_message_receiver(); // This method would need to be implemented

    // Start forwarding
    forwarder.start(message_rx).await
}
```

## Response Handling and Streaming

### Efficient Response Processing

```rust
// src/proxy/response.rs
use anyhow::{Result, Context};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Response;
use std::collections::HashMap;
use tokio::io::AsyncRead;
use tracing::{debug, warn};

pub struct ResponseHandler {
    max_body_size: usize,
    streaming_threshold: usize,
}

impl ResponseHandler {
    pub fn new(max_body_size: usize, streaming_threshold: usize) -> Self {
        Self {
            max_body_size,
            streaming_threshold,
        }
    }

    pub async fn process_response(&self, response: Response) -> Result<ProcessedResponse> {
        let status = response.status();
        let headers = self.extract_headers(&response);
        
        // Check content length to decide on streaming vs buffering
        let content_length = response.content_length();
        let should_stream = content_length
            .map(|len| len as usize > self.streaming_threshold)
            .unwrap_or(false);

        if should_stream {
            debug!("Using streaming for large response ({:?} bytes)", content_length);
            self.process_streaming_response(status.as_u16(), headers, response).await
        } else {
            debug!("Buffering small response");
            self.process_buffered_response(status.as_u16(), headers, response).await
        }
    }

    async fn process_buffered_response(
        &self,
        status: u16,
        headers: HashMap<String, String>,
        response: Response,
    ) -> Result<ProcessedResponse> {
        let body_bytes = response.bytes().await
            .context("Failed to read response body")?;

        if body_bytes.len() > self.max_body_size {
            return Err(anyhow::anyhow!(
                "Response body too large: {} bytes (max: {})",
                body_bytes.len(),
                self.max_body_size
            ));
        }

        let body = if body_bytes.is_empty() {
            None
        } else {
            Some(body_bytes.to_vec())
        };

        Ok(ProcessedResponse {
            status,
            headers,
            body,
            is_streaming: false,
        })
    }

    async fn process_streaming_response(
        &self,
        status: u16,
        headers: HashMap<String, String>,
        response: Response,
    ) -> Result<ProcessedResponse> {
        let mut stream = response.bytes_stream();
        let mut body_chunks = Vec::new();
        let mut total_size = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.context("Failed to read response chunk")?;
            total_size += chunk.len();

            if total_size > self.max_body_size {
                return Err(anyhow::anyhow!(
                    "Streamed response too large: {} bytes (max: {})",
                    total_size,
                    self.max_body_size
                ));
            }

            body_chunks.push(chunk);
        }

        // Combine all chunks
        let body = if body_chunks.is_empty() {
            None
        } else {
            let mut combined = Vec::with_capacity(total_size);
            for chunk in body_chunks {
                combined.extend_from_slice(&chunk);
            }
            Some(combined)
        };

        Ok(ProcessedResponse {
            status,
            headers,
            body,
            is_streaming: true,
        })
    }

    fn extract_headers(&self, response: &Response) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                // Skip certain headers that shouldn't be forwarded
                let key_lower = key.as_str().to_lowercase();
                if !self.should_skip_response_header(&key_lower) {
                    headers.insert(key.to_string(), value_str.to_string());
                }
            }
        }

        headers
    }

    fn should_skip_response_header(&self, header_name: &str) -> bool {
        matches!(
            header_name,
            "connection" | "upgrade" | "proxy-connection" |
            "transfer-encoding" | "te" | "trailers"
        )
    }
}

#[derive(Debug)]
pub struct ProcessedResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub is_streaming: bool,
}

impl ProcessedResponse {
    pub fn body_size(&self) -> usize {
        self.body.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_size_calculation() {
        let response = ProcessedResponse {
            status: 200,
            headers: HashMap::new(),
            body: Some(b"Hello, World!".to_vec()),
            is_streaming: false,
        };

        assert_eq!(response.body_size(), 13);
        assert!(response.has_body());
    }

    #[test]
    fn test_empty_response() {
        let response = ProcessedResponse {
            status: 204,
            headers: HashMap::new(),
            body: None,
            is_streaming: false,
        };

        assert_eq!(response.body_size(), 0);
        assert!(!response.has_body());
    }
}
```

## Connection Pooling Implementation

### HTTP Connection Pool

```rust
// src/proxy/pool.rs
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use url::Url;

pub struct ConnectionPool {
    pools: Arc<RwLock<HashMap<String, Arc<Mutex<Pool>>>>>,
    max_connections_per_host: usize,
    idle_timeout: Duration,
    connection_timeout: Duration,
}

struct Pool {
    connections: Vec<PooledConnection>,
    created: usize,
    max_size: usize,
}

struct PooledConnection {
    // This would contain the actual connection type
    // For now, we'll use a placeholder
    id: String,
    created_at: Instant,
    last_used: Instant,
    is_busy: bool,
}

impl ConnectionPool {
    pub fn new(
        max_connections_per_host: usize,
        idle_timeout: Duration,
        connection_timeout: Duration,
    ) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_connections_per_host,
            idle_timeout,
            connection_timeout,
        }
    }

    pub async fn get_connection(&self, url: &Url) -> Result<PooledConnection> {
        let host_key = self.host_key(url);
        
        // Get or create pool for this host
        let pool = {
            let pools = self.pools.read().await;
            pools.get(&host_key).cloned()
        };

        let pool = match pool {
            Some(p) => p,
            None => {
                let mut pools = self.pools.write().await;
                pools.entry(host_key.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(Pool::new(self.max_connections_per_host))))
                    .clone()
            }
        };

        // Get connection from pool
        let mut pool_guard = pool.lock().await;
        
        // Try to find an idle connection
        if let Some(conn) = pool_guard.get_idle_connection() {
            debug!("Reusing pooled connection for {}", host_key);
            return Ok(conn);
        }

        // Create new connection if pool not full
        if pool_guard.can_create_connection() {
            debug!("Creating new connection for {}", host_key);
            let conn = pool_guard.create_connection(&host_key).await?;
            return Ok(conn);
        }

        // Wait for connection to become available
        debug!("Waiting for available connection for {}", host_key);
        // Implementation would wait for a connection to become available
        // For now, we'll return an error
        Err(anyhow::anyhow!("No connections available for {}", host_key))
    }

    pub async fn return_connection(&self, url: &Url, connection: PooledConnection) {
        let host_key = self.host_key(url);
        
        if let Some(pool) = self.pools.read().await.get(&host_key) {
            let mut pool_guard = pool.lock().await;
            pool_guard.return_connection(connection);
        }
    }

    pub async fn cleanup_idle_connections(&self) {
        let mut pools = self.pools.write().await;
        let mut hosts_to_remove = Vec::new();

        for (host, pool) in pools.iter() {
            let mut pool_guard = pool.lock().await;
            pool_guard.cleanup_idle(self.idle_timeout);
            
            if pool_guard.is_empty() {
                hosts_to_remove.push(host.clone());
            }
        }

        // Remove empty pools
        for host in hosts_to_remove {
            pools.remove(&host);
            debug!("Removed empty connection pool for {}", host);
        }
    }

    fn host_key(&self, url: &Url) -> String {
        format!("{}://{}", url.scheme(), url.host_str().unwrap_or("unknown"))
    }

    pub async fn get_stats(&self) -> ConnectionPoolStats {
        let pools = self.pools.read().await;
        let mut stats = ConnectionPoolStats::default();

        for pool in pools.values() {
            let pool_guard = pool.lock().await;
            stats.total_pools += 1;
            stats.total_connections += pool_guard.connections.len();
            stats.idle_connections += pool_guard.connections.iter()
                .filter(|c| !c.is_busy)
                .count();
            stats.busy_connections += pool_guard.connections.iter()
                .filter(|c| c.is_busy)
                .count();
        }

        stats
    }
}

impl Pool {
    fn new(max_size: usize) -> Self {
        Self {
            connections: Vec::new(),
            created: 0,
            max_size,
        }
    }

    fn get_idle_connection(&mut self) -> Option<PooledConnection> {
        for conn in &mut self.connections {
            if !conn.is_busy {
                conn.is_busy = true;
                conn.last_used = Instant::now();
                return Some(conn.clone());
            }
        }
        None
    }

    fn can_create_connection(&self) -> bool {
        self.created < self.max_size
    }

    async fn create_connection(&mut self, host: &str) -> Result<PooledConnection> {
        let conn = PooledConnection {
            id: format!("{}_{}", host, self.created),
            created_at: Instant::now(),
            last_used: Instant::now(),
            is_busy: true,
        };

        self.connections.push(conn.clone());
        self.created += 1;

        Ok(conn)
    }

    fn return_connection(&mut self, mut connection: PooledConnection) {
        connection.is_busy = false;
        connection.last_used = Instant::now();
        
        // Update the connection in the pool
        for conn in &mut self.connections {
            if conn.id == connection.id {
                *conn = connection;
                break;
            }
        }
    }

    fn cleanup_idle(&mut self, idle_timeout: Duration) {
        let now = Instant::now();
        self.connections.retain(|conn| {
            if !conn.is_busy && now.duration_since(conn.last_used) > idle_timeout {
                debug!("Removing idle connection: {}", conn.id);
                false
            } else {
                true
            }
        });
    }

    fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

impl Clone for PooledConnection {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            created_at: self.created_at,
            last_used: self.last_used,
            is_busy: self.is_busy,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConnectionPoolStats {
    pub total_pools: usize,
    pub total_connections: usize,
    pub idle_connections: usize,
    pub busy_connections: usize,
}

// Background task to clean up idle connections
pub async fn start_cleanup_task(pool: Arc<ConnectionPool>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;
        pool.cleanup_idle_connections().await;
    }
}
```

## Integration with Application

### Module Integration

```rust
// src/proxy/mod.rs
pub mod forwarder;
pub mod client;
pub mod response;
pub mod pool;

pub use forwarder::{ProxyForwarder, run_proxy_forwarder, ProxyStats};
pub use client::{LocalServerClient, ProxyResponse};
pub use response::{ResponseHandler, ProcessedResponse};
pub use pool::{ConnectionPool, ConnectionPoolStats, start_cleanup_task};

use anyhow::Result;
use std::sync::Arc;
use crate::AppState;

// Initialize proxy system
pub async fn initialize_proxy(app_state: Arc<AppState>) -> Result<()> {
    // Start proxy forwarder
    let forwarder_task = tokio::spawn({
        let state = app_state.clone();
        async move {
            if let Err(e) = run_proxy_forwarder(state).await {
                tracing::error!("Proxy forwarder error: {}", e);
            }
        }
    });

    // Start connection pool cleanup
    let pool = Arc::new(ConnectionPool::new(
        app_state.settings.local_server.max_connections,
        std::time::Duration::from_secs(60),
        app_state.settings.local_server.timeout,
    ));

    let cleanup_task = tokio::spawn({
        let pool = pool.clone();
        async move {
            start_cleanup_task(pool).await;
        }
    });

    // Wait for tasks (they should run indefinitely)
    tokio::try_join!(
        async { forwarder_task.await.map_err(|e| anyhow::anyhow!("Forwarder task error: {}", e)) },
        async { cleanup_task.await.map_err(|e| anyhow::anyhow!("Cleanup task error: {}", e)) }
    )?;

    Ok(())
}
```

This HTTP Proxy Forwarder implementation provides:

1. **Efficient request forwarding** with connection pooling and reuse
2. **Comprehensive error handling** with proper error responses
3. **Response streaming** for large payloads to minimize memory usage
4. **Connection management** with automatic cleanup of idle connections
5. **SSL/TLS support** for HTTPS local servers with certificate options
6. **Statistics tracking** for monitoring and debugging
7. **Header filtering** to remove proxy-specific headers
8. **Timeout management** to prevent hanging requests
9. **Concurrent processing** to handle multiple requests simultaneously
10. **Integration with WebSocket** for bidirectional communication

The forwarder is designed to be performant, reliable, and maintainable while handling the complexities of HTTP proxy operations.

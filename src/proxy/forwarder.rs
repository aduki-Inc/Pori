use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, instrument, warn};

use super::client::{LocalServerClient, LocalServerResponse};
use crate::protocol::http::HttpMessage;
use crate::protocol::messages::{HttpPayload, MessagePayload};
use crate::protocol::tunnel::TunnelMessage;
use crate::{local_log, utils::http::get_status_description, AppState, DashboardEvent};

/// HTTP proxy forwarder that forwards requests to a local server
pub struct ProxyForwarder {
    local_client: LocalServerClient,
    app_state: Arc<AppState>,
    stats: Arc<RwLock<ProxyStats>>,
}

/// Proxy statistics
#[derive(Debug, Default, Clone)]
pub struct ProxyStats {
    pub requests_processed: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub bytes_forwarded: u64,
    pub average_response_time_ms: f64,
    pub active_requests: u64,
}

impl ProxyForwarder {
    /// Create a new proxy forwarder
    pub fn new(app_state: Arc<AppState>) -> Result<Self> {
        let local_client = LocalServerClient::new(
            app_state.settings.local_server.url.clone(),
            app_state.settings.local_server.timeout,
            app_state.settings.local_server.verify_ssl,
            &app_state.settings.local_server.http_version,
        )?;

        Ok(Self {
            local_client,
            app_state,
            stats: Arc::new(RwLock::new(ProxyStats::default())),
        })
    }

    /// Main forwarder run loop
    #[instrument(skip(self, message_rx))]
    pub async fn run(&self, mut message_rx: mpsc::UnboundedReceiver<HttpMessage>) -> Result<()> {
        local_log!("HTTP proxy forwarder started");

        while let Some(message) = message_rx.recv().await {
            // Extract HTTP request information from the message
            if let Some((method, url, headers, cloud_request_id)) =
                message.extract_request_info_with_id()
            {
                let body = match &message.message.payload {
                    MessagePayload::Http(HttpPayload::Request { body, .. }) => body.clone(),
                    _ => None,
                };

                // Process request in the background to avoid blocking
                let forwarder = self.clone();
                let request_id = message.request_id().to_string();
                tokio::spawn(async move {
                    if let Err(e) = forwarder
                        .handle_http_request(
                            request_id,
                            method,
                            url,
                            headers,
                            body,
                            cloud_request_id,
                        )
                        .await
                    {
                        error!("Failed to handle HTTP request: {}", e);
                    }
                });
            } else {
                debug!("Ignoring non-HTTP request message");
            }
        }

        local_log!("Proxy forwarder shutting down");
        Ok(())
    }

    /// Handle individual HTTP request
    #[instrument(skip(self, headers, body))]
    async fn handle_http_request(
        &self,
        request_id: String,
        method: String,
        url: String,
        headers: std::collections::HashMap<String, String>,
        body: Option<Vec<u8>>,
        cloud_request_id: String,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Update active request count
        {
            let mut stats = self.stats.write().await;
            stats.active_requests += 1;
        }

        // Extract path from URL
        let path = self.extract_path_from_url(&url)?;

        local_log!(
            "Forwarding request to local server: {} {} (ID: {}, Cloud RequestID: {})",
            method,
            path,
            request_id,
            cloud_request_id
        );

        debug!(
            "Processing request: {} {} (ID: {}, Cloud RequestID: {})",
            method, path, request_id, cloud_request_id
        );

        // Add X-Request-ID header with the cloud request ID for tracking (if not already present)
        let mut headers_with_request_id = headers.clone();

        // Only add X-Request-ID if it doesn't already exist
        if !headers_with_request_id.contains_key("x-request-id")
            && !headers_with_request_id.contains_key("X-Request-ID")
        {
            headers_with_request_id.insert("X-Request-ID".to_string(), cloud_request_id.clone());
        }

        // Only add X-Forwarded-By if it doesn't already exist
        if !headers_with_request_id.contains_key("x-forwarded-by")
            && !headers_with_request_id.contains_key("X-Forwarded-By")
        {
            headers_with_request_id.insert("X-Forwarded-By".to_string(), "pori-proxy".to_string());
        }

        // Notify dashboard
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::RequestForwarded(format!("{method} {path}")));

        // Forward request to local server with timeout handling
        let result = tokio::time::timeout(
            self.app_state.settings.local_server.timeout,
            self.local_client
                .forward_request(&method, &path, headers_with_request_id, body),
        )
        .await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(response)) => {
                // Successfully received response from a local server
                self.handle_successful_response(
                    request_id,
                    method,
                    path,
                    response,
                    duration,
                    cloud_request_id,
                )
                .await?;
            }
            Ok(Err(e)) => {
                // Check if it's a connection error or server error
                let error_string = e.to_string().to_lowercase();
                if error_string.contains("connection")
                    || error_string.contains("refused")
                    || error_string.contains("unreachable")
                    || error_string.contains("network")
                {
                    // Connection/network error - local server is unreachable
                    self.handle_connection_error(
                        request_id,
                        method,
                        path,
                        e,
                        duration,
                        cloud_request_id,
                    )
                    .await?;
                } else {
                    // Other server error (e.g., HTTP error responses)
                    self.handle_server_error(
                        request_id,
                        method,
                        path,
                        e,
                        duration,
                        cloud_request_id,
                    )
                    .await?;
                }
            }
            Err(_) => {
                // Request timed out
                self.handle_timeout_error(request_id, method, path, duration, cloud_request_id)
                    .await?;
            }
        }

        Ok(())
    }

    /// Handle successful response from local server
    async fn handle_successful_response(
        &self,
        request_id: String,
        method: String,
        path: String,
        response: LocalServerResponse,
        duration: std::time::Duration,
        cloud_request_id: String,
    ) -> Result<()> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.requests_processed += 1;
            stats.requests_successful += 1;
            stats.active_requests -= 1;

            if let Some(ref body) = response.body {
                stats.bytes_forwarded += body.len() as u64;
            }

            // Update average response time
            let duration_ms = duration.as_millis() as f64;
            let count = stats.requests_processed as f64;
            let current_avg = stats.average_response_time_ms;
            stats.average_response_time_ms = (current_avg * (count - 1.0) + duration_ms) / count;
        }

        // Store status before sending
        let status = response.status;
        let body_size = response.body.as_ref().map(|b| b.len()).unwrap_or(0);
        let status_description = get_status_description(status);

        local_log!(
            "Request completed successfully: {} {} -> {} {} bytes ({:?})",
            method,
            path,
            status_description,
            body_size,
            duration
        );

        // Send response back via WebSocket
        self.send_response(request_id, response, cloud_request_id)
            .await?;

        // Notify dashboard of successful response
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::ResponseReceived(status, body_size));

        Ok(())
    }

    /// Handle error from a local server
    async fn handle_server_error(
        &self,
        request_id: String,
        method: String,
        path: String,
        error: anyhow::Error,
        duration: std::time::Duration,
        cloud_request_id: String,
    ) -> Result<()> {
        // Update error stats
        {
            let mut stats = self.stats.write().await;
            stats.requests_processed += 1;
            stats.requests_failed += 1;
            stats.active_requests -= 1;
        }

        error!(
            "Local server error: {} {} -> Error: {} ({:?})",
            method, path, error, duration
        );

        local_log!(
            "Sending error response: {} {} -> 502 Bad Gateway (ID: {})",
            method,
            path,
            request_id
        );

        // Send error response
        self.send_error_response(
            request_id,
            502,
            "Bad Gateway",
            &format!("Local server error: {}", error),
            cloud_request_id,
        )
        .await?;

        // Notify dashboard
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::Error(format!(
                "Local server error: {}",
                error
            )));

        Ok(())
    }

    /// Handle timeout when the local server doesn't respond
    async fn handle_timeout_error(
        &self,
        request_id: String,
        method: String,
        path: String,
        duration: std::time::Duration,
        cloud_request_id: String,
    ) -> Result<()> {
        // Update error stats
        {
            let mut stats = self.stats.write().await;
            stats.requests_processed += 1;
            stats.requests_failed += 1;
            stats.active_requests -= 1;
        }

        error!(
            "Request timeout: {} {} -> Timeout after {:?}",
            method, path, duration
        );

        local_log!(
            "Sending timeout response: {} {} -> 504 Gateway Timeout (ID: {})",
            method,
            path,
            request_id
        );

        // Send timeout error response
        self.send_error_response(
            request_id,
            504,
            "Gateway Timeout",
            &format!(
                "Local server did not respond within {:?}",
                self.app_state.settings.local_server.timeout
            ),
            cloud_request_id,
        )
        .await?;

        // Notify dashboard
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::Error(format!(
                "Request timeout: {} {}",
                method, path
            )));

        Ok(())
    }

    /// Send a successful response back via WebSocket
    async fn send_response(
        &self,
        request_id: String,
        response: LocalServerResponse,
        cloud_request_id: String,
    ) -> Result<()> {
        let body_size = response.body.as_ref().map(|b| b.len()).unwrap_or(0);
        let status_description = get_status_description(response.status);

        crate::proxy_log!(
            "RESPONSE [{}] {} {} bytes - sending to proxy server",
            request_id,
            status_description,
            body_size
        );

        // Log response headers for debugging
        debug!("Response headers: {:?}", response.headers);

        let tunnel_message = TunnelMessage::http_response_with_id(
            "default-tunnel".to_string(),
            "default-client".to_string(),
            response.status,
            response.status_text,
            response.headers,
            response.body,
            cloud_request_id,
        );

        // Send via WebSocket to a proxy server
        if let Err(e) = self.app_state.websocket_tx.send(tunnel_message) {
            warn!("Failed to send a response to WebSocket: {}", e);
            return Err(anyhow::anyhow!(
                "Failed to send a response to the proxy server: {}",
                e
            ));
        }

        crate::proxy_log!(
            "Response successfully sent to proxy server [{}]",
            request_id
        );
        Ok(())
    }

    /// Send error response back via WebSocket
    async fn send_error_response(
        &self,
        request_id: String,
        status: u16,
        status_text: &str,
        error_message: &str,
        cloud_request_id: String,
    ) -> Result<()> {
        crate::proxy_log!(
            "Sending error response to proxy server: {} {} - {} (ID: {})",
            status,
            status_text,
            error_message,
            request_id
        );

        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        headers.insert("cache-control".to_string(), "no-cache".to_string());

        // Create HTML error page
        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #d32f2f; }}
        .error-code {{ font-size: 2em; font-weight: bold; }}
        .error-message {{ margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>Error {}</h1>
    <div class="error-code">{} {}</div>
    <div class="error-message">{}</div>
</body>
</html>"#,
            status, status_text, status, status, status_text, error_message
        );

        let body = Some(html_body.as_bytes().to_vec());

        let tunnel_message = TunnelMessage::http_response_with_id(
            "default-tunnel".to_string(),
            "default-client".to_string(),
            status,
            status_text.to_string(),
            headers,
            body,
            cloud_request_id,
        );

        if let Err(e) = self.app_state.websocket_tx.send(tunnel_message) {
            warn!("Failed to send an error response to WebSocket: {}", e);
            return Err(anyhow::anyhow!(
                "Failed to send an error response to the proxy server: {}",
                e
            ));
        }

        crate::proxy_log!(
            "Error response successfully sent to the proxy server (ID: {})",
            request_id
        );
        Ok(())
    }

    /// Handle case where the local server is unreachable
    async fn handle_connection_error(
        &self,
        request_id: String,
        method: String,
        path: String,
        error: anyhow::Error,
        duration: std::time::Duration,
        cloud_request_id: String,
    ) -> Result<()> {
        // Update error stats
        {
            let mut stats = self.stats.write().await;
            stats.requests_processed += 1;
            stats.requests_failed += 1;
            stats.active_requests -= 1;
        }

        error!(
            "Local server connection failed: {} {} -> Error: {} ({:?})",
            method, path, error, duration
        );

        local_log!(
            "Sending connection error response: {} {} -> 503 Service Unavailable (ID: {})",
            method,
            path,
            request_id
        );

        // Send service unavailable response
        self.send_error_response(
            request_id,
            503,
            "Service Unavailable",
            &format!("Local server is unreachable: {}", error),
            cloud_request_id,
        )
        .await?;

        // Notify dashboard
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::Error(format!(
                "Local server unreachable: {}",
                error
            )));

        Ok(())
    }

    /// Extract path from URL
    fn extract_path_from_url(&self, url: &str) -> Result<String> {
        if url.starts_with('/') {
            // Already a path
            Ok(url.to_string())
        } else if let Ok(parsed_url) = url::Url::parse(url) {
            // Full URL, extract path and query
            let path = parsed_url.path();
            let query = parsed_url
                .query()
                .map(|q| format!("?{q}"))
                .unwrap_or_default();
            Ok(format!("{path}{query}"))
        } else {
            // Assume it's a relative path
            Ok(format!("/{}", url.trim_start_matches('/')))
        }
    }

    /// Get proxy statistics
    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }

    /// Get detailed proxy statistics with additional metrics
    pub async fn get_detailed_stats(&self) -> DetailedProxyStats {
        let stats = self.stats.read().await;
        let success_rate = if stats.requests_processed > 0 {
            (stats.requests_successful as f64 / stats.requests_processed as f64) * 100.0
        } else {
            0.0
        };

        DetailedProxyStats {
            basic_stats: stats.clone(),
            success_rate_percentage: success_rate,
            local_server_url: self.app_state.settings.local_server.url.to_string(),
            timeout_duration: self.app_state.settings.local_server.timeout,
        }
    }
}

/// Detailed proxy statistics
#[derive(Debug, Clone)]
pub struct DetailedProxyStats {
    pub basic_stats: ProxyStats,
    pub success_rate_percentage: f64,
    pub local_server_url: String,
    pub timeout_duration: std::time::Duration,
}

impl Clone for ProxyForwarder {
    fn clone(&self) -> Self {
        Self {
            local_client: self.local_client.clone(),
            app_state: self.app_state.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{cli::CliArgs, settings::AppSettings};

    fn create_test_app_state() -> Arc<AppState> {
        let args = CliArgs {
            url: Some("ws://localhost:7616".parse().unwrap()),
            token: Some("test-token".to_string()),
            yml: None,
            protocol: "https".to_string(),
            port: 3000,
            dashboard_port: 7616,
            log_level: "info".to_string(),
            config: None,
            no_dashboard: false,
            timeout: 30,
            max_reconnects: 0,
            verify_ssl: false,
            max_connections: 10,
            http_version: "auto".to_string(),
        };

        let settings = AppSettings::from_cli(args).unwrap();
        let (app_state, _) = AppState::new(settings);
        Arc::new(app_state)
    }

    #[test]
    fn test_url_path_extraction() {
        let app_state = create_test_app_state();
        let forwarder = ProxyForwarder::new(app_state).unwrap();

        assert_eq!(
            forwarder.extract_path_from_url("/api/test").unwrap(),
            "/api/test"
        );
        assert_eq!(
            forwarder.extract_path_from_url("api/test").unwrap(),
            "/api/test"
        );

        let full_url_result =
            forwarder.extract_path_from_url("https://example.com/api/test?param=value");
        assert!(full_url_result.is_ok());
        assert_eq!(full_url_result.unwrap(), "/api/test?param=value");
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let app_state = create_test_app_state();
        let forwarder = ProxyForwarder::new(app_state).unwrap();

        let stats = forwarder.get_stats().await;
        assert_eq!(stats.requests_processed, 0);
        assert_eq!(stats.requests_successful, 0);
        assert_eq!(stats.requests_failed, 0);
    }
}

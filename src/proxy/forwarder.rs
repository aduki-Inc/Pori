use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, instrument, warn};

use super::{
    client::{LocalServerClient, LocalServerResponse},
    messages::ProxyMessage,
};
use crate::{AppState, DashboardEvent};

/// HTTP proxy forwarder that forwards requests to local server
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
    /// Create new proxy forwarder
    pub fn new(app_state: Arc<AppState>) -> Result<Self> {
        let local_client = LocalServerClient::new(
            app_state.settings.local_server.url.clone(),
            app_state.settings.local_server.timeout,
            app_state.settings.local_server.verify_ssl,
        )?;

        Ok(Self {
            local_client,
            app_state,
            stats: Arc::new(RwLock::new(ProxyStats::default())),
        })
    }

    /// Main forwarder run loop
    #[instrument(skip(self, message_rx))]
    pub async fn run(&self, mut message_rx: mpsc::UnboundedReceiver<ProxyMessage>) -> Result<()> {
        info!("HTTP proxy forwarder started");

        while let Some(message) = message_rx.recv().await {
            match message {
                ProxyMessage::HttpRequest {
                    id,
                    method,
                    url,
                    headers,
                    body,
                } => {
                    // Process request in background to avoid blocking
                    let forwarder = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = forwarder
                            .handle_http_request(id, method, url, headers, body)
                            .await
                        {
                            error!("Failed to handle HTTP request: {}", e);
                        }
                    });
                }

                ProxyMessage::Stats { .. } => {
                    // Handle stats request
                    debug!("Received stats request");
                }

                _ => {
                    debug!(
                        "Ignoring non-HTTP request message: {:?}",
                        message.message_type()
                    );
                }
            }
        }

        info!("Proxy forwarder shutting down");
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
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::RequestForwarded(format!("{method} {path}")));

        // Forward request to local server
        let result = self
            .local_client
            .forward_request(&method, &path, headers, body)
            .await;

        let duration = start_time.elapsed();

        match result {
            Ok(response) => {
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
                    stats.average_response_time_ms =
                        (current_avg * (count - 1.0) + duration_ms) / count;
                }

                // Store status before sending
                let status = response.status;

                // Send response back via WebSocket
                self.send_response(request_id, response).await?;

                info!(
                    "Request completed successfully: {} {} -> {} ({:?})",
                    method, path, status, duration
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
                self.send_error_response(request_id, 502, "Bad Gateway", &e.to_string())
                    .await?;

                // Notify dashboard
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::Error(format!("Request failed: {e}")));
            }
        }

        Ok(())
    }

    /// Send successful response back via WebSocket
    async fn send_response(&self, request_id: String, response: LocalServerResponse) -> Result<()> {
        let tunnel_message = crate::websocket::messages::TunnelMessage::http_response(
            request_id,
            response.status,
            response.status_text,
            response.headers,
            response.body,
        );

        // Send via WebSocket
        if let Err(e) = self.app_state.websocket_tx.send(tunnel_message) {
            warn!("Failed to send response to WebSocket: {}", e);
        }

        Ok(())
    }

    /// Send error response back via WebSocket
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

        let tunnel_message = crate::websocket::messages::TunnelMessage::http_response(
            request_id,
            status,
            status_text.to_string(),
            headers,
            body,
        );

        if let Err(e) = self.app_state.websocket_tx.send(tunnel_message) {
            warn!("Failed to send error response to WebSocket: {}", e);
        }

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

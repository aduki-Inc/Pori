use super::messages::TunnelMessage;
use crate::{AppState, ConnectionStatus, DashboardEvent};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// Handle HTTP tunnel messages
pub struct TunnelHandler {
    app_state: Arc<AppState>,
}

impl TunnelHandler {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Process incoming tunnel message from WebSocket
    #[instrument(skip(self, message))]
    pub async fn handle_message(&self, message: TunnelMessage) -> Result<Option<TunnelMessage>> {
        match message {
            TunnelMessage::Auth { .. } => {
                // Client should not receive auth messages
                warn!("Received unexpected auth message");
                Ok(None)
            }

            TunnelMessage::AuthSuccess { session_id } => {
                info!("Authentication successful, session ID: {}", session_id);

                // Update connection status
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::ConnectionStatus(
                        ConnectionStatus::Connected,
                    ));

                // Update stats
                self.app_state
                    .update_stats(|stats| {
                        stats.connection_status = "connected".to_string();
                    })
                    .await;

                Ok(None)
            }

            TunnelMessage::AuthError { error } => {
                error!("Authentication failed: {}", error);

                // Update connection status
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::ConnectionStatus(ConnectionStatus::Error(
                        error.clone(),
                    )));

                // Return error for client to handle
                Err(anyhow::anyhow!("Authentication failed: {}", error))
            }

            TunnelMessage::HttpRequest {
                id,
                method,
                url,
                headers,
                body,
            } => {
                debug!("Received HTTP request: {} {} (ID: {})", method, url, id);

                // Forward to proxy component
                let proxy_message = crate::proxy::messages::ProxyMessage::HttpRequest {
                    id: id.clone(),
                    method: method.clone(),
                    url: url.clone(),
                    headers: headers.clone(),
                    body: body.clone(),
                };

                if let Err(e) = self.app_state.proxy_tx.send(proxy_message) {
                    error!("Failed to forward HTTP request to proxy: {}", e);

                    // Send error response
                    return Ok(Some(TunnelMessage::error_for_request(
                        id,
                        "Internal proxy error".to_string(),
                        Some(500),
                    )));
                }

                // Notify dashboard
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::RequestForwarded(format!("{method} {url}")));

                // Update stats
                self.app_state
                    .update_stats(|stats| {
                        stats.requests_processed += 1;
                    })
                    .await;

                Ok(None)
            }

            TunnelMessage::HttpResponse { .. } => {
                // Client should not receive HTTP responses
                warn!("Received unexpected HTTP response message");
                Ok(None)
            }

            TunnelMessage::Error {
                request_id,
                error,
                code,
            } => {
                if let Some(req_id) = request_id {
                    error!("Request {} failed: {} (code: {:?})", req_id, error, code);
                } else {
                    error!("General error: {} (code: {:?})", error, code);
                }

                // Notify dashboard
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::Error(error.clone()));

                // Update error stats
                self.app_state
                    .update_stats(|stats| {
                        stats.requests_failed += 1;
                    })
                    .await;

                Ok(None)
            }

            TunnelMessage::Ping { timestamp } => {
                debug!("Received ping with timestamp: {}", timestamp);

                // Respond with pong
                Ok(Some(TunnelMessage::pong(timestamp)))
            }

            TunnelMessage::Pong { timestamp } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let latency = now.saturating_sub(timestamp);
                debug!("Received pong, latency: {}s", latency);

                Ok(None)
            }

            TunnelMessage::Stats { .. } => {
                // Client should not receive stats messages
                warn!("Received unexpected stats message");
                Ok(None)
            }

            TunnelMessage::Status { status, message } => {
                info!("Server status: {} - {:?}", status, message);

                // Update connection status
                let _ = self
                    .app_state
                    .dashboard_tx
                    .send(DashboardEvent::ConnectionStatus(
                        ConnectionStatus::Connected,
                    ));

                Ok(None)
            }
        }
    }

    /// Create authentication message for initial connection
    pub fn create_auth_message(&self) -> TunnelMessage {
        TunnelMessage::auth(self.app_state.settings.websocket.token.clone())
    }

    /// Create periodic ping message
    pub fn create_ping_message() -> TunnelMessage {
        TunnelMessage::ping()
    }

    /// Create statistics message
    pub async fn create_stats_message(&self) -> TunnelMessage {
        let stats = self.app_state.get_stats().await;
        TunnelMessage::stats(
            stats.requests_processed,
            stats.bytes_forwarded,
            stats.uptime_seconds,
        )
    }

    /// Handle HTTP response from proxy to send back via WebSocket
    pub async fn handle_proxy_response(
        &self,
        request_id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> TunnelMessage {
        debug!("Creating HTTP response for request ID: {}", request_id);

        // Notify dashboard
        let body_size = body.as_ref().map(|b| b.len()).unwrap_or(0);
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::ResponseReceived(status, body_size));

        // Update stats
        self.app_state
            .update_stats(|stats| {
                stats.requests_successful += 1;
                stats.bytes_forwarded += body_size as u64;
            })
            .await;

        TunnelMessage::http_response(request_id, status, status_text, headers, body)
    }

    /// Handle error from proxy to send back via WebSocket
    pub async fn handle_proxy_error(
        &self,
        request_id: String,
        error: String,
        status_code: Option<u16>,
    ) -> TunnelMessage {
        error!("Proxy error for request {}: {}", request_id, error);

        // Notify dashboard
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::Error(format!("Proxy error: {error}")));

        // Update error stats
        self.app_state
            .update_stats(|stats| {
                stats.requests_failed += 1;
            })
            .await;

        TunnelMessage::error_for_request(request_id, error, status_code)
    }

    /// Validate incoming HTTP request
    pub fn validate_http_request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<()> {
        // Validate HTTP method
        if !matches!(
            method.to_uppercase().as_str(),
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" | "TRACE"
        ) {
            anyhow::bail!("Invalid HTTP method: {}", method);
        }

        // Validate URL
        if url.is_empty() {
            anyhow::bail!("URL cannot be empty");
        }

        // Check for required headers if needed
        // This could be extended based on requirements

        // Validate header names and values
        for (name, value) in headers {
            if name.is_empty() {
                anyhow::bail!("Header name cannot be empty");
            }

            // Check for invalid characters in header names
            if !name.chars().all(|c| c.is_ascii() && !c.is_control()) {
                anyhow::bail!("Invalid characters in header name: {}", name);
            }

            // Check header value
            if !value
                .chars()
                .all(|c| c.is_ascii() && c != '\r' && c != '\n')
            {
                anyhow::bail!("Invalid characters in header value for {}: {}", name, value);
            }
        }

        Ok(())
    }

    /// Filter headers for forwarding
    pub fn filter_headers(&self, headers: HashMap<String, String>) -> HashMap<String, String> {
        headers
            .into_iter()
            .filter(|(name, _)| !self.should_skip_header(name))
            .collect()
    }

    /// Check if header should be skipped during forwarding
    fn should_skip_header(&self, header_name: &str) -> bool {
        let header_lower = header_name.to_lowercase();
        matches!(
            header_lower.as_str(),
            "connection"
                | "upgrade"
                | "proxy-connection"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "host"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{cli::CliArgs, settings::AppSettings};

    fn create_test_app_state() -> Arc<AppState> {
        // This would need proper initialization in actual tests
        // For now, we'll create a minimal mock
        let args = CliArgs {
            url: "ws://localhost:7616".parse().unwrap(),
            token: "test-token".to_string(),
            protocol: "http".to_string(),
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

    #[tokio::test]
    async fn test_ping_pong_handling() {
        let app_state = create_test_app_state();
        let handler = TunnelHandler::new(app_state);

        let ping_message = TunnelMessage::ping();
        if let TunnelMessage::Ping { timestamp } = ping_message {
            let response = handler
                .handle_message(TunnelMessage::Ping { timestamp })
                .await
                .unwrap();

            assert!(response.is_some());
            if let Some(TunnelMessage::Pong {
                timestamp: response_timestamp,
            }) = response
            {
                assert_eq!(response_timestamp, timestamp);
            }
        }
    }

    #[test]
    fn test_header_validation() {
        let app_state = create_test_app_state();
        let handler = TunnelHandler::new(app_state);

        let mut valid_headers = HashMap::new();
        valid_headers.insert("content-type".to_string(), "application/json".to_string());
        valid_headers.insert("authorization".to_string(), "Bearer token".to_string());

        assert!(handler
            .validate_http_request("GET", "/api/test", &valid_headers)
            .is_ok());

        // Test invalid method
        assert!(handler
            .validate_http_request("INVALID", "/api/test", &valid_headers)
            .is_err());

        // Test empty URL
        assert!(handler
            .validate_http_request("GET", "", &valid_headers)
            .is_err());

        // Test invalid header name
        let mut invalid_headers = HashMap::new();
        invalid_headers.insert("".to_string(), "value".to_string());
        assert!(handler
            .validate_http_request("GET", "/api/test", &invalid_headers)
            .is_err());
    }

    #[test]
    fn test_header_filtering() {
        let app_state = create_test_app_state();
        let handler = TunnelHandler::new(app_state);

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("connection".to_string(), "keep-alive".to_string());
        headers.insert("host".to_string(), "example.com".to_string());
        headers.insert("authorization".to_string(), "Bearer token".to_string());

        let filtered = handler.filter_headers(headers);

        assert!(filtered.contains_key("content-type"));
        assert!(filtered.contains_key("authorization"));
        assert!(!filtered.contains_key("connection"));
        assert!(!filtered.contains_key("host"));
    }
}

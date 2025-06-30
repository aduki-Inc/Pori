use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::AppState;

/// API handler for dashboard endpoints
pub struct ApiHandler {
    app_state: Arc<AppState>,
}

impl ApiHandler {
    /// Create new API handler
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Handle API request
    pub async fn handle_request(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        let path = req.uri().path();
        let method = req.method();

        debug!("API request: {} {}", method, path);

        match (method, path) {
            (&Method::GET, "/api/status") => self.handle_status().await,
            (&Method::GET, "/api/stats") => self.handle_stats().await,
            (&Method::GET, "/api/config") => self.handle_config().await,
            (&Method::POST, "/api/reconnect") => self.handle_reconnect().await,
            (&Method::POST, "/api/shutdown") => self.handle_shutdown().await,
            _ => self.handle_not_found(),
        }
    }

    /// Handle status endpoint
    async fn handle_status(&self) -> Result<Response<Full<Bytes>>> {
        let stats = self.app_state.get_stats().await;

        let status = json!({
            "status": "ok",
            "connection_status": stats.connection_status,
            "uptime_seconds": stats.uptime_seconds,
            "requests_processed": stats.requests_processed,
            "websocket_reconnects": stats.websocket_reconnects
        });

        self.json_response(StatusCode::OK, status)
    }

    /// Handle stats endpoint
    async fn handle_stats(&self) -> Result<Response<Full<Bytes>>> {
        let stats = self.app_state.get_stats().await;

        let response = json!({
            "requests_processed": stats.requests_processed,
            "requests_successful": stats.requests_successful,
            "requests_failed": stats.requests_failed,
            "bytes_forwarded": stats.bytes_forwarded,
            "uptime_seconds": stats.uptime_seconds,
            "websocket_reconnects": stats.websocket_reconnects,
            "connection_status": stats.connection_status
        });

        self.json_response(StatusCode::OK, response)
    }

    /// Handle config endpoint
    async fn handle_config(&self) -> Result<Response<Full<Bytes>>> {
        let settings = &self.app_state.settings;

        let config = json!({
            "websocket_url": settings.websocket.url.to_string(),
            "local_server_url": settings.local_server.url.to_string(),
            "dashboard_port": settings.dashboard.port,
            "dashboard_address": settings.dashboard.bind_address,
            "max_reconnects": settings.websocket.max_reconnects,
            "timeout_seconds": settings.websocket.timeout.as_secs(),
            "verify_ssl": settings.local_server.verify_ssl
        });

        self.json_response(StatusCode::OK, config)
    }

    /// Handle reconnect endpoint
    async fn handle_reconnect(&self) -> Result<Response<Full<Bytes>>> {
        // This would trigger a WebSocket reconnection
        // For now, we'll just return a success message
        warn!("Reconnect requested via API");

        let response = json!({
            "status": "success",
            "message": "Reconnection initiated"
        });

        self.json_response(StatusCode::OK, response)
    }

    /// Handle shutdown endpoint
    async fn handle_shutdown(&self) -> Result<Response<Full<Bytes>>> {
        // This would trigger a graceful shutdown
        // For now, we'll just return a message
        warn!("Shutdown requested via API");

        let response = json!({
            "status": "success",
            "message": "Shutdown initiated"
        });

        self.json_response(StatusCode::OK, response)
    }

    /// Handle not found
    fn handle_not_found(&self) -> Result<Response<Full<Bytes>>> {
        let error = json!({
            "error": "Not Found",
            "message": "API endpoint not found"
        });

        self.json_response(StatusCode::NOT_FOUND, error)
    }

    /// Create JSON response
    fn json_response(
        &self,
        status: StatusCode,
        data: serde_json::Value,
    ) -> Result<Response<Full<Bytes>>> {
        let json_string = serde_json::to_string(&data)?;

        let response = Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .header("access-control-allow-origin", "*")
            .header("access-control-allow-methods", "GET, POST, OPTIONS")
            .header("access-control-allow-headers", "content-type")
            .body(Full::new(Bytes::from(json_string)))?;

        Ok(response)
    }

    /// Handle CORS preflight requests
    pub fn handle_cors_preflight(&self) -> Result<Response<Full<Bytes>>> {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("access-control-allow-origin", "*")
            .header("access-control-allow-methods", "GET, POST, OPTIONS")
            .header("access-control-allow-headers", "content-type")
            .header("access-control-max-age", "86400")
            .body(Full::new(Bytes::new()))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{cli::CliArgs, settings::AppSettings};

    fn create_test_app_state() -> Arc<AppState> {
        let args = CliArgs {
            url: "ws://localhost:8080".parse().unwrap(),
            token: "test-token".to_string(),
            protocol: "http".to_string(),
            port: 3000,
            dashboard_port: 8080,
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
    async fn test_status_endpoint() {
        let app_state = create_test_app_state();
        let handler = ApiHandler::new(app_state);

        let response = handler.handle_status().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let app_state = create_test_app_state();
        let handler = ApiHandler::new(app_state);

        let response = handler.handle_stats().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_cors_preflight() {
        let app_state = create_test_app_state();
        let handler = ApiHandler::new(app_state);

        let response = handler.handle_cors_preflight().unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert!(headers.contains_key("access-control-allow-origin"));
        assert!(headers.contains_key("access-control-allow-methods"));
    }
}

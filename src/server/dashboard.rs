use anyhow::Result;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::{AppState, DashboardEvent};
use super::{api::ApiHandler, static_files::{StaticFileHandler, create_default_static_files}};

/// Dashboard service for handling HTTP requests
pub struct DashboardService {
    app_state: Arc<AppState>,
    static_handler: Arc<StaticFileHandler>,
    api_handler: Arc<ApiHandler>,
    events: Arc<RwLock<Vec<DashboardEvent>>>,
}

impl DashboardService {
    /// Create new dashboard service
    pub fn new(app_state: Arc<AppState>) -> Self {
        // Try to load static files, fall back to defaults if needed
        let static_handler = Arc::new(StaticFileHandler::new());
        
        // If no static files were loaded, we could fall back to defaults
        // but for now we'll just use the empty handler
        
        let api_handler = Arc::new(ApiHandler::new(app_state.clone()));

        Self {
            app_state,
            static_handler,
            api_handler,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Handle incoming dashboard event
    pub async fn handle_event(&self, event: DashboardEvent) {
        debug!("Dashboard received event: {:?}", event);
        
        // Store event for dashboard display
        let mut events = self.events.write().await;
        events.push(event);
        
        // Keep only last 100 events
        if events.len() > 100 {
            let drain_count = events.len() - 100;
            events.drain(0..drain_count);
        }
    }

    /// Handle HTTP request
    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let method = req.method();
        let path = req.uri().path();
        let query = req.uri().query();

        debug!("Dashboard request: {} {} {:?}", method, path, query);

        let result = self.handle_request_internal(req).await;
        
        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                error!("Dashboard request error: {}", e);
                
                // Return internal server error
                let error_response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Body::from(format!("Internal Server Error: {}", e)))
                    .unwrap_or_else(|_| {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("Internal Server Error"))
                            .unwrap()
                    });
                
                Ok(error_response)
            }
        }
    }

    /// Internal request handling
    async fn handle_request_internal(&self, req: Request<Body>) -> Result<Response<Body>> {
        let method = req.method();
        let path = req.uri().path();

        // Handle CORS preflight requests
        if method == &Method::OPTIONS {
            return self.api_handler.handle_cors_preflight();
        }

        // Handle API requests
        if path.starts_with("/api/") {
            return self.api_handler.handle_request(req).await;
        }

        // Handle static files
        if let Some(static_file) = self.static_handler.get_file(path) {
            return self.serve_static_file(static_file, &req);
        }

        // Handle root path or fallback to index.html for SPA routing
        if path == "/" || (!path.starts_with("/api/") && !path.contains('.')) {
            if let Some(index_file) = self.static_handler.get_file("index.html") {
                return self.serve_static_file(index_file, &req);
            }
        }

        // If no static files available, serve a basic response
        if self.static_handler.list_files().is_empty() {
            return self.serve_default_dashboard();
        }

        // Not found
        self.handle_not_found()
    }

    /// Serve static file with proper headers
    fn serve_static_file(
        &self,
        static_file: &super::static_files::StaticFile,
        req: &Request<Body>,
    ) -> Result<Response<Body>> {
        // Check If-None-Match header for caching
        if let Some(if_none_match) = req.headers().get("if-none-match") {
            if let Ok(etag_value) = if_none_match.to_str() {
                if etag_value == static_file.etag {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_MODIFIED)
                        .header("etag", &static_file.etag)
                        .body(Body::empty())?);
                }
            }
        }

        let mut response_builder = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", &static_file.mime_type)
            .header("etag", &static_file.etag)
            .header("cache-control", "public, max-age=3600"); // 1 hour cache

        // Add CORS headers
        if self.app_state.settings.dashboard.enable_cors {
            response_builder = response_builder
                .header("access-control-allow-origin", "*")
                .header("access-control-allow-methods", "GET, OPTIONS")
                .header("access-control-allow-headers", "content-type");
        }

        let response = response_builder.body(Body::from(static_file.content.clone()))?;
        Ok(response)
    }

    /// Serve default dashboard when no static files are available
    fn serve_default_dashboard(&self) -> Result<Response<Body>> {
        let default_files = create_default_static_files();
        
        if let Some(index_file) = default_files.get("index.html") {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html")
                .header("access-control-allow-origin", "*")
                .body(Body::from(index_file.content.clone()))?;
            Ok(response)
        } else {
            self.handle_not_found()
        }
    }

    /// Handle 404 not found
    fn handle_not_found(&self) -> Result<Response<Body>> {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>404 - Not Found</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; margin-top: 50px; }
        h1 { color: #e74c3c; }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>404 - Page Not Found</h1>
    <p>The requested page could not be found.</p>
    <p><a href="/">Return to Dashboard</a></p>
</body>
</html>"#;

        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/html")
            .header("access-control-allow-origin", "*")
            .body(Body::from(html))?;

        Ok(response)
    }

    /// Get recent events for dashboard
    pub async fn get_recent_events(&self) -> Vec<DashboardEvent> {
        self.events.read().await.clone()
    }
}

impl Clone for DashboardService {
    fn clone(&self) -> Self {
        Self {
            app_state: self.app_state.clone(),
            static_handler: self.static_handler.clone(),
            api_handler: self.api_handler.clone(),
            events: self.events.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{cli::CliArgs, settings::AppSettings};

    fn create_test_app_state() -> Arc<AppState> {
        let mut args = CliArgs::parse();
        args.url = "ws://localhost:8080".parse().unwrap();
        args.token = "test-token".to_string();
        
        let settings = AppSettings::from_cli(args).unwrap();
        let (app_state, _) = AppState::new(settings);
        Arc::new(app_state)
    }

    #[tokio::test]
    async fn test_dashboard_service_creation() {
        let app_state = create_test_app_state();
        let service = DashboardService::new(app_state);
        
        let events = service.get_recent_events().await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_event_handling() {
        let app_state = create_test_app_state();
        let service = DashboardService::new(app_state);
        
        let event = DashboardEvent::RequestForwarded("GET /test".to_string());
        service.handle_event(event).await;
        
        let events = service.get_recent_events().await;
        assert_eq!(events.len(), 1);
    }
}

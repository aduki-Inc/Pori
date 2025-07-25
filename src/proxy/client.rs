use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};
use url::Url;

use crate::config::settings::HttpVersion;

/// HTTP client for local server communication
#[derive(Clone)]
pub struct LocalServerClient {
    client: Client,
    base_url: Url,
    timeout: Duration,
}

/// Response from a local server
#[derive(Debug)]
pub struct LocalServerResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl LocalServerClient {
    /// Create a new local server client
    pub fn new(
        base_url: Url,
        timeout: Duration,
        verify_ssl: bool,
        http_version: &HttpVersion,
    ) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(!verify_ssl)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(30));

        // Configure HTTP version based on settings
        builder = match http_version {
            HttpVersion::Http1Only => {
                info!("Local server HTTP client: forcing HTTP/1.1");
                builder.http1_only()
            }
            HttpVersion::Http2Only => {
                info!("Local server HTTP client: forcing HTTP/2");
                builder.http2_prior_knowledge()
            }
            HttpVersion::Auto => {
                info!("Local server HTTP client: auto-negotiating HTTP version (HTTP/2 with HTTP/1.1 fallback)");
                // Default behavior - try HTTP/2, fallback to HTTP/1.1
                builder
            }
        };

        let client = builder.build().context("Failed to create an HTTP client")?;

        Ok(Self {
            client,
            base_url,
            timeout,
        })
    }

    /// Forward HTTP request to local server
    pub async fn forward_request(
        &self,
        method: &str,
        path: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<LocalServerResponse> {
        let url = self.build_url(path)?;

        debug!("Forwarding {} {} to a local server", method, url);

        // Build request
        let mut request_builder = self
            .client
            .request(method.parse().context("Invalid HTTP method")?, url.clone());

        // Add headers (excluding certain proxy-specific headers)
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
        let response = request_builder
            .send()
            .await
            .context("Failed to send a request to a local server")?;

        let duration = start_time.elapsed();
        let status = response.status();

        info!(
            "Local server response: {} {} -> {} ({:?})",
            method, path, status, duration
        );

        // Convert response
        let local_response = self.convert_response(response).await?;

        Ok(local_response)
    }

    /// Build target URL from a path
    fn build_url(&self, path: &str) -> Result<Url> {
        let path = if let Some(stripped) = path.strip_prefix('/') {
            stripped
        } else {
            path
        };

        self.base_url
            .join(path)
            .context("Failed to build target URL")
    }

    /// Convert reqwest response to our response type
    async fn convert_response(&self, response: reqwest::Response) -> Result<LocalServerResponse> {
        let status = response.status();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                if !self.should_skip_response_header(key.as_str()) {
                    headers.insert(key.to_string(), value_str.to_string());
                }
            }
        }

        // Read body
        let body_bytes = response
            .bytes()
            .await
            .context("Failed to read the response body")?;

        let body = if body_bytes.is_empty() {
            None
        } else {
            Some(body_bytes.to_vec())
        };

        Ok(LocalServerResponse {
            status: status.as_u16(),
            status_text,
            headers,
            body,
        })
    }

    /// Check if the header should be skipped when forwarding requests
    fn should_skip_header(&self, header_name: &str) -> bool {
        let header_lower = header_name.to_lowercase();
        matches!(
            header_lower.as_str(),
            "host"
                | "connection"
                | "upgrade"
                | "proxy-connection"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "content-length"  // Skip content-length to let reqwest set it correctly
        )
    }

    /// Check if the response header should be skipped
    fn should_skip_response_header(&self, header_name: &str) -> bool {
        let header_lower = header_name.to_lowercase();
        matches!(
            header_lower.as_str(),
            // Connection-related headers
            "connection" | "upgrade" | "proxy-connection" | "transfer-encoding" | "te" | "trailers"
            // Headers that might conflict with proxy operation
            | "x-request-id" | "x-forwarded-by" | "x-forwarded-for" | "x-forwarded-proto"
            // Server-specific headers that shouldn't be forwarded
            | "server" | "x-powered-by"
        )
    }

    /// Get client statistics
    pub fn get_stats(&self) -> ClientStats {
        ClientStats {
            base_url: self.base_url.to_string(),
            timeout: self.timeout,
        }
    }
}

/// Client statistics
#[derive(Debug, Clone)]
pub struct ClientStats {
    pub base_url: String,
    pub timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let base_url: Url = "https://localhost:3000".parse().unwrap();
        let client =
            LocalServerClient::new(base_url, Duration::from_secs(30), false, &HttpVersion::Auto)
                .unwrap();

        let url1 = client.build_url("/api/test").unwrap();
        assert_eq!(url1.to_string(), "https://localhost:3000/api/test");

        let url2 = client.build_url("api/test").unwrap();
        assert_eq!(url2.to_string(), "https://localhost:3000/api/test");
    }

    #[test]
    fn test_header_filtering() {
        let base_url: Url = "https://localhost:3000".parse().unwrap();
        let client =
            LocalServerClient::new(base_url, Duration::from_secs(30), false, &HttpVersion::Auto)
                .unwrap();

        assert!(client.should_skip_header("host"));
        assert!(client.should_skip_header("Connection"));
        assert!(client.should_skip_header("PROXY-CONNECTION"));
        assert!(!client.should_skip_header("content-type"));
        assert!(!client.should_skip_header("authorization"));
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::messages::{HttpPayload, MessagePayload, ProtocolMessage};
use crate::utils::http::status::get_status_text;

/// HTTP-specific message wrapper for local server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMessage {
    /// HTTP message envelope
    pub envelope: HttpEnvelope,
    /// Protocol message payload
    pub message: ProtocolMessage,
}

/// HTTP message envelope with HTTP-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEnvelope {
    /// Request/response identifier
    pub connection_id: String,
    /// HTTP version (1.0, 1.1, 2.0)
    pub http_version: String,
    /// Keep-alive flag
    pub keep_alive: bool,
    /// Proxy information
    pub proxy_info: ProxyInfo,
    /// Request timing
    pub timing: RequestTiming,
}

/// Proxy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    /// Proxy identifier
    pub proxy_id: String,
    /// Client IP address
    pub client_ip: String,
    /// User agent
    pub user_agent: Option<String>,
    /// Forwarded headers
    pub forwarded_for: Vec<String>,
    /// Via headers
    pub via: Vec<String>,
}

/// Request timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTiming {
    /// Request start time (milliseconds since Unix epoch)
    pub start_time: u64,
    /// DNS resolution time in milliseconds
    pub dns_time: Option<u64>,
    /// TCP connection time in milliseconds
    pub connect_time: Option<u64>,
    /// TLS handshake time in milliseconds
    pub tls_time: Option<u64>,
    /// Time to first byte in milliseconds
    pub ttfb: Option<u64>,
    /// Total request time in milliseconds
    pub total_time: Option<u64>,
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestConfig {
    /// Timeout settings
    pub timeouts: HttpTimeouts,
    /// Retry configuration
    pub retry: HttpRetryConfig,
    /// Cache configuration
    pub cache: HttpCacheConfig,
    /// Connection configuration
    pub connection: HttpConnectionConfig,
}

/// HTTP timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTimeouts {
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Response timeout in seconds
    pub response_timeout: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: u64,
}

/// HTTP retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRetryConfig {
    /// Enable retries
    pub enabled: bool,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay in milliseconds
    pub base_delay: u64,
    /// Maximum delay in milliseconds
    pub max_delay: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Retryable status codes
    pub retryable_status_codes: Vec<u16>,
    /// Retryable error types
    pub retryable_errors: Vec<String>,
}

/// HTTP cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache size limit in bytes
    pub max_size: usize,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Cache headers to respect
    pub respect_headers: Vec<String>,
    /// Cacheable methods
    pub cacheable_methods: Vec<String>,
    /// Cacheable status codes
    pub cacheable_status_codes: Vec<u16>,
}

/// HTTP connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpConnectionConfig {
    /// Connection pool settings
    pub pool: ConnectionPoolConfig,
    /// TLS settings
    pub tls: HttpTlsConfig,
    /// Proxy settings
    pub proxy: HttpProxyConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Maximum idle connections
    pub max_idle_connections: usize,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Connection lifetime in seconds
    pub max_lifetime: u64,
    /// Enable HTTP/2
    pub enable_http2: bool,
}

/// HTTP TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTlsConfig {
    /// Verify SSL certificates
    pub verify_ssl: bool,
    /// Client certificate file
    pub client_cert_file: Option<String>,
    /// Client key file
    pub client_key_file: Option<String>,
    /// CA certificate file
    pub ca_cert_file: Option<String>,
    /// SNI hostname
    pub sni_hostname: Option<String>,
    /// Supported protocols
    pub protocols: Vec<String>,
}

/// HTTP proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpProxyConfig {
    /// Proxy URL
    pub url: Option<String>,
    /// Authentication
    pub auth: Option<ProxyAuth>,
    /// Bypass list
    pub bypass: Vec<String>,
}

/// Proxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    /// Authentication type
    pub auth_type: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// HTTP response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponseMetadata {
    /// Response size in bytes
    pub content_length: Option<u64>,
    /// Content type
    pub content_type: Option<String>,
    /// Content encoding
    pub content_encoding: Option<String>,
    /// Last modified
    pub last_modified: Option<String>,
    /// ETag
    pub etag: Option<String>,
    /// Cache control
    pub cache_control: Option<String>,
    /// Server information
    pub server: Option<String>,
}

impl HttpMessage {
    /// Create a new HTTP message
    pub fn new(connection_id: String, message: ProtocolMessage) -> Self {
        Self {
            envelope: HttpEnvelope {
                connection_id,
                http_version: "1.1".to_string(),
                keep_alive: true,
                proxy_info: ProxyInfo {
                    proxy_id: "pori-proxy".to_string(),
                    client_ip: "unknown".to_string(),
                    user_agent: None,
                    forwarded_for: Vec::new(),
                    via: Vec::new(),
                },
                timing: RequestTiming {
                    start_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    dns_time: None,
                    connect_time: None,
                    tls_time: None,
                    ttfb: None,
                    total_time: None,
                },
            },
            message,
        }
    }

    /// Create HTTP request message
    pub fn http_request(
        connection_id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let message = ProtocolMessage::http_request(method, url, headers, body);
        Self::new(connection_id, message)
    }

    /// Create HTTP request message with request ID
    pub fn http_request_with_id(
        connection_id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        let message = ProtocolMessage::http_request_with_id(method, url, headers, body, request_id);
        Self::new(connection_id, message)
    }

    /// Create HTTP response message
    pub fn http_response(
        connection_id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let message = ProtocolMessage::http_response(status, status_text, headers, body);
        Self::new(connection_id, message)
    }

    /// Create HTTP response message with request ID
    pub fn http_response_with_id(
        connection_id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        let message =
            ProtocolMessage::http_response_with_id(status, status_text, headers, body, request_id);
        Self::new(connection_id, message)
    }

    /// Create HTTP error message
    pub fn http_error(connection_id: String, status: u16, error_message: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let message = ProtocolMessage::http_response(
            status,
            get_status_text(status),
            headers,
            Some(error_message.into_bytes()),
        );

        Self::new(connection_id, message)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Get message type
    pub fn message_type(&self) -> &str {
        &self.message.metadata.message_type
    }

    /// Get request ID
    pub fn request_id(&self) -> &str {
        &self.message.metadata.id
    }

    /// Check if message has binary data
    pub fn has_binary_data(&self) -> bool {
        self.message.has_binary_data()
    }

    /// Get body size
    pub fn body_size(&self) -> usize {
        match &self.message.payload {
            MessagePayload::Http(HttpPayload::Request { body, .. }) => {
                body.as_ref().map(|b| b.len()).unwrap_or(0)
            }
            MessagePayload::Http(HttpPayload::Response { body, .. }) => {
                body.as_ref().map(|b| b.len()).unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Extract HTTP request information
    pub fn extract_request_info(&self) -> Option<(String, String, HashMap<String, String>)> {
        if let MessagePayload::Http(HttpPayload::Request {
            method,
            url,
            headers,
            ..
        }) = &self.message.payload
        {
            Some((method.clone(), url.clone(), headers.clone()))
        } else {
            None
        }
    }

    /// Extract HTTP request information with request ID
    pub fn extract_request_info_with_id(
        &self,
    ) -> Option<(String, String, HashMap<String, String>, String)> {
        if let MessagePayload::Http(HttpPayload::Request {
            method,
            url,
            headers,
            request_id,
            ..
        }) = &self.message.payload
        {
            Some((
                method.clone(),
                url.clone(),
                headers.clone(),
                request_id.clone(),
            ))
        } else {
            None
        }
    }

    /// Get request ID from HTTP payload
    pub fn get_request_id(&self) -> Option<String> {
        match &self.message.payload {
            MessagePayload::Http(HttpPayload::Request { request_id, .. }) => {
                Some(request_id.clone())
            }
            MessagePayload::Http(HttpPayload::Response { request_id, .. }) => {
                Some(request_id.clone())
            }
            _ => None,
        }
    }

    /// Extract HTTP response information
    pub fn extract_response_info(&self) -> Option<(u16, String, HashMap<String, String>)> {
        if let MessagePayload::Http(HttpPayload::Response {
            status,
            status_text,
            headers,
            ..
        }) = &self.message.payload
        {
            Some((*status, status_text.clone(), headers.clone()))
        } else {
            None
        }
    }

    /// Update timing information
    pub fn with_timing(mut self, timing: RequestTiming) -> Self {
        self.envelope.timing = timing;
        self
    }

    /// Update proxy information
    pub fn with_proxy_info(mut self, proxy_info: ProxyInfo) -> Self {
        self.envelope.proxy_info = proxy_info;
        self
    }

    /// Update HTTP version
    pub fn with_http_version(mut self, version: String) -> Self {
        self.envelope.http_version = version;
        self
    }
}

impl Default for HttpTimeouts {
    fn default() -> Self {
        Self {
            connect_timeout: 10,
            request_timeout: 60,
            response_timeout: 30,
            keep_alive_timeout: 300,
        }
    }
}

impl Default for HttpRetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            base_delay: 1000,
            max_delay: 30000,
            backoff_multiplier: 2.0,
            retryable_status_codes: vec![429, 502, 503, 504],
            retryable_errors: vec![
                "connection_timeout".to_string(),
                "connection_refused".to_string(),
                "dns_failure".to_string(),
            ],
        }
    }
}

impl Default for HttpCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_size: 100 * 1024 * 1024, // 100MB
            default_ttl: 300,
            respect_headers: vec![
                "cache-control".to_string(),
                "expires".to_string(),
                "etag".to_string(),
                "last-modified".to_string(),
            ],
            cacheable_methods: vec!["GET".to_string(), "HEAD".to_string()],
            cacheable_status_codes: vec![200, 203, 300, 301, 410],
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_idle_connections: 100,
            idle_timeout: 300,
            max_lifetime: 3600,
            enable_http2: false,
        }
    }
}

impl Default for HttpTlsConfig {
    fn default() -> Self {
        Self {
            verify_ssl: true,
            client_cert_file: None,
            client_key_file: None,
            ca_cert_file: None,
            sni_hostname: None,
            protocols: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_message_creation() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let message = HttpMessage::http_request(
            "conn-1".to_string(),
            "GET".to_string(),
            "/api/test".to_string(),
            headers,
            None,
        );

        assert_eq!(message.envelope.connection_id, "conn-1");
        assert_eq!(message.message_type(), "http_request");
    }

    #[test]
    fn test_status_text_mapping() {
        assert_eq!(get_status_text(200), "OK");
        assert_eq!(get_status_text(404), "Not Found");
        assert_eq!(get_status_text(500), "Internal Server Error");
        assert_eq!(get_status_text(999), "Unknown");
    }

    #[test]
    fn test_request_info_extraction() {
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), "example.com".to_string());

        let message = HttpMessage::http_request(
            "conn-1".to_string(),
            "POST".to_string(),
            "/api/data".to_string(),
            headers.clone(),
            Some(b"test data".to_vec()),
        );

        let (method, url, extracted_headers) = message.extract_request_info().unwrap();
        assert_eq!(method, "POST");
        assert_eq!(url, "/api/data");
        assert_eq!(extracted_headers, headers);
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::messages::{ErrorCategory, HttpPayload, MessagePayload, ProtocolMessage};

/// Tunnel-specific message wrapper for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelMessage {
    /// Tunnel message envelope
    pub envelope: TunnelEnvelope,
    /// Protocol message payload
    pub message: ProtocolMessage,
}

/// Tunnel message envelope with tunnel-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelEnvelope {
    /// Tunnel identifier
    pub tunnel_id: String,
    /// Client identifier
    pub client_id: String,
    /// Server identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    /// Tunnel protocol version (optional, defaults to "1.0")
    #[serde(
        default = "default_protocol_version",
        skip_serializing_if = "Option::is_none"
    )]
    pub protocol_version: Option<String>,
    /// Compression applied (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
    /// Encryption applied (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,
    /// Routing information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<TunnelRouting>,
}

fn default_protocol_version() -> Option<String> {
    Some("1.0".to_string())
}

/// Tunnel routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRouting {
    /// Source endpoint
    pub source: TunnelEndpoint,
    /// Destination endpoint
    pub destination: TunnelEndpoint,
    /// Routing path
    pub path: Vec<String>,
    /// Load balancing strategy
    pub load_balancing: Option<String>,
}

/// Tunnel endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelEndpoint {
    /// Endpoint type (client, server, proxy)
    pub endpoint_type: String,
    /// Endpoint identifier
    pub id: String,
    /// Endpoint address
    pub address: Option<String>,
    /// Endpoint capabilities
    pub capabilities: Vec<String>,
}

/// Tunnel configuration message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// Tunnel settings
    pub settings: TunnelSettings,
    /// Authentication configuration
    pub auth: TunnelAuthConfig,
    /// Network configuration
    pub network: TunnelNetworkConfig,
    /// Security configuration
    pub security: TunnelSecurityConfig,
}

/// Tunnel settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelSettings {
    /// Keep-alive interval in seconds
    pub keep_alive_interval: u64,
    /// Ping timeout in seconds
    pub ping_timeout: u64,
    /// Max idle time in seconds
    pub max_idle_time: u64,
    /// Buffer size in bytes
    pub buffer_size: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Enable encryption
    pub enable_encryption: bool,
}

/// Tunnel authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelAuthConfig {
    /// Authentication method
    pub method: String,
    /// Required scopes
    pub required_scopes: Vec<String>,
    /// Token validation endpoint
    pub validation_endpoint: Option<String>,
    /// Certificate validation
    pub certificate_validation: bool,
}

/// Tunnel network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelNetworkConfig {
    /// Bind address
    pub bind_address: String,
    /// Port range
    pub port_range: (u16, u16),
    /// Protocol family (ipv4, ipv6, dual)
    pub protocol_family: String,
    /// TCP configuration
    pub tcp: TcpConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
}

/// TCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// No delay (Nagle algorithm)
    pub no_delay: bool,
    /// Keep alive
    pub keep_alive: bool,
    /// Keep alive timeout
    pub keep_alive_timeout: u64,
    /// Send buffer size
    pub send_buffer_size: Option<usize>,
    /// Receive buffer size
    pub recv_buffer_size: Option<usize>,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Max frame size
    pub max_frame_size: usize,
    /// Max message size
    pub max_message_size: usize,
    /// Enable per-message deflate
    pub enable_per_message_deflate: bool,
    /// Deflate window bits
    pub deflate_window_bits: u8,
    /// Auto fragment size
    pub auto_fragment_size: Option<usize>,
}

/// Tunnel security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TunnelSecurityConfig {
    /// TLS configuration
    pub tls: TlsConfig,
    /// Access control
    pub access_control: AccessControlConfig,
    /// Rate limiting
    pub rate_limiting: RateLimitConfig,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Certificate file path
    pub cert_file: Option<String>,
    /// Private key file path
    pub key_file: Option<String>,
    /// CA certificate file path
    pub ca_file: Option<String>,
    /// Verify peer certificate
    pub verify_peer: bool,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Blocked IP addresses
    pub blocked_ips: Vec<String>,
    /// Authentication required
    pub auth_required: bool,
    /// Permissions
    pub permissions: HashMap<String, Vec<String>>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enabled
    pub enabled: bool,
    /// Requests per second
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
    /// Window size in seconds
    pub window_size: u64,
    /// Block duration in seconds
    pub block_duration: u64,
}

impl TunnelMessage {
    /// Create a new tunnel message
    pub fn new(tunnel_id: String, client_id: String, message: ProtocolMessage) -> Self {
        Self {
            envelope: TunnelEnvelope {
                tunnel_id,
                client_id,
                server_id: None,
                protocol_version: Some("1.0.0".to_string()),
                compression: None,
                encryption: None,
                routing: None,
            },
            message,
        }
    }

    /// Create HTTP request tunnel message
    pub fn http_request(
        tunnel_id: String,
        client_id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let message = ProtocolMessage::http_request(method, url, headers, body);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create HTTP request tunnel message with request ID
    pub fn http_request_with_id(
        tunnel_id: String,
        client_id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        let message = ProtocolMessage::http_request_with_id(method, url, headers, body, request_id);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create HTTP response tunnel message
    pub fn http_response(
        tunnel_id: String,
        client_id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let message = ProtocolMessage::http_response(status, status_text, headers, body);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create HTTP response tunnel message with request ID
    pub fn http_response_with_id(
        tunnel_id: String,
        client_id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        let message =
            ProtocolMessage::http_response_with_id(status, status_text, headers, body, request_id);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create authentication tunnel message
    pub fn auth_token(
        tunnel_id: String,
        client_id: String,
        token: String,
        token_type: String,
        scopes: Vec<String>,
    ) -> Self {
        let message = ProtocolMessage::auth_token(token, token_type, scopes);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create ping tunnel message
    pub fn ping(tunnel_id: String, client_id: String) -> Self {
        let message = ProtocolMessage::ping();
        Self::new(tunnel_id, client_id, message)
    }

    /// Create pong tunnel message
    pub fn pong(tunnel_id: String, client_id: String, timestamp: u64) -> Self {
        let message = ProtocolMessage::pong(timestamp);
        Self::new(tunnel_id, client_id, message)
    }

    /// Create error tunnel message
    pub fn error(
        tunnel_id: String,
        client_id: String,
        code: String,
        message: String,
        category: ErrorCategory,
        related_id: Option<String>,
    ) -> Self {
        let message = ProtocolMessage::error(code, message, category, related_id);
        Self::new(tunnel_id, client_id, message)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serialize to binary
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(Into::into)
    }

    /// Deserialize from binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data).map_err(Into::into)
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

    /// Check if this is a control message
    pub fn is_control_message(&self) -> bool {
        matches!(
            &self.message.payload,
            MessagePayload::Auth(_) | MessagePayload::Control(_) | MessagePayload::Stats(_)
        )
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
}

impl Default for TunnelSettings {
    fn default() -> Self {
        Self {
            keep_alive_interval: 30,
            ping_timeout: 10,
            max_idle_time: 300,
            buffer_size: 64 * 1024,
            enable_compression: true,
            enable_encryption: false,
        }
    }
}

impl Default for TunnelAuthConfig {
    fn default() -> Self {
        Self {
            method: "token".to_string(),
            required_scopes: Vec::new(),
            validation_endpoint: None,
            certificate_validation: false,
        }
    }
}

impl Default for TunnelNetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port_range: (8000, 9000),
            protocol_family: "dual".to_string(),
            tcp: TcpConfig::default(),
            websocket: WebSocketConfig::default(),
        }
    }
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            no_delay: true,
            keep_alive: true,
            keep_alive_timeout: 300,
            send_buffer_size: Some(64 * 1024),
            recv_buffer_size: Some(64 * 1024),
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_frame_size: 16 * 1024 * 1024,
            max_message_size: 16 * 1024 * 1024,
            enable_per_message_deflate: true,
            deflate_window_bits: 15,
            auto_fragment_size: Some(1024 * 1024),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_file: None,
            key_file: None,
            ca_file: None,
            verify_peer: false,
            protocols: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
            cipher_suites: Vec::new(),
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_ips: Vec::new(),
            blocked_ips: Vec::new(),
            auth_required: false,
            permissions: HashMap::new(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100,
            burst_size: 10,
            window_size: 60,
            block_duration: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_message_creation() {
        let message = TunnelMessage::ping("tunnel-1".to_string(), "client-1".to_string());
        assert_eq!(message.envelope.tunnel_id, "tunnel-1");
        assert_eq!(message.envelope.client_id, "client-1");
        assert_eq!(message.message_type(), "ping");
    }

    #[test]
    fn test_tunnel_http_request() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let message = TunnelMessage::http_request(
            "tunnel-1".to_string(),
            "client-1".to_string(),
            "GET".to_string(),
            "/api/test".to_string(),
            headers,
            None,
        );

        assert_eq!(message.message_type(), "http_request");
        assert!(!message.has_binary_data());
    }

    #[test]
    fn test_tunnel_serialization() {
        let message = TunnelMessage::ping("tunnel-1".to_string(), "client-1".to_string());
        let json = message.to_json().unwrap();
        let deserialized = TunnelMessage::from_json(&json).unwrap();

        assert_eq!(message.envelope.tunnel_id, deserialized.envelope.tunnel_id);
        assert_eq!(message.message_type(), deserialized.message_type());
    }
}

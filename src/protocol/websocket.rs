use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::messages::{
    ConnectionStatus, ControlPayload, ErrorCategory, MessagePayload, ProtocolMessage,
};

/// WebSocket-specific message wrapper for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// WebSocket message envelope
    pub envelope: WebSocketEnvelope,
    /// Protocol message payload
    pub message: ProtocolMessage,
}

/// WebSocket message envelope with WebSocket-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEnvelope {
    /// WebSocket connection identifier
    pub connection_id: String,
    /// WebSocket protocol version
    pub ws_version: String,
    /// Message frame type
    pub frame_type: FrameType,
    /// Compression information
    pub compression: Option<CompressionInfo>,
    /// Extension data
    pub extensions: HashMap<String, String>,
}

/// WebSocket frame types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameType {
    Text,
    Binary,
    Close,
    Ping,
    Pong,
    Continuation,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Compression algorithm
    pub algorithm: String,
    /// Compression level
    pub level: u8,
    /// Original size before compression
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Connection settings
    pub connection: WebSocketConnectionConfig,
    /// Message settings
    pub message: WebSocketMessageConfig,
    /// Security settings
    pub security: WebSocketSecurityConfig,
    /// Extension settings
    pub extensions: WebSocketExtensionConfig,
}

/// WebSocket connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConnectionConfig {
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Ping interval in seconds
    pub ping_interval: u64,
    /// Pong timeout in seconds
    pub pong_timeout: u64,
    /// Max idle time in seconds
    pub max_idle_time: u64,
    /// Reconnection settings
    pub reconnect: ReconnectConfig,
    /// Keep-alive settings
    pub keep_alive: KeepAliveConfig,
}

/// Reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Enable auto-reconnect
    pub enabled: bool,
    /// Maximum reconnection attempts
    pub max_attempts: u32,
    /// Base delay in milliseconds
    pub base_delay: u64,
    /// Maximum delay in milliseconds
    pub max_delay: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0-1.0)
    pub jitter_factor: f64,
}

/// Keep-alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepAliveConfig {
    /// Enable keep-alive
    pub enabled: bool,
    /// Ping interval in seconds
    pub ping_interval: u64,
    /// Max missed pongs before disconnect
    pub max_missed_pongs: u32,
    /// Custom ping data
    pub ping_data: Option<Vec<u8>>,
}

/// WebSocket message configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessageConfig {
    /// Maximum frame size in bytes
    pub max_frame_size: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Auto-fragment large messages
    pub auto_fragment: bool,
    /// Fragment size for auto-fragmentation
    pub fragment_size: usize,
    /// Message queue size
    pub queue_size: usize,
    /// Enable message ordering
    pub ordering_enabled: bool,
}

/// WebSocket security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSecurityConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Required subprotocols
    pub required_subprotocols: Vec<String>,
    /// Custom headers validation
    pub header_validation: HeaderValidationConfig,
    /// Rate limiting
    pub rate_limiting: WebSocketRateLimitConfig,
}

/// Header validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderValidationConfig {
    /// Required headers
    pub required_headers: HashMap<String, String>,
    /// Forbidden headers
    pub forbidden_headers: Vec<String>,
    /// Header size limit
    pub max_header_size: usize,
}

/// WebSocket rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketRateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Messages per second
    pub messages_per_second: u32,
    /// Burst size
    pub burst_size: u32,
    /// Data rate limit in bytes per second
    pub bytes_per_second: u64,
    /// Window size in seconds
    pub window_size: u64,
}

/// WebSocket extension configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketExtensionConfig {
    /// Per-message deflate settings
    pub deflate: DeflateConfig,
    /// Custom extensions
    pub custom: HashMap<String, serde_json::Value>,
}

/// Per-message deflate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeflateConfig {
    /// Enable per-message deflate
    pub enabled: bool,
    /// Server max window bits
    pub server_max_window_bits: u8,
    /// Client max window bits
    pub client_max_window_bits: u8,
    /// Server no context takeover
    pub server_no_context_takeover: bool,
    /// Client no context takeover
    pub client_no_context_takeover: bool,
    /// Compression threshold
    pub compression_threshold: usize,
}

/// WebSocket handshake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeInfo {
    /// Request URI
    pub uri: String,
    /// Origin header
    pub origin: Option<String>,
    /// Sec-WebSocket-Protocol header
    pub protocols: Vec<String>,
    /// Sec-WebSocket-Extensions header
    pub extensions: Vec<String>,
    /// Sec-WebSocket-Version header
    pub version: String,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// WebSocket close information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseInfo {
    /// Close code
    pub code: u16,
    /// Close reason
    pub reason: String,
    /// Whether close was initiated locally
    pub initiated_locally: bool,
    /// Timestamp of close
    pub timestamp: u64,
}

impl WebSocketMessage {
    /// Create a new WebSocket message
    pub fn new(connection_id: String, frame_type: FrameType, message: ProtocolMessage) -> Self {
        Self {
            envelope: WebSocketEnvelope {
                connection_id,
                ws_version: "13".to_string(),
                frame_type,
                compression: None,
                extensions: HashMap::new(),
            },
            message,
        }
    }

    /// Create a text WebSocket message
    pub fn text(connection_id: String, message: ProtocolMessage) -> Self {
        Self::new(connection_id, FrameType::Text, message)
    }

    /// Create a binary WebSocket message
    pub fn binary(connection_id: String, message: ProtocolMessage) -> Self {
        Self::new(connection_id, FrameType::Binary, message)
    }

    /// Create a ping WebSocket message
    pub fn ping(connection_id: String) -> Self {
        let message = ProtocolMessage::ping();
        Self::new(connection_id, FrameType::Ping, message)
    }

    /// Create a pong WebSocket message
    pub fn pong(connection_id: String, timestamp: u64) -> Self {
        let message = ProtocolMessage::pong(timestamp);
        Self::new(connection_id, FrameType::Pong, message)
    }

    /// Create a close WebSocket message
    pub fn close(connection_id: String, code: u16, reason: String) -> Self {
        let message = ProtocolMessage::new(
            "close".to_string(),
            MessagePayload::Control(ControlPayload::Status {
                status: ConnectionStatus::Disconnected,
                message: Some(reason),
                details: {
                    let mut details = HashMap::new();
                    details.insert("close_code".to_string(), code.to_string());
                    details
                },
            }),
        );
        Self::new(connection_id, FrameType::Close, message)
    }

    /// Create an authentication WebSocket message
    pub fn auth_token(
        connection_id: String,
        token: String,
        token_type: String,
        scopes: Vec<String>,
    ) -> Self {
        let message = ProtocolMessage::auth_token(token, token_type, scopes);
        Self::text(connection_id, message)
    }

    /// Create an error WebSocket message
    pub fn error(
        connection_id: String,
        code: String,
        message: String,
        category: ErrorCategory,
    ) -> Self {
        let message = ProtocolMessage::error(code, message, category, None);
        Self::text(connection_id, message)
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

    /// Get connection ID
    pub fn connection_id(&self) -> &str {
        &self.envelope.connection_id
    }

    /// Get frame type
    pub fn frame_type(&self) -> &FrameType {
        &self.envelope.frame_type
    }

    /// Check if message has binary data
    pub fn has_binary_data(&self) -> bool {
        matches!(self.envelope.frame_type, FrameType::Binary) || self.message.has_binary_data()
    }

    /// Get message size
    pub fn message_size(&self) -> usize {
        self.message.size()
    }

    /// Check if this is a control frame
    pub fn is_control_frame(&self) -> bool {
        matches!(
            self.envelope.frame_type,
            FrameType::Close | FrameType::Ping | FrameType::Pong
        )
    }

    /// Add compression information
    pub fn with_compression(mut self, compression: CompressionInfo) -> Self {
        self.envelope.compression = Some(compression);
        self
    }

    /// Add extension data
    pub fn with_extension(mut self, name: String, value: String) -> Self {
        self.envelope.extensions.insert(name, value);
        self
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connection: WebSocketConnectionConfig::default(),
            message: WebSocketMessageConfig::default(),
            security: WebSocketSecurityConfig::default(),
            extensions: WebSocketExtensionConfig::default(),
        }
    }
}

impl Default for WebSocketConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 30,
            ping_interval: 30,
            pong_timeout: 10,
            max_idle_time: 300,
            reconnect: ReconnectConfig::default(),
            keep_alive: KeepAliveConfig::default(),
        }
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            base_delay: 1000,
            max_delay: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl Default for KeepAliveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ping_interval: 30,
            max_missed_pongs: 3,
            ping_data: None,
        }
    }
}

impl Default for WebSocketMessageConfig {
    fn default() -> Self {
        Self {
            max_frame_size: 16 * 1024 * 1024,
            max_message_size: 16 * 1024 * 1024,
            auto_fragment: true,
            fragment_size: 1024 * 1024,
            queue_size: 1000,
            ordering_enabled: false,
        }
    }
}

impl Default for WebSocketSecurityConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            required_subprotocols: Vec::new(),
            header_validation: HeaderValidationConfig::default(),
            rate_limiting: WebSocketRateLimitConfig::default(),
        }
    }
}

impl Default for HeaderValidationConfig {
    fn default() -> Self {
        Self {
            required_headers: HashMap::new(),
            forbidden_headers: Vec::new(),
            max_header_size: 8192,
        }
    }
}

impl Default for WebSocketRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            messages_per_second: 100,
            burst_size: 10,
            bytes_per_second: 1024 * 1024,
            window_size: 60,
        }
    }
}

impl Default for WebSocketExtensionConfig {
    fn default() -> Self {
        Self {
            deflate: DeflateConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for DeflateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_max_window_bits: 15,
            client_max_window_bits: 15,
            server_no_context_takeover: false,
            client_no_context_takeover: false,
            compression_threshold: 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_creation() {
        let message = WebSocketMessage::ping("ws-conn-1".to_string());
        assert_eq!(message.connection_id(), "ws-conn-1");
        assert_eq!(message.message_type(), "ping");
        assert!(matches!(message.frame_type(), FrameType::Ping));
        assert!(message.is_control_frame());
    }

    #[test]
    fn test_websocket_auth_message() {
        let message = WebSocketMessage::auth_token(
            "ws-conn-1".to_string(),
            "test-token".to_string(),
            "Bearer".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert_eq!(message.connection_id(), "ws-conn-1");
        assert!(matches!(message.frame_type(), FrameType::Text));
        assert!(!message.is_control_frame());
    }

    #[test]
    fn test_websocket_serialization() {
        let message = WebSocketMessage::ping("ws-conn-1".to_string());
        let json = message.to_json().unwrap();
        let deserialized = WebSocketMessage::from_json(&json).unwrap();

        assert_eq!(message.connection_id(), deserialized.connection_id());
        assert_eq!(message.message_type(), deserialized.message_type());
    }

    #[test]
    fn test_compression_info() {
        let compression = CompressionInfo {
            algorithm: "deflate".to_string(),
            level: 6,
            original_size: 1000,
            compressed_size: 600,
        };

        let message = WebSocketMessage::ping("ws-conn-1".to_string()).with_compression(compression);

        assert!(message.envelope.compression.is_some());
    }
}

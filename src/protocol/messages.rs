use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::config::{DeliveryMode, MessageEncoding, MessagePriority};

/// Base message structure for all protocol communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Message metadata
    pub metadata: MessageMetadata,
    /// Message payload
    pub payload: MessagePayload,
}

/// Message metadata containing routing and control information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Unique message identifier
    pub id: String,
    /// Message type identifier
    pub message_type: String,
    /// Protocol version
    pub version: String,
    /// Timestamp in milliseconds since Unix epoch
    pub timestamp: u64,
    /// Message priority
    pub priority: MessagePriority,
    /// Delivery mode
    pub delivery_mode: DeliveryMode,
    /// Encoding format
    pub encoding: MessageEncoding,
    /// Source identifier
    pub source: Option<String>,
    /// Destination identifier
    pub destination: Option<String>,
    /// Correlation ID for request-response pairs
    pub correlation_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Message tags for routing and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Retry count
    #[serde(default)]
    pub retry_count: u32,
    /// Max retry attempts
    #[serde(default)]
    pub max_retries: u32,
    /// TTL in seconds
    pub ttl: Option<u64>,
}

/// Message payload containing the actual data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum MessagePayload {
    /// Authentication and authorization messages
    Auth(AuthPayload),
    /// HTTP request/response messages
    #[serde(rename = "HTTP")]
    Http(HttpPayload),
    /// Control and management messages
    Control(ControlPayload),
    /// Statistics and monitoring messages
    Stats(StatsPayload),
    /// Error messages
    Error(ErrorPayload),
    /// Streaming data messages
    Stream(StreamPayload),
    /// Custom application messages
    Custom(CustomPayload),
}

/// Authentication payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "auth_type")]
pub enum AuthPayload {
    /// Token-based authentication
    TokenAuth {
        token: String,
        token_type: String,
        scopes: Vec<String>,
    },
    /// Challenge-response authentication
    Challenge { challenge: String, method: String },
    /// Authentication response
    Response {
        response: String,
        proof: Option<String>,
    },
    /// Authentication success
    Success {
        session_id: String,
        expires_at: Option<u64>,
        permissions: Vec<String>,
    },
    /// Authentication failure
    Failure {
        error_code: String,
        error_message: String,
        retry_after: Option<u64>,
    },
}

/// HTTP communication payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HttpPayload {
    /// HTTP request
    Request {
        method: String,
        url: String,
        headers: HashMap<String, String>,
        #[serde(with = "body_serializer", default)]
        body: Option<Vec<u8>>,
        #[serde(default)]
        query_params: HashMap<String, String>,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    /// HTTP response
    Response {
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        #[serde(with = "body_serializer", default)]
        body: Option<Vec<u8>>,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    /// HTTP upgrade request
    Upgrade {
        protocol: String,
        headers: HashMap<String, String>,
    },
    /// HTTP connection close
    Close { reason: String },
}

/// Control message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ControlPayload {
    /// Authentication status
    Authentication {
        status: String,
        message: String,
        timestamp: String,
    },
    /// Error message
    Error {
        error: String,
        code: String,
        timestamp: String,
    },
    /// Ping for connection health
    Ping {
        timestamp: u64,
        data: Option<Vec<u8>>,
    },
    /// Pong response to ping
    Pong {
        timestamp: u64,
        data: Option<Vec<u8>>,
    },
    /// Connection status update
    Status {
        status: ConnectionStatus,
        message: Option<String>,
        details: HashMap<String, String>,
    },
    /// Configuration update
    Config {
        config_type: String,
        config_data: serde_json::Value,
    },
    /// Command execution
    Command {
        command: String,
        arguments: Vec<String>,
        environment: HashMap<String, String>,
    },
    /// Shutdown notification
    Shutdown {
        reason: String,
        grace_period_seconds: u64,
    },
}

/// Statistics payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stats_type")]
pub enum StatsPayload {
    /// Connection statistics
    Connection {
        connected_clients: u64,
        total_connections: u64,
        failed_connections: u64,
        uptime_seconds: u64,
    },
    /// Request/response statistics
    Traffic {
        requests_processed: u64,
        requests_successful: u64,
        requests_failed: u64,
        bytes_transferred: u64,
        average_response_time_ms: f64,
    },
    /// System resource statistics
    System {
        cpu_usage_percent: f64,
        memory_usage_bytes: u64,
        memory_total_bytes: u64,
        disk_usage_bytes: u64,
        network_bytes_sent: u64,
        network_bytes_received: u64,
    },
    /// Application-specific metrics
    Custom {
        metric_name: String,
        metric_value: serde_json::Value,
        metric_type: String,
        labels: HashMap<String, String>,
    },
}

/// Error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Detailed error description
    pub details: Option<String>,
    /// Stack trace or debug information
    pub trace: Option<String>,
    /// Related request/message ID
    pub related_id: Option<String>,
    /// Error category
    pub category: ErrorCategory,
    /// Suggested recovery actions
    pub recovery_actions: Vec<String>,
}

/// Stream data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stream_type")]
pub enum StreamPayload {
    /// Stream data chunk
    Data {
        stream_id: String,
        sequence: u64,
        data: Vec<u8>,
        is_final: bool,
    },
    /// Stream control message
    Control {
        stream_id: String,
        action: StreamAction,
        metadata: HashMap<String, String>,
    },
}

/// Custom application payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPayload {
    /// Custom message type
    pub message_type: String,
    /// Custom data
    pub data: serde_json::Value,
    /// Schema version
    pub schema_version: String,
}

/// Connection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Authenticated,
    Disconnecting,
    Disconnected,
    Reconnecting,
    Failed,
}

/// Error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    Authentication,
    Authorization,
    Network,
    Protocol,
    Validation,
    Internal,
    External,
    Timeout,
    RateLimit,
}

/// Stream actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamAction {
    Start,
    Pause,
    Resume,
    Stop,
    Reset,
}

impl ProtocolMessage {
    /// Create a new protocol message
    pub fn new(message_type: String, payload: MessagePayload) -> Self {
        Self {
            metadata: MessageMetadata::new(message_type),
            payload,
        }
    }

    /// Create an HTTP request message
    pub fn http_request(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let mut query_params = HashMap::new();

        // Extract query parameters from URL
        if let Ok(parsed_url) = url::Url::parse(&url) {
            for (key, value) in parsed_url.query_pairs() {
                query_params.insert(key.to_string(), value.to_string());
            }
        }

        Self::new(
            "http_request".to_string(),
            MessagePayload::Http(HttpPayload::Request {
                method,
                url,
                headers,
                body,
                query_params,
                request_id: Uuid::new_v4().to_string(), // Generate a default requestId
            }),
        )
    }

    /// Create an HTTP request message with request ID
    pub fn http_request_with_id(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        let mut query_params = HashMap::new();

        // Extract query parameters from URL
        if let Ok(parsed_url) = url::Url::parse(&url) {
            for (key, value) in parsed_url.query_pairs() {
                query_params.insert(key.to_string(), value.to_string());
            }
        }

        Self::new(
            "http_request".to_string(),
            MessagePayload::Http(HttpPayload::Request {
                method,
                url,
                headers,
                body,
                query_params,
                request_id,
            }),
        )
    }

    /// Create an HTTP response message
    pub fn http_response(
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self::new(
            "http_response".to_string(),
            MessagePayload::Http(HttpPayload::Response {
                status,
                status_text,
                headers,
                body,
                request_id: Uuid::new_v4().to_string(), // Generate a default requestId
            }),
        )
    }

    /// Create an HTTP response message with request ID
    pub fn http_response_with_id(
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        request_id: String,
    ) -> Self {
        Self::new(
            "http_response".to_string(),
            MessagePayload::Http(HttpPayload::Response {
                status,
                status_text,
                headers,
                body,
                request_id,
            }),
        )
    }

    /// Create an authentication message
    pub fn auth_token(token: String, token_type: String, scopes: Vec<String>) -> Self {
        Self::new(
            "auth_token".to_string(),
            MessagePayload::Auth(AuthPayload::TokenAuth {
                token,
                token_type,
                scopes,
            }),
        )
    }

    /// Create an error message
    pub fn error(
        code: String,
        message: String,
        category: ErrorCategory,
        related_id: Option<String>,
    ) -> Self {
        Self::new(
            "error".to_string(),
            MessagePayload::Error(ErrorPayload {
                code,
                message,
                details: None,
                trace: None,
                related_id,
                category,
                recovery_actions: Vec::new(),
            }),
        )
    }

    /// Create a ping message
    pub fn ping() -> Self {
        Self::new(
            "ping".to_string(),
            MessagePayload::Control(ControlPayload::Ping {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                data: None,
            }),
        )
    }

    /// Create a pong message
    pub fn pong(timestamp: u64) -> Self {
        Self::new(
            "pong".to_string(),
            MessagePayload::Control(ControlPayload::Pong {
                timestamp,
                data: None,
            }),
        )
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serialize to binary (MessagePack)
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(Into::into)
    }

    /// Deserialize from binary (MessagePack)
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data).map_err(Into::into)
    }

    /// Get message size in bytes
    pub fn size(&self) -> usize {
        self.to_json().map(|s| s.len()).unwrap_or(0)
    }

    /// Check if message has binary data
    pub fn has_binary_data(&self) -> bool {
        match &self.payload {
            MessagePayload::Http(HttpPayload::Request { body, .. }) => body.is_some(),
            MessagePayload::Http(HttpPayload::Response { body, .. }) => body.is_some(),
            MessagePayload::Stream(StreamPayload::Data { .. }) => true,
            _ => false,
        }
    }

    /// Set correlation ID for request-response tracking
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.metadata.session_id = Some(session_id);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// Add header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.metadata.headers.insert(key, value);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.metadata.tags.push(tag);
        self
    }
}

impl MessageMetadata {
    /// Create new message metadata
    pub fn new(message_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_type,
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            priority: MessagePriority::Normal,
            delivery_mode: DeliveryMode::AtLeastOnce,
            encoding: MessageEncoding::Json,
            source: None,
            destination: None,
            correlation_id: None,
            session_id: None,
            headers: HashMap::new(),
            tags: Vec::new(),
            retry_count: 0,
            max_retries: 3,
            ttl: None,
        }
    }
}

/// Custom serializer for body field that can handle both JSON objects and byte arrays
mod body_serializer {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::Value;

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => {
                // Try to parse as JSON first, fallback to raw bytes if that fails
                if let Ok(json_str) = std::str::from_utf8(bytes) {
                    if let Ok(json_value) = serde_json::from_str::<Value>(json_str) {
                        json_value.serialize(serializer)
                    } else {
                        // Serialize as byte array
                        bytes.serialize(serializer)
                    }
                } else {
                    // Serialize as byte array
                    bytes.serialize(serializer)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value: Option<Value> = Option::deserialize(deserializer)?;
        match value {
            Some(val) => match val {
                // Handle case where body is sent as JSON object (but not array)
                Value::Object(_) => {
                    let json_str = serde_json::to_string(&val).map_err(|e| {
                        D::Error::custom(format!("Failed to serialize JSON body: {}", e))
                    })?;
                    Ok(Some(json_str.into_bytes()))
                }
                // Handle case where body is sent as a string
                Value::String(s) => Ok(Some(s.into_bytes())),
                // Handle case where body is sent as byte array
                Value::Array(arr) => {
                    let bytes: Result<Vec<u8>, _> = arr
                        .into_iter()
                        .map(|v| match v {
                            Value::Number(n) => {
                                if let Some(byte_val) = n.as_u64() {
                                    if byte_val <= 255 {
                                        Ok(byte_val as u8)
                                    } else {
                                        Err(D::Error::custom("Invalid byte value"))
                                    }
                                } else {
                                    Err(D::Error::custom("Invalid byte value"))
                                }
                            }
                            _ => Err(D::Error::custom("Invalid byte array format")),
                        })
                        .collect();
                    Ok(Some(bytes.map_err(|e| D::Error::custom(e))?))
                }
                // Handle null values
                Value::Null => Ok(None),
                // For other types, try to convert to string then bytes
                _ => {
                    let json_str = serde_json::to_string(&val).map_err(|e| {
                        D::Error::custom(format!("Failed to serialize body value: {}", e))
                    })?;
                    Ok(Some(json_str.into_bytes()))
                }
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_message() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let message = ProtocolMessage::http_request(
            "GET".to_string(),
            "/api/test".to_string(),
            headers,
            Some(b"test body".to_vec()),
        );

        assert_eq!(message.metadata.message_type, "http_request");
        assert!(message.has_binary_data());
    }

    #[test]
    fn test_message_serialization() {
        let message = ProtocolMessage::ping();
        let json = message.to_json().unwrap();
        let deserialized = ProtocolMessage::from_json(&json).unwrap();

        assert_eq!(
            message.metadata.message_type,
            deserialized.metadata.message_type
        );
    }

    #[test]
    fn test_message_builder_pattern() {
        let message = ProtocolMessage::ping()
            .with_correlation_id("test-correlation".to_string())
            .with_priority(MessagePriority::High)
            .with_tag("test".to_string());

        assert_eq!(
            message.metadata.correlation_id,
            Some("test-correlation".to_string())
        );
        assert_eq!(message.metadata.priority, MessagePriority::High);
        assert_eq!(message.metadata.tags, vec!["test".to_string()]);
    }

    #[test]
    fn test_json_body_deserialization() {
        use crate::protocol::tunnel::TunnelMessage;

        let json_with_object_body = r#"{
            "envelope": {
                "tunnel_id": "tunnel_1752429342302",
                "client_id": "client_msg_9b0d807ecc58922f"
            },
            "message": {
                "metadata": {
                    "id": "msg_9b0d807ecc58922f",
                    "message_type": "http_request",
                    "version": "0.1.45",
                    "timestamp": 1752429342302,
                    "priority": "normal",
                    "delivery_mode": "at_least_once",
                    "encoding": "json"
                },
                "payload": {
                    "kind": "HTTP",
                    "data": {
                        "kind": "Request",
                        "method": "POST",
                        "url": "/api/v1/M0X/stk/callback/test",
                        "headers": {
                            "content-type": "application/json"
                        },
                        "body": {
                            "Body": {
                                "stkCallback": {
                                    "MerchantRequestID": "29115-34620561-1",
                                    "CheckoutRequestID": "ws_CO_191220191020363925",
                                    "ResultCode": 0
                                }
                            }
                        },
                        "requestId": "R0X88528F3E97F3"
                    }
                }
            }
        }"#;

        let result = TunnelMessage::from_json(json_with_object_body);
        assert!(
            result.is_ok(),
            "Failed to parse tunnel message with JSON body: {:?}",
            result.err()
        );

        let tunnel_message = result.unwrap();
        if let MessagePayload::Http(HttpPayload::Request { body, .. }) =
            &tunnel_message.message.payload
        {
            assert!(body.is_some(), "Body should not be None");
            let body_bytes = body.as_ref().unwrap();
            let body_str = std::str::from_utf8(body_bytes).expect("Body should be valid UTF-8");
            assert!(
                body_str.contains("stkCallback"),
                "Body should contain the original JSON content"
            );
        } else {
            panic!("Expected HTTP Request payload");
        }
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// WebSocket tunnel messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TunnelMessage {
    /// Authentication message with token
    Auth { token: String },

    /// Successful authentication response
    AuthSuccess { session_id: String },

    /// Authentication error
    AuthError { error: String },

    /// HTTP request from the cloud to forward to a local server
    HttpRequest {
        id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    },

    /// HTTP response from a local server to send back to the cloud
    HttpResponse {
        id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    },

    /// Error response for a request
    Error {
        request_id: Option<String>,
        error: String,
        code: Option<u16>,
    },

    /// Ping message for connection health
    Ping { timestamp: u64 },

    /// Pong response to ping
    Pong { timestamp: u64 },

    /// Statistics message
    Stats {
        requests_processed: u64,
        bytes_transferred: u64,
        uptime_seconds: u64,
    },

    /// Connection status update
    Status {
        status: String,
        message: Option<String>,
    },
}

impl TunnelMessage {
    /// Create an authentication message
    pub fn auth(token: String) -> Self {
        Self::Auth { token }
    }

    /// Create a successful authentication response
    pub fn auth_success(session_id: String) -> Self {
        Self::AuthSuccess { session_id }
    }

    /// Create authentication error
    pub fn auth_error(error: String) -> Self {
        Self::AuthError { error }
    }

    /// Create an HTTP request message
    pub fn http_request(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self::HttpRequest {
            id: Uuid::new_v4().to_string(),
            method,
            url,
            headers,
            body,
        }
    }

    /// Create an HTTP request message with a specific I D
    pub fn http_request_with_id(
        id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self::HttpRequest {
            id,
            method,
            url,
            headers,
            body,
        }
    }

    /// Create an HTTP response message
    pub fn http_response(
        id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self::HttpResponse {
            id,
            status,
            status_text,
            headers,
            body,
        }
    }

    /// Create an error message
    pub fn error(error: String) -> Self {
        Self::Error {
            request_id: None,
            error,
            code: None,
        }
    }

    /// Create an error message for a specific request
    pub fn error_for_request(request_id: String, error: String, code: Option<u16>) -> Self {
        Self::Error {
            request_id: Some(request_id),
            error,
            code,
        }
    }

    /// Create a ping message
    pub fn ping() -> Self {
        Self::Ping {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a pong message
    pub fn pong(timestamp: u64) -> Self {
        Self::Pong { timestamp }
    }

    /// Create a statistics message
    pub fn stats(requests_processed: u64, bytes_transferred: u64, uptime_seconds: u64) -> Self {
        Self::Stats {
            requests_processed,
            bytes_transferred,
            uptime_seconds,
        }
    }

    /// Create a status message
    pub fn status(status: String, message: Option<String>) -> Self {
        Self::Status { status, message }
    }

    /// Serialize a message to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserialize message from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serialize a message to binary (MessagePack or similar)
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        // For now, use JSON as a binary format 
        // In the future, could use MessagePack, CBOR, or Protocol Buffers
        Ok(self.to_json()?.into_bytes())
    }

    /// Deserialize a message from a binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let json = String::from_utf8(data.to_vec())?;
        Self::from_json(&json)
    }

    /// Get message type as string
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::Auth { .. } => "auth",
            Self::AuthSuccess { .. } => "auth_success",
            Self::AuthError { .. } => "auth_error",
            Self::HttpRequest { .. } => "request",
            Self::HttpResponse { .. } => "response",
            Self::Error { .. } => "error",
            Self::Ping { .. } => "ping",
            Self::Pong { .. } => "pong",
            Self::Stats { .. } => "stats",
            Self::Status { .. } => "status",
        }
    }

    /// Get request ID if available
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::HttpRequest { id, .. } => Some(id),
            Self::HttpResponse { id, .. } => Some(id),
            Self::Error { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }

    /// Check if a message contains binary data
    pub fn has_binary_data(&self) -> bool {
        match self {
            Self::HttpRequest { body, .. } => body.is_some(),
            Self::HttpResponse { body, .. } => body.is_some(),
            _ => false,
        }
    }

    /// Get size of message body
    pub fn body_size(&self) -> usize {
        match self {
            Self::HttpRequest { body, .. } => body.as_ref().map(|b| b.len()).unwrap_or(0),
            Self::HttpResponse { body, .. } => body.as_ref().map(|b| b.len()).unwrap_or(0),
            _ => 0,
        }
    }

    /// Check if this is a control message (not HTTP traffic)
    pub fn is_control_message(&self) -> bool {
        matches!(
            self,
            Self::Auth { .. }
                | Self::AuthSuccess { .. }
                | Self::AuthError { .. }
                | Self::Ping { .. }
                | Self::Pong { .. }
                | Self::Stats { .. }
                | Self::Status { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = TunnelMessage::auth("test-token".to_string());
        let json = message.to_json().unwrap();
        let deserialized = TunnelMessage::from_json(&json).unwrap();

        match deserialized {
            TunnelMessage::Auth { token } => assert_eq!(token, "test-token"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_http_request_message() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let message = TunnelMessage::http_request(
            "GET".to_string(),
            "/api/test".to_string(),
            headers,
            Some(b"test body".to_vec()),
        );

        assert_eq!(message.message_type(), "request");
        assert!(message.has_binary_data());
        assert_eq!(message.body_size(), 9);
        assert!(!message.is_control_message());
    }

    #[test]
    fn test_ping_pong_messages() {
        let ping = TunnelMessage::ping();
        assert_eq!(ping.message_type(), "ping");
        assert!(ping.is_control_message());

        if let TunnelMessage::Ping { timestamp } = ping {
            let pong = TunnelMessage::pong(timestamp);
            assert_eq!(pong.message_type(), "pong");
            assert!(pong.is_control_message());
        }
    }
}

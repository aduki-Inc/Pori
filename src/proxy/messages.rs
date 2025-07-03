use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Messages for proxy communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyMessage {
    /// HTTP request to forward to a local server
    HttpRequest {
        id: String,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    },

    /// HTTP response from a local server
    HttpResponse {
        id: String,
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    },

    /// Error during request processing
    Error {
        id: String,
        error: String,
        status_code: Option<u16>,
    },

    /// Request statistics
    Stats {
        requests_processed: u64,
        requests_successful: u64,
        requests_failed: u64,
        bytes_forwarded: u64,
    },
}

impl ProxyMessage {
    /// Create an HTTP request message
    pub fn http_request(
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
    pub fn error(id: String, error: String, status_code: Option<u16>) -> Self {
        Self::Error {
            id,
            error,
            status_code,
        }
    }

    /// Create a stat message
    pub fn stats(
        requests_processed: u64,
        requests_successful: u64,
        requests_failed: u64,
        bytes_forwarded: u64,
    ) -> Self {
        Self::Stats {
            requests_processed,
            requests_successful,
            requests_failed,
            bytes_forwarded,
        }
    }

    /// Get request/response ID
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::HttpRequest { id, .. } => Some(id),
            Self::HttpResponse { id, .. } => Some(id),
            Self::Error { id, .. } => Some(id),
            Self::Stats { .. } => None,
        }
    }

    /// Get message type
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::HttpRequest { .. } => "request",
            Self::HttpResponse { .. } => "response",
            Self::Error { .. } => "error",
            Self::Stats { .. } => "stats",
        }
    }

    /// Check if a message has body data
    pub fn has_body(&self) -> bool {
        match self {
            Self::HttpRequest { body, .. } => body.is_some(),
            Self::HttpResponse { body, .. } => body.is_some(),
            _ => false,
        }
    }

    /// Get body size
    pub fn body_size(&self) -> usize {
        match self {
            Self::HttpRequest { body, .. } => body.as_ref().map(|b| b.len()).unwrap_or(0),
            Self::HttpResponse { body, .. } => body.as_ref().map(|b| b.len()).unwrap_or(0),
            _ => 0,
        }
    }
}

use thiserror::Error;

/// Application-specific error types
#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("WebSocket connection error: {0}")]
    WebSocketConnection(String),

    #[error("WebSocket authentication failed: {0}")]
    WebSocketAuth(String),

    #[error("HTTP proxy error: {0}")]
    HttpProxy(String),

    #[error("Local server error: {0}")]
    LocalServer(String),

    #[error("Dashboard server error: {0}")]
    Dashboard(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Message parsing error: {0}")]
    MessageParsing(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
}

impl TunnelError {
    /// Check if the error is recoverable (should retry)
    pub fn is_recoverable(&self) -> bool {
        match self {
            TunnelError::WebSocketConnection(_) => true,
            TunnelError::HttpProxy(_) => true,
            TunnelError::LocalServer(_) => true,
            TunnelError::Timeout(_) => true,
            TunnelError::Io(_) => true,
            TunnelError::Request(_) => true,
            TunnelError::WebSocket(_) => true,
            TunnelError::WebSocketAuth(_) => false,
            TunnelError::Configuration(_) => false,
            TunnelError::MessageParsing(_) => false,
            TunnelError::Dashboard(_) => false,
            TunnelError::UrlParsing(_) => false,
            TunnelError::JsonSerialization(_) => false,
            TunnelError::Http(_) => false,
        }
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            TunnelError::WebSocketConnection(_) | TunnelError::WebSocket(_) => "websocket",
            TunnelError::WebSocketAuth(_) => "authentication",
            TunnelError::HttpProxy(_) => "proxy",
            TunnelError::LocalServer(_) | TunnelError::Request(_) => "local_server",
            TunnelError::Dashboard(_) | TunnelError::Http(_) => "dashboard",
            TunnelError::Configuration(_) => "configuration",
            TunnelError::MessageParsing(_) | TunnelError::JsonSerialization(_) => "serialization",
            TunnelError::Timeout(_) => "timeout",
            TunnelError::Io(_) => "io",
            TunnelError::UrlParsing(_) => "url",
        }
    }
}

/// Convert anyhow::Error to TunnelError
impl From<anyhow::Error> for TunnelError {
    fn from(err: anyhow::Error) -> Self {
        TunnelError::Configuration(err.to_string())
    }
}

/// Result type alias for convenience
pub type TunnelResult<T> = Result<T, TunnelError>;

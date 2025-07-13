//! Proxy message module - now using unified protocol messages

// Re-export protocol message types
pub use crate::protocol::{
    messages::*,
    http::*,
};

// Type aliases for commonly used types
pub type ProxyMessage = HttpMessage;

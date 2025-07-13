//! Protocol module for unified message structures
//!
//! This module defines all message structures used throughout the proxy system
//! for communication between components and external systems.

pub mod config;
pub mod http;
pub mod messages;
pub mod tunnel;
pub mod websocket;

pub use config::{
    CompressionConfig, FeatureConfig, LimitConfig, ProtocolConfig, SecurityConfig, TimeoutConfig,
};
pub use http::{HttpEnvelope, HttpMessage, HttpRequestConfig};
pub use messages::{
    AuthPayload, ControlPayload, CustomPayload, ErrorPayload, HttpPayload, MessageMetadata,
    MessagePayload, ProtocolMessage, StatsPayload, StreamPayload,
};
pub use tunnel::{TunnelConfig, TunnelEnvelope, TunnelMessage, TunnelSettings};
pub use websocket::{WebSocketConfig as WsConfig, WebSocketEnvelope, WebSocketMessage};

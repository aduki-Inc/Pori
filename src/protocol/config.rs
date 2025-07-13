use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for protocol message handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub version: String,
    pub compression: CompressionConfig,
    pub security: SecurityConfig,
    pub timeouts: TimeoutConfig,
    pub limits: LimitConfig,
    pub features: FeatureConfig,
}

/// Compression configuration for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: CompressionAlgorithm,
    pub level: u8,
    pub min_size_threshold: usize,
}

/// Security configuration for message handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub signature_validation: bool,
    pub allowed_origins: Vec<String>,
    pub rate_limiting: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub block_duration_seconds: u64,
}

/// Timeout configuration for various operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    #[serde(with = "duration_serde")]
    pub connection_timeout: Duration,
    #[serde(with = "duration_serde")]
    pub request_timeout: Duration,
    #[serde(with = "duration_serde")]
    pub response_timeout: Duration,
    #[serde(with = "duration_serde")]
    pub ping_timeout: Duration,
    #[serde(with = "duration_serde")]
    pub auth_timeout: Duration,
}

/// Limits for message sizes and counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
    pub max_message_size: usize,
    pub max_header_size: usize,
    pub max_body_size: usize,
    pub max_concurrent_requests: usize,
    pub max_queue_size: usize,
}

/// Feature toggles for protocol capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub streaming_enabled: bool,
    pub websocket_compression: bool,
    pub http2_enabled: bool,
    pub keep_alive_enabled: bool,
    pub metrics_collection: bool,
}

/// Compression algorithms supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Deflate,
    Brotli,
    None,
}

/// Message encoding format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageEncoding {
    Json,
    MessagePack,
    Protobuf,
    Cbor,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Message delivery mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryMode {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            compression: CompressionConfig::default(),
            security: SecurityConfig::default(),
            timeouts: TimeoutConfig::default(),
            limits: LimitConfig::default(),
            features: FeatureConfig::default(),
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Gzip,
            level: 6,
            min_size_threshold: 1024,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: false,
            signature_validation: false,
            allowed_origins: vec!["*".to_string()],
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: 100,
            burst_size: 10,
            block_duration_seconds: 300,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            response_timeout: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            auth_timeout: Duration::from_secs(15),
        }
    }
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            max_message_size: 16 * 1024 * 1024, // 16MB
            max_header_size: 8 * 1024,          // 8KB
            max_body_size: 16 * 1024 * 1024,    // 16MB
            max_concurrent_requests: 100,
            max_queue_size: 1000,
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            streaming_enabled: true,
            websocket_compression: true,
            http2_enabled: false,
            keep_alive_enabled: true,
            metrics_collection: true,
        }
    }
}

/// Serde helper for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_config_default() {
        let config = ProtocolConfig::default();
        assert_eq!(config.version, "1.0.0");
        assert!(config.compression.enabled);
        assert_eq!(config.limits.max_message_size, 16 * 1024 * 1024);
    }

    #[test]
    fn test_serialization() {
        let config = ProtocolConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ProtocolConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.version, deserialized.version);
    }
}

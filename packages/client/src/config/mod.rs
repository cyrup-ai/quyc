//! Comprehensive Configuration Architecture
//!
//! Zero-allocation configuration system with compile-time optimization and runtime flexibility.
//! Supports both static (compile-time) and dynamic (runtime) configuration patterns.

use std::time::Duration;



pub mod network;
pub mod timeouts;
pub mod buffers;
pub mod protocol;
pub mod retry;
pub mod performance;
pub mod security;
pub mod client;
pub mod validation;

// Core configuration utilities (existing)
pub mod core;

// Re-export all configuration types for easy access
pub use network::{NetworkConfig, NetworkConfigProvider, StaticNetworkConfig};
pub use timeouts::{TimeoutConfig, TimeoutConfigProvider, StaticTimeoutConfig};
pub use buffers::{BufferConfig, BufferConfigProvider, StaticBufferConfig};
pub use protocol::{ProtocolConfig, ProtocolConfigProvider, StaticProtocolConfig};
pub use retry::{RetryConfig, RetryConfigProvider, StaticRetryConfig};
pub use performance::{PerformanceConfig, PerformanceConfigProvider, StaticPerformanceConfig};
pub use security::{SecurityConfig, SecurityConfigProvider, StaticSecurityConfig, TlsVersion};
pub use client::ClientConfig;
pub use validation::{ConfigValidator, ConfigDefaults};


/// TCP connection configuration
#[derive(Debug, Clone)]
pub struct TcpConfig {
    pub nodelay: bool,
    pub keepalive: Option<Duration>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            nodelay: true,
            keepalive: Some(Duration::from_secs(60)),
        }
    }
}

/// TLS connection configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub use_native_certs: bool,
    pub early_data: bool,
    pub https_only: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            use_native_certs: true,
            early_data: false,
            https_only: false,
        }
    }
}

/// Compression algorithm configuration
#[derive(Debug, Clone)]
pub struct CompressionAlgorithm {
    pub enabled: bool,
    pub level: Option<u32>,
}

impl CompressionAlgorithm {
    #[must_use]
    pub fn new(enabled: bool, level: Option<u32>) -> Self {
        Self { enabled, level }
    }
    
    #[must_use]
    pub fn enabled_with_level(level: u32) -> Self {
        Self {
            enabled: true,
            level: Some(level),
        }
    }
    
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            level: None,
        }
    }
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::enabled_with_level(6)
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub request_compression: bool,
    pub response_compression: bool,
    pub gzip: CompressionAlgorithm,
    pub brotli: CompressionAlgorithm,
    pub deflate: CompressionAlgorithm,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            request_compression: true,
            response_compression: true,
            gzip: CompressionAlgorithm::default(),
            brotli: CompressionAlgorithm::default(),
            deflate: CompressionAlgorithm::default(),
        }
    }
}

/// HTTP-specific configuration for client behavior
#[derive(Debug, Clone)]
pub struct HttpConfig {
    // Request/Response timeouts
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub dns_cache_duration: Duration,
    
    // Connection pool settings
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub pool_size: usize,
    
    // Structured configuration objects
    pub tcp: TcpConfig,
    pub tls: TlsConfig,
    pub compression: CompressionConfig,
    
    // HTTP/2 settings
    pub http2_keep_alive_interval: Option<Duration>,
    pub http2_keep_alive_timeout: Option<Duration>,
    
    // QUIC settings
    pub quic_max_idle_timeout: Option<Duration>,
    pub quic_stream_receive_window: Option<u32>,
    pub quic_receive_window: Option<u32>,
    pub quic_send_window: Option<u32>,
    
    // User agent
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            // Timeouts
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            dns_cache_duration: Duration::from_secs(300),
            
            // Connection pool
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
            pool_size: 100,
            
            // Structured configuration objects
            tcp: TcpConfig::default(),
            tls: TlsConfig::default(),
            compression: CompressionConfig::default(),
            
            // HTTP/2
            http2_keep_alive_interval: Some(Duration::from_secs(30)),
            http2_keep_alive_timeout: Some(Duration::from_secs(10)),
            
            // QUIC
            quic_max_idle_timeout: Some(Duration::from_secs(30)),
            quic_stream_receive_window: Some(65536),
            quic_receive_window: Some(1_048_576),
            quic_send_window: Some(1_048_576),
            
            // User agent
            user_agent: "quyc-http3-client/0.1.0".to_string(),
        }
    }
}



/// Master configuration provider trait for compile-time optimization
pub trait ConfigurationProvider {
    type NetworkConfig: NetworkConfigProvider;
    type TimeoutConfig: TimeoutConfigProvider;
    type BufferConfig: BufferConfigProvider;
    type ProtocolConfig: ProtocolConfigProvider;
    type RetryConfig: RetryConfigProvider;
    type PerformanceConfig: PerformanceConfigProvider;
    type SecurityConfig: SecurityConfigProvider;
    
    fn network(&self) -> &Self::NetworkConfig;
    fn timeouts(&self) -> &Self::TimeoutConfig;
    fn buffers(&self) -> &Self::BufferConfig;
    fn protocol(&self) -> &Self::ProtocolConfig;
    fn retry(&self) -> &Self::RetryConfig;
    fn performance(&self) -> &Self::PerformanceConfig;
    fn security(&self) -> &Self::SecurityConfig;
}

/// Default production configuration
#[derive(Debug, Clone, Default)]
pub struct DefaultConfiguration {
    pub network: NetworkConfig,
    pub timeouts: TimeoutConfig,
    pub buffers: BufferConfig,
    pub protocol: ProtocolConfig,
    pub retry: RetryConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
}

impl ConfigurationProvider for DefaultConfiguration {
    type NetworkConfig = NetworkConfig;
    type TimeoutConfig = TimeoutConfig;
    type BufferConfig = BufferConfig;
    type ProtocolConfig = ProtocolConfig;
    type RetryConfig = RetryConfig;
    type PerformanceConfig = PerformanceConfig;
    type SecurityConfig = SecurityConfig;
    
    #[inline]
    fn network(&self) -> &Self::NetworkConfig {
        &self.network
    }
    
    #[inline]
    fn timeouts(&self) -> &Self::TimeoutConfig {
        &self.timeouts
    }
    
    #[inline]
    fn buffers(&self) -> &Self::BufferConfig {
        &self.buffers
    }
    
    #[inline]
    fn protocol(&self) -> &Self::ProtocolConfig {
        &self.protocol
    }
    
    #[inline]
    fn retry(&self) -> &Self::RetryConfig {
        &self.retry
    }
    
    #[inline]
    fn performance(&self) -> &Self::PerformanceConfig {
        &self.performance
    }
    
    #[inline]
    fn security(&self) -> &Self::SecurityConfig {
        &self.security
    }
}

/// Compile-time configuration for zero-allocation access
pub struct StaticConfiguration<
    const MAX_CONNECTIONS: usize = 100,
    const BUFFER_SIZE: usize = 8192,
    const MAX_RETRIES: u32 = 3,
>;

impl<
    const MAX_CONNECTIONS: usize,
    const BUFFER_SIZE: usize,
    const MAX_RETRIES: u32,
> ConfigurationProvider for StaticConfiguration<MAX_CONNECTIONS, BUFFER_SIZE, MAX_RETRIES> {
    type NetworkConfig = StaticNetworkConfig<MAX_CONNECTIONS>;
    type TimeoutConfig = StaticTimeoutConfig<30000, 10000>;
    type BufferConfig = StaticBufferConfig<BUFFER_SIZE>;
    type ProtocolConfig = StaticProtocolConfig;
    type RetryConfig = StaticRetryConfig<MAX_RETRIES>;
    type PerformanceConfig = StaticPerformanceConfig;
    type SecurityConfig = StaticSecurityConfig;
    
    #[inline]
    fn network(&self) -> &Self::NetworkConfig {
        &StaticNetworkConfig::<MAX_CONNECTIONS>
    }
    
    #[inline]
    fn timeouts(&self) -> &Self::TimeoutConfig {
        &StaticTimeoutConfig::<30000, 10000>
    }
    
    #[inline]
    fn buffers(&self) -> &Self::BufferConfig {
        &StaticBufferConfig::<BUFFER_SIZE>
    }
    
    #[inline]
    fn protocol(&self) -> &Self::ProtocolConfig {
        &StaticProtocolConfig
    }
    
    #[inline]
    fn retry(&self) -> &Self::RetryConfig {
        &StaticRetryConfig::<MAX_RETRIES>
    }
    
    #[inline]
    fn performance(&self) -> &Self::PerformanceConfig {
        &StaticPerformanceConfig
    }
    
    #[inline]
    fn security(&self) -> &Self::SecurityConfig {
        &StaticSecurityConfig
    }
}

/// Configuration validation and error handling
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Invalid network configuration: {0}")]
    Network(String),
    #[error("Invalid timeout configuration: {0}")]
    Timeout(String),
    #[error("Configuration validation failed: {0}")]
    Validation(String),
}

/// Configuration validation trait
pub trait ConfigurationValidator {
    /// Validates the configuration for correctness and consistency
    /// 
    /// # Errors
    /// 
    /// Returns a `ConfigurationError` if validation fails:
    /// - `Network` - if network configuration is invalid (ports, addresses, etc.)
    /// - `Timeout` - if timeout values are invalid (zero, negative, or excessive)
    /// - `Validation` - if general validation constraints are violated
    fn validate(&self) -> Result<(), ConfigurationError>;
}

impl ConfigurationValidator for DefaultConfiguration {
    fn validate(&self) -> Result<(), ConfigurationError> {
        self.network.validate().map_err(ConfigurationError::Network)?;
        Ok(())
    }
}
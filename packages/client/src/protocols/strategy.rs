//! HTTP protocol strategy pattern implementation
//!
//! Provides strategy enumeration for protocol selection with automatic fallback
//! and protocol-specific configuration management.

use std::time::Duration;
// task imports removed - not used

// prelude import removed - not used
// http imports removed - not used
// futures noop_waker import removed - not used

use crate::config::HttpConfig;
use crate::protocols::core::{HttpVersion, ProtocolConfig, TimeoutConfig};
// connection import removed - not used
// transport imports removed - not used
// http types removed - not used
// http response types removed - not used

/// Protocol selection strategy with fallback support
#[derive(Debug, Clone)]
pub enum HttpProtocolStrategy {
    /// Force HTTP/2 with specific configuration
    Http2(H2Config),
    /// Force HTTP/3 with specific configuration  
    Http3(H3Config),
    /// Force QUIC with Quiche implementation
    Quiche(QuicheConfig),
    /// Automatic selection with preference ordering
    Auto {
        prefer: Vec<HttpVersion>,
        fallback_chain: Vec<HttpVersion>,
        configs: ProtocolConfigs,
    },
}

impl Default for HttpProtocolStrategy {
    fn default() -> Self {
        Self::Auto {
            prefer: vec![HttpVersion::Http3, HttpVersion::Http2],
            fallback_chain: vec![HttpVersion::Http3, HttpVersion::Http2],
            configs: ProtocolConfigs::default(),
        }
    }
}

impl HttpProtocolStrategy {
    /// Build the appropriate ProtocolStrategy implementation
    pub fn build(&self) -> Box<dyn crate::protocols::strategy_trait::ProtocolStrategy> {
        use crate::protocols::h2::strategy::H2Strategy;
        use crate::protocols::h3::strategy::H3Strategy;
        use crate::protocols::auto_strategy::AutoStrategy;
        
        match self {
            Self::Http2(config) => Box::new(H2Strategy::new(config.clone())),
            Self::Http3(config) => Box::new(H3Strategy::new(config.clone())),
            Self::Quiche(config) => {
                // Quiche is just H3 with specific config
                Box::new(H3Strategy::new(H3Config {
                    max_idle_timeout: config.max_idle_timeout,
                    max_udp_payload_size: config.max_udp_payload_size,
                    initial_max_data: config.initial_max_data,
                    initial_max_stream_data_bidi_local: config.initial_max_stream_data_bidi_local,
                    initial_max_stream_data_bidi_remote: config.initial_max_stream_data_bidi_remote,
                    initial_max_stream_data_uni: config.initial_max_stream_data_uni,
                    initial_max_streams_bidi: config.initial_max_streams_bidi,
                    initial_max_streams_uni: config.initial_max_streams_uni,
                    enable_early_data: config.enable_early_data,
                    enable_0rtt: config.enable_early_data,
                    congestion_control: config.congestion_control,
                }))
            },
            Self::Auto { prefer, fallback_chain: _, configs } => {
                Box::new(AutoStrategy::new(prefer.clone(), configs.clone()))
            },
        }
    }
    
    /// Create AI-optimized strategy for streaming workloads
    pub fn ai_optimized() -> Self {
        Self::Auto {
            prefer: vec![HttpVersion::Http3],
            fallback_chain: vec![HttpVersion::Http3, HttpVersion::Http2],
            configs: ProtocolConfigs {
                h2: H2Config::ai_optimized(),
                h3: H3Config::ai_optimized(),
                quiche: QuicheConfig::ai_optimized(),
            },
        }
    }

    /// Create streaming-optimized strategy for real-time data
    pub fn streaming_optimized() -> Self {
        Self::Http3(H3Config::streaming_optimized())
    }

    /// Create low-latency strategy for interactive applications
    pub fn low_latency() -> Self {
        Self::Quiche(QuicheConfig::low_latency())
    }



}

/// Configuration bundle for all protocols
#[derive(Debug, Clone)]
pub struct ProtocolConfigs {
    pub h2: H2Config,
    pub h3: H3Config,
    pub quiche: QuicheConfig,
}

impl Default for ProtocolConfigs {
    fn default() -> Self {
        Self {
            h2: H2Config::default(),
            h3: H3Config::default(),
            quiche: QuicheConfig::default(),
        }
    }
}

/// Strategy-specific protocol configuration
#[derive(Debug, Clone)]
pub enum StrategyProtocolConfig {
    H2(H2Config),
    H3(H3Config),
    Quiche(QuicheConfig),
}

/// HTTP/2 protocol configuration
#[derive(Debug, Clone)]
pub struct H2Config {
    pub max_concurrent_streams: u32,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub enable_push: bool,
    pub enable_connect_protocol: bool,
    pub keepalive_interval: Option<Duration>,
    pub keepalive_timeout: Duration,
    pub adaptive_window: bool,
    pub max_send_buffer_size: usize,
}

impl Default for H2Config {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            enable_push: false,
            enable_connect_protocol: true,
            keepalive_interval: Some(Duration::from_secs(30)),
            keepalive_timeout: Duration::from_secs(10),
            adaptive_window: true,
            max_send_buffer_size: 1024 * 1024,
        }
    }
}

impl H2Config {
    pub fn ai_optimized() -> Self {
        Self {
            max_concurrent_streams: 1000,
            initial_window_size: 1048576, // 1MB
            max_frame_size: 32768,
            enable_push: false,
            enable_connect_protocol: true,
            keepalive_interval: Some(Duration::from_secs(15)),
            keepalive_timeout: Duration::from_secs(5),
            adaptive_window: true,
            max_send_buffer_size: 4 * 1024 * 1024, // 4MB
        }
    }
}

impl ProtocolConfig for H2Config {
    fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_streams == 0 {
            return Err("max_concurrent_streams must be greater than 0".to_string());
        }
        if self.initial_window_size < 65535 {
            return Err("initial_window_size must be at least 65535".to_string());
        }
        if self.max_frame_size < 16384 || self.max_frame_size > 16777215 {
            return Err("max_frame_size must be between 16384 and 16777215".to_string());
        }
        Ok(())
    }

    fn timeout_config(&self) -> TimeoutConfig {
        TimeoutConfig {
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            keepalive_timeout: Some(self.keepalive_timeout),
        }
    }

    fn to_http_config(&self) -> HttpConfig {
        HttpConfig::default()
    }
}

/// HTTP/3 protocol configuration
#[derive(Debug, Clone)]
pub struct H3Config {
    pub max_idle_timeout: Duration,
    pub max_udp_payload_size: u16,
    pub initial_max_data: u64,
    pub initial_max_stream_data_bidi_local: u64,
    pub initial_max_stream_data_bidi_remote: u64,
    pub initial_max_stream_data_uni: u64,
    pub initial_max_streams_bidi: u64,
    pub initial_max_streams_uni: u64,
    pub enable_early_data: bool,
    pub enable_0rtt: bool,
    pub congestion_control: CongestionControl,
}

impl Default for H3Config {
    fn default() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(30),
            max_udp_payload_size: 1452,
            initial_max_data: 10485760,                   // 10MB
            initial_max_stream_data_bidi_local: 1048576,  // 1MB
            initial_max_stream_data_bidi_remote: 1048576, // 1MB
            initial_max_stream_data_uni: 1048576,         // 1MB
            initial_max_streams_bidi: 100,
            initial_max_streams_uni: 100,
            enable_early_data: true,
            enable_0rtt: true,
            congestion_control: CongestionControl::Cubic,
        }
    }
}

impl H3Config {
    pub fn ai_optimized() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(60),
            max_udp_payload_size: 1452,
            initial_max_data: 104857600,                   // 100MB
            initial_max_stream_data_bidi_local: 10485760,  // 10MB
            initial_max_stream_data_bidi_remote: 10485760, // 10MB
            initial_max_stream_data_uni: 10485760,         // 10MB
            initial_max_streams_bidi: 1000,
            initial_max_streams_uni: 1000,
            enable_early_data: true,
            enable_0rtt: true,
            congestion_control: CongestionControl::Bbr,
        }
    }

    pub fn streaming_optimized() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(300),
            max_udp_payload_size: 1452,
            initial_max_data: 1073741824,                   // 1GB
            initial_max_stream_data_bidi_local: 104857600,  // 100MB
            initial_max_stream_data_bidi_remote: 104857600, // 100MB
            initial_max_stream_data_uni: 104857600,         // 100MB
            initial_max_streams_bidi: 10000,
            initial_max_streams_uni: 10000,
            enable_early_data: true,
            enable_0rtt: true,
            congestion_control: CongestionControl::Bbr,
        }
    }


}

impl ProtocolConfig for H3Config {
    fn validate(&self) -> Result<(), String> {
        if self.max_idle_timeout.as_secs() == 0 {
            return Err("max_idle_timeout must be greater than 0".to_string());
        }
        if self.max_udp_payload_size < 1200 {
            return Err("max_udp_payload_size must be at least 1200".to_string());
        }
        if self.initial_max_data == 0 {
            return Err("initial_max_data must be greater than 0".to_string());
        }
        Ok(())
    }

    fn timeout_config(&self) -> TimeoutConfig {
        TimeoutConfig {
            request_timeout: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(5),
            idle_timeout: self.max_idle_timeout,
            keepalive_timeout: Some(self.max_idle_timeout / 2),
        }
    }

    fn to_http_config(&self) -> HttpConfig {
        HttpConfig::default()
    }
}

/// Quiche QUIC configuration
#[derive(Debug, Clone)]
pub struct QuicheConfig {
    pub max_idle_timeout: Duration,
    pub initial_max_data: u64,
    pub initial_max_stream_data_bidi_local: u64,
    pub initial_max_stream_data_bidi_remote: u64,
    pub initial_max_stream_data_uni: u64,
    pub initial_max_streams_bidi: u64,
    pub initial_max_streams_uni: u64,
    pub max_udp_payload_size: u16,
    pub enable_early_data: bool,
    pub enable_hystart: bool,
    pub congestion_control: CongestionControl,
    pub max_connection_window: u64,
    pub max_stream_window: u64,
}

impl Default for QuicheConfig {
    fn default() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(30),
            initial_max_data: 10485760,                   // 10MB
            initial_max_stream_data_bidi_local: 1048576,  // 1MB
            initial_max_stream_data_bidi_remote: 1048576, // 1MB
            initial_max_stream_data_uni: 1048576,         // 1MB
            initial_max_streams_bidi: 100,
            initial_max_streams_uni: 100,
            max_udp_payload_size: 1452,
            enable_early_data: true,
            enable_hystart: true,
            congestion_control: CongestionControl::Cubic,
            max_connection_window: 25165824, // 24MB
            max_stream_window: 16777216,     // 16MB
        }
    }
}

impl QuicheConfig {
    pub fn ai_optimized() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(60),
            initial_max_data: 104857600,                   // 100MB
            initial_max_stream_data_bidi_local: 10485760,  // 10MB
            initial_max_stream_data_bidi_remote: 10485760, // 10MB
            initial_max_stream_data_uni: 10485760,         // 10MB
            initial_max_streams_bidi: 1000,
            initial_max_streams_uni: 1000,
            max_udp_payload_size: 1452,
            enable_early_data: true,
            enable_hystart: true,
            congestion_control: CongestionControl::Bbr,
            max_connection_window: 268435456, // 256MB
            max_stream_window: 134217728,     // 128MB
        }
    }

    pub fn low_latency() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(15),
            initial_max_data: 52428800,                   // 50MB
            initial_max_stream_data_bidi_local: 5242880,  // 5MB
            initial_max_stream_data_bidi_remote: 5242880, // 5MB
            initial_max_stream_data_uni: 5242880,         // 5MB
            initial_max_streams_bidi: 500,
            initial_max_streams_uni: 500,
            max_udp_payload_size: 1200, // Conservative for low latency
            enable_early_data: true,
            enable_hystart: false, // Disable for predictable latency
            congestion_control: CongestionControl::Bbr,
            max_connection_window: 67108864, // 64MB
            max_stream_window: 33554432,     // 32MB
        }
    }
}

impl ProtocolConfig for QuicheConfig {
    fn validate(&self) -> Result<(), String> {
        if self.max_idle_timeout.as_secs() == 0 {
            return Err("max_idle_timeout must be greater than 0".to_string());
        }
        if self.initial_max_data == 0 {
            return Err("initial_max_data must be greater than 0".to_string());
        }
        if self.max_udp_payload_size < 1200 {
            return Err("max_udp_payload_size must be at least 1200".to_string());
        }
        Ok(())
    }

    fn timeout_config(&self) -> TimeoutConfig {
        TimeoutConfig {
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            idle_timeout: self.max_idle_timeout,
            keepalive_timeout: Some(self.max_idle_timeout / 3),
        }
    }

    fn to_http_config(&self) -> HttpConfig {
        HttpConfig::default()
    }
}

/// Congestion control algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CongestionControl {
    Reno,
    Cubic,
    Bbr,
    BbrV2,
}

impl Default for CongestionControl {
    fn default() -> Self {
        Self::Cubic
    }
}

/// Convert H3Config to quiche::Config for HTTP/3 connections
fn convert_h3_config_to_quiche(h3_config: &H3Config) -> Result<quiche::Config, String> {
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)
        .map_err(|e| format!("Failed to create quiche config: {}", e))?;

    // Set transport parameters from H3Config
    config.set_initial_max_data(h3_config.initial_max_data);
    config.set_initial_max_streams_bidi(h3_config.initial_max_streams_bidi);
    config.set_initial_max_streams_uni(h3_config.initial_max_streams_uni);
    config.set_initial_max_stream_data_bidi_local(h3_config.initial_max_stream_data_bidi_local);
    config.set_initial_max_stream_data_bidi_remote(h3_config.initial_max_stream_data_bidi_remote);
    config.set_initial_max_stream_data_uni(h3_config.initial_max_stream_data_uni);
    
    // Set idle timeout
    config.set_max_idle_timeout(h3_config.max_idle_timeout.as_millis() as u64);
    
    // Set UDP payload size
    config.set_max_recv_udp_payload_size(h3_config.max_udp_payload_size as usize);
    config.set_max_send_udp_payload_size(h3_config.max_udp_payload_size as usize);

    // Enable early data and 0-RTT if requested
    config.enable_early_data();
    if h3_config.enable_0rtt {
        // 0-RTT is enabled by default in quiche when early data is enabled
    }

    // Set congestion control algorithm
    match h3_config.congestion_control {
        CongestionControl::Cubic => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::CUBIC),
        CongestionControl::Reno => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::Reno),
        CongestionControl::Bbr => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR),
        CongestionControl::BbrV2 => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2),
    }

    // Set HTTP/3 application protocol
    config.set_application_protos(&[b"h3"]).map_err(|e| format!("Failed to set application protos: {}", e))?;

    Ok(config)
}

/// Convert QuicheConfig to quiche::Config for HTTP/3 connections  
fn convert_quiche_config_to_quiche(quiche_config: &QuicheConfig) -> Result<quiche::Config, String> {
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)
        .map_err(|e| format!("Failed to create quiche config: {}", e))?;

    // Set transport parameters from QuicheConfig
    config.set_initial_max_data(quiche_config.initial_max_data);
    config.set_initial_max_streams_bidi(quiche_config.initial_max_streams_bidi);
    config.set_initial_max_streams_uni(quiche_config.initial_max_streams_uni);
    config.set_initial_max_stream_data_bidi_local(quiche_config.initial_max_stream_data_bidi_local);
    config.set_initial_max_stream_data_bidi_remote(quiche_config.initial_max_stream_data_bidi_remote);
    config.set_initial_max_stream_data_uni(quiche_config.initial_max_stream_data_uni);
    
    // Set idle timeout
    config.set_max_idle_timeout(quiche_config.max_idle_timeout.as_millis() as u64);
    
    // Set UDP payload size
    config.set_max_recv_udp_payload_size(quiche_config.max_udp_payload_size as usize);
    config.set_max_send_udp_payload_size(quiche_config.max_udp_payload_size as usize);

    // Enable early data and 0-RTT if requested
    if quiche_config.enable_early_data {
        config.enable_early_data();
    }

    // Set congestion control algorithm
    match quiche_config.congestion_control {
        CongestionControl::Cubic => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::CUBIC),
        CongestionControl::Reno => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::Reno),
        CongestionControl::Bbr => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR),
        CongestionControl::BbrV2 => config.set_cc_algorithm(quiche::CongestionControlAlgorithm::BBR2),
    }

    // Set HTTP/3 application protocol
    config.set_application_protos(&[b"h3"]).map_err(|e| format!("Failed to set application protos: {}", e))?;

    Ok(config)
}

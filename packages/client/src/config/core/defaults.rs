//! Default implementations for HTTP configuration
//!
//! Provides comprehensive default values for `HttpConfig` with optimized
//! settings for HTTP/3, connection pooling, and retry behavior.

use std::time::Duration;

use super::retry::{ConnectionReuse, RetryPolicy};
use super::types::HttpConfig;
use crate::http::conversions::SecurityMode;

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: 32,
            pool_idle_timeout: Duration::from_secs(90),
            timeout: Duration::from_secs(86400),
            connect_timeout: Duration::from_secs(10),
            tcp_keepalive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
            http2_adaptive_window: true,
            gzip_enabled: true,
            brotli_enabled: true,
            http2_max_frame_size: Some(1 << 20), // 1MB
            use_native_certs: true,
            https_only: false,
            deflate: true,
            user_agent: "fluent-ai-http3/0.1.0 (QUIC/HTTP3+rustls)".to_string(),
            http3_enabled: true,
            pool_size: 10,
            max_redirects: 10,
            cookie_store: false,
            response_compression: true,
            request_compression: true,
            dns_cache_duration: Duration::from_secs(300),
            dns_over_https: false,
            happy_eyeballs: true,
            local_address: None,
            interface: None,
            http2_server_push: false,
            http2_initial_stream_window_size: None,
            http2_initial_connection_window_size: None,
            http2_max_concurrent_streams: None,
            http2_keep_alive: true,
            http2_keep_alive_interval: Some(Duration::from_secs(30)),
            http2_keep_alive_timeout: Some(Duration::from_secs(5)),
            http2_adaptive_window_scaling: true,
            trust_dns: false,
            metrics_enabled: true,
            tracing_enabled: false,
            connection_reuse: ConnectionReuse::Aggressive,
            retry_policy: RetryPolicy::default(),
            
            // UTF-8 validation security - strict by default for production safety
            utf8_validation_mode: SecurityMode::Strict,

            // HTTP/3 (QUIC) defaults - conservative but optimized values
            quic_max_idle_timeout: Some(Duration::from_secs(30)),
            quic_stream_receive_window: Some(256 * 1024), // 256KB per stream
            quic_receive_window: Some(1024 * 1024),       // 1MB connection window
            quic_send_window: Some(512 * 1024),           // 512KB send window
            quic_congestion_bbr: false,                   // Use CUBIC by default for compatibility
            tls_early_data: false,                        // Disabled by default for security
            h3_max_field_section_size: Some(16 * 1024),   // 16KB header limit
            h3_enable_grease: true,                       // Enable grease for protocol evolution

            // Compression level defaults - None uses library optimal defaults
            gzip_level: None,     // Use flate2 default (level 6)
            brotli_level: None,   // Use brotli default (level 6)
            deflate_level: None,  // Use flate2 default (level 6)
        }
    }
}

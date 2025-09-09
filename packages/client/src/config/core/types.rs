//! Core HTTP configuration structure and field definitions
//!
//! Contains the main HttpConfig struct with all configuration fields
//! for HTTP client behavior, connection management, and protocol settings.

use std::time::Duration;

use super::retry::{ConnectionReuse, RetryPolicy};

/// HTTP client configuration
///
/// Central configuration struct containing all HTTP client settings including
/// connection pools, timeouts, HTTP/2 and HTTP/3 parameters, and security options.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Maximum number of idle connections per host
    pub pool_max_idle_per_host: usize,

    /// Pool idle timeout
    pub pool_idle_timeout: Duration,

    /// Request timeout
    pub timeout: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// TCP keep-alive duration
    pub tcp_keepalive: Option<Duration>,

    /// Enable TCP_NODELAY
    pub tcp_nodelay: bool,

    /// Enable HTTP/2 adaptive window
    pub http2_adaptive_window: bool,

    /// HTTP/2 max frame size
    pub http2_max_frame_size: Option<u32>,

    /// Enable gzip compression
    pub gzip_enabled: bool,

    /// Enable brotli compression
    pub brotli_enabled: bool,

    /// Use native root certificates
    pub use_native_certs: bool,

    /// Require HTTPS
    pub https_only: bool,

    /// Enable gzip compression
    pub gzip: bool,

    /// Enable brotli compression
    pub brotli: bool,

    /// Enable deflate compression
    pub deflate: bool,

    /// User agent string
    pub user_agent: String,

    /// Enable HTTP/3 (QUIC)
    pub http3_enabled: bool,

    /// Connection pool size
    pub pool_size: usize,

    /// Maximum number of redirects to follow
    pub max_redirects: usize,

    /// Enable cookie storage
    pub cookie_store: bool,

    /// Enable response compression
    pub response_compression: bool,

    /// Enable request compression
    pub request_compression: bool,

    /// DNS cache duration
    pub dns_cache_duration: Duration,

    /// Enable DNS over HTTPS
    pub dns_over_https: bool,

    /// Enable happy eyeballs for IPv6
    pub happy_eyeballs: bool,

    /// Local address to bind to
    pub local_address: Option<std::net::IpAddr>,

    /// Interface to bind to
    pub interface: Option<String>,

    /// Enable HTTP/2 server push
    pub http2_server_push: bool,

    /// HTTP/2 initial stream window size
    pub http2_initial_stream_window_size: Option<u32>,

    /// HTTP/2 initial connection window size
    pub http2_initial_connection_window_size: Option<u32>,

    /// HTTP/2 max concurrent streams
    pub http2_max_concurrent_streams: Option<u32>,

    /// Enable HTTP/2 keep-alive
    pub http2_keep_alive: bool,

    /// HTTP/2 keep-alive interval
    pub http2_keep_alive_interval: Option<Duration>,

    /// HTTP/2 keep-alive timeout
    pub http2_keep_alive_timeout: Option<Duration>,

    /// Enable HTTP/2 adaptive window scaling
    pub http2_adaptive_window_scaling: bool,

    /// Trust DNS
    pub trust_dns: bool,

    /// Enable metrics collection
    pub metrics_enabled: bool,

    /// Enable tracing
    pub tracing_enabled: bool,

    /// Connection reuse strategy
    pub connection_reuse: ConnectionReuse,

    /// Retry policy
    pub retry_policy: RetryPolicy,

    // ===== HTTP/3 (QUIC) Configuration =====
    /// QUIC connection maximum idle timeout before closing
    /// Controls how long a QUIC connection can remain idle before being closed
    pub quic_max_idle_timeout: Option<Duration>,

    /// QUIC per-stream receive window size in bytes
    /// Controls flow control for individual HTTP/3 streams
    pub quic_stream_receive_window: Option<u32>,

    /// QUIC connection-wide receive window size in bytes  
    /// Controls aggregate flow control across all streams in a connection
    pub quic_receive_window: Option<u32>,

    /// QUIC send window size in bytes
    /// Controls how much data can be sent without acknowledgment
    pub quic_send_window: Option<u64>,

    /// Use BBR congestion control algorithm instead of CUBIC
    /// BBR typically provides better performance over high-latency networks
    pub quic_congestion_bbr: bool,

    /// Enable TLS 1.3 early data (0-RTT) for QUIC connections
    /// Reduces connection establishment latency for resumed connections
    pub tls_early_data: bool,

    /// Maximum HTTP/3 header field section size in bytes
    /// Controls maximum size of HTTP/3 headers to prevent memory exhaustion
    pub h3_max_field_section_size: Option<u64>,

    /// Enable HTTP/3 protocol grease
    /// Sends random grease values to ensure protocol extensibility
    pub h3_enable_grease: bool,
}

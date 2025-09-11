//! HTTP client preset configurations
//!
//! Provides specialized configuration presets optimized for different use cases
//! including AI providers, streaming responses, batch processing, and low-latency applications.

use std::time::Duration;

use super::core::HttpConfig;
use super::core::retry::{ConnectionReuse, RetryPolicy, RetryableError};

impl HttpConfig {
    /// Create a new configuration optimized for AI providers
    ///
    /// This preset is specifically tuned for interacting with AI/LLM providers like `OpenAI`,
    /// Anthropic, and others. It includes settings for larger response windows, aggressive
    /// connection reuse, and enhanced retry policies for API reliability.
    ///
    /// # Features
    /// - Larger connection pools and timeouts for AI workloads
    /// - Enhanced retry policy with more retry attempts
    /// - HTTPS-only for security
    /// - DNS over HTTPS for privacy
    /// - HTTP/3 enabled with optimized QUIC settings
    /// - BBR congestion control for optimal performance
    /// - Larger header limits for AI metadata
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::ai_optimized();
    /// assert_eq!(config.pool_max_idle_per_host, 64);
    /// assert!(config.https_only);
    /// assert!(config.quic_congestion_bbr);
    /// ```
    #[must_use] 
    pub fn ai_optimized() -> Self {
        Self {
            pool_max_idle_per_host: 64,
            pool_idle_timeout: Duration::from_secs(120),
            timeout: Duration::from_secs(300),
            connect_timeout: Duration::from_secs(5),
            tcp_keepalive: Some(Duration::from_secs(30)),
            tcp_nodelay: true,
            http2_adaptive_window: true,
            http2_max_frame_size: Some(2 << 20), // 2MB for large responses
            use_native_certs: true,
            https_only: true,

            deflate: true,
            gzip_enabled: true,
            brotli_enabled: true,
            user_agent: "fluent-ai-http3/0.1.0 (AI-optimized QUIC/HTTP3+rustls)".to_string(),
            http3_enabled: true,
            pool_size: 20,
            max_redirects: 3,
            cookie_store: false,
            response_compression: true,
            request_compression: true,
            dns_cache_duration: Duration::from_secs(600),
            dns_over_https: true,
            happy_eyeballs: true,
            local_address: None,
            interface: None,
            http2_server_push: false,
            http2_initial_stream_window_size: Some(2 << 20), // 2MB
            http2_initial_connection_window_size: Some(8 << 20), // 8MB
            http2_max_concurrent_streams: Some(100),
            http2_keep_alive: true,
            http2_keep_alive_interval: Some(Duration::from_secs(20)),
            http2_keep_alive_timeout: Some(Duration::from_secs(5)),
            http2_adaptive_window_scaling: true,
            trust_dns: true,
            metrics_enabled: true,
            tracing_enabled: true,
            connection_reuse: ConnectionReuse::Aggressive,
            retry_policy: RetryPolicy {
                max_retries: 5,
                base_delay: Duration::from_millis(50),
                max_delay: Duration::from_secs(60),
                backoff_factor: 1.5,
                jitter_factor: 0.2,
                retry_on_status: vec![429, 500, 502, 503, 504, 520, 521, 522, 523, 524],
                retry_on_errors: vec![
                    RetryableError::Network,
                    RetryableError::Timeout,
                    RetryableError::Connection,
                    RetryableError::Dns,
                    RetryableError::Tls,
                ],
            },
            
            // UTF-8 validation security - strict by default for AI providers
            utf8_validation_mode: crate::http::conversions::SecurityMode::Strict,

            // AI-optimized HTTP/3 (QUIC) settings for maximum performance
            quic_max_idle_timeout: Some(Duration::from_secs(120)), // Longer idle for AI workloads
            quic_stream_receive_window: Some(512 * 1024), // 512KB per stream for large AI responses
            quic_receive_window: Some(4 * 1024 * 1024), // 4MB connection window for high throughput
            quic_send_window: Some(2 * 1024 * 1024),    // 2MB send window for large requests
            quic_congestion_bbr: true,                  // BBR for optimal AI provider performance
            tls_early_data: true,                       // Enable 0-RTT for repeat connections
            h3_max_field_section_size: Some(64 * 1024), // 64KB for large AI headers
            h3_enable_grease: true,                     // Enable grease for future compatibility
            
            // Compression level configuration
            gzip_level: Some(6),     // Balanced compression/speed for AI workloads
            brotli_level: Some(4),   // Faster brotli for real-time AI responses  
            deflate_level: Some(6),  // Balanced deflate compression
        }
    }

    /// Create a new configuration optimized for streaming responses
    ///
    /// This preset is designed for long-lived streaming connections such as Server-Sent Events,
    /// WebSocket upgrades, or streaming AI responses. It maximizes buffer sizes and minimizes
    /// connection interruptions.
    ///
    /// # Features
    /// - Extended timeouts for long-running streams
    /// - Larger stream and connection windows
    /// - More frequent keep-alive for connection stability
    /// - Fewer concurrent streams to focus bandwidth
    /// - Reduced retry attempts to avoid duplicate streams
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::streaming_optimized();
    /// assert_eq!(config.timeout.as_secs(), 600);
    /// assert_eq!(config.http2_max_concurrent_streams, Some(10));
    /// ```
    #[must_use] 
    pub fn streaming_optimized() -> Self {
        let mut config = Self::ai_optimized();
        config.timeout = Duration::from_secs(600); // 10 minutes for streaming
        config.pool_idle_timeout = Duration::from_secs(300); // 5 minutes
        config.http2_initial_stream_window_size = Some(4 << 20); // 4MB
        config.http2_initial_connection_window_size = Some(16 << 20); // 16MB
        config.http2_max_concurrent_streams = Some(10); // Fewer concurrent streams for streaming
        config.http2_keep_alive_interval = Some(Duration::from_secs(10)); // More frequent keep-alive
        config.retry_policy.max_retries = 2; // Fewer retries for streaming

        // Streaming-optimized QUIC settings
        config.quic_max_idle_timeout = Some(Duration::from_secs(600)); // Match streaming timeout
        config.quic_stream_receive_window = Some(1024 * 1024); // 1MB per stream for streaming
        config.quic_receive_window = Some(8 * 1024 * 1024); // 8MB connection window for streaming
        config.quic_send_window = Some(4 * 1024 * 1024); // 4MB send window for streaming
        config.h3_max_field_section_size = Some(32 * 1024); // 32KB headers for streaming metadata

        config
    }

    /// Create a new configuration optimized for batch processing
    ///
    /// This preset is designed for high-throughput batch operations with many concurrent
    /// requests. It maximizes connection pools and concurrent streams while extending
    /// timeouts for batch operations.
    ///
    /// # Features
    /// - Very large connection pools for concurrent requests
    /// - Extended timeouts for batch processing
    /// - Maximum concurrent streams for throughput
    /// - Aggressive retry policy for batch reliability
    /// - Optimized for high-volume operations
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::batch_optimized();
    /// assert_eq!(config.pool_max_idle_per_host, 128);
    /// assert_eq!(config.pool_size, 50);
    /// assert_eq!(config.retry_policy.max_retries, 10);
    /// ```
    #[must_use] 
    pub fn batch_optimized() -> Self {
        let mut config = Self::ai_optimized();
        config.pool_max_idle_per_host = 128;
        config.pool_size = 50;
        config.timeout = Duration::from_secs(900); // 15 minutes for batch
        config.http2_max_concurrent_streams = Some(200);
        config.retry_policy.max_retries = 10;
        config.retry_policy.max_delay = Duration::from_secs(120);

        // Batch-optimized QUIC settings for high throughput
        config.quic_max_idle_timeout = Some(Duration::from_secs(300)); // 5 minutes for batch efficiency
        config.quic_stream_receive_window = Some(2 * 1024 * 1024); // 2MB per stream for batch
        config.quic_receive_window = Some(16 * 1024 * 1024); // 16MB connection window for batch
        config.quic_send_window = Some(8 * 1024 * 1024); // 8MB send window for batch uploads
        config.h3_max_field_section_size = Some(128 * 1024); // 128KB headers for batch metadata

        config
    }

    /// Create a new configuration for low-latency applications
    ///
    /// This preset minimizes latency at every opportunity, trading some reliability for speed.
    /// Ideal for real-time applications, gaming, or interactive AI responses where speed
    /// is critical.
    ///
    /// # Features
    /// - Minimal timeouts for fast failure detection
    /// - Aggressive keep-alive settings
    /// - TLS 0-RTT enabled for connection speed
    /// - BBR congestion control for latency optimization
    /// - Reduced retry attempts for faster failure
    /// - Smaller buffers to minimize buffering delays
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::low_latency();
    /// assert_eq!(config.connect_timeout.as_secs(), 2);
    /// assert_eq!(config.timeout.as_secs(), 10);
    /// assert_eq!(config.retry_policy.max_retries, 1);
    /// assert!(config.tls_early_data);
    /// ```
    #[must_use] 
    pub fn low_latency() -> Self {
        let mut config = Self::ai_optimized();
        config.connect_timeout = Duration::from_secs(2);
        config.timeout = Duration::from_secs(10);
        config.tcp_keepalive = Some(Duration::from_secs(10));
        config.http2_keep_alive_interval = Some(Duration::from_secs(5));
        config.retry_policy.max_retries = 1;
        config.retry_policy.base_delay = Duration::from_millis(10);
        config.retry_policy.max_delay = Duration::from_secs(5);

        // Low-latency optimized QUIC settings
        config.quic_max_idle_timeout = Some(Duration::from_secs(15)); // Short idle for low latency
        config.quic_stream_receive_window = Some(128 * 1024); // 128KB per stream for low latency
        config.quic_receive_window = Some(512 * 1024); // 512KB connection window for low latency
        config.quic_send_window = Some(256 * 1024); // 256KB send window for low latency
        config.quic_congestion_bbr = true; // BBR for better latency characteristics
        config.tls_early_data = true; // Critical for low latency - enable 0-RTT
        config.h3_max_field_section_size = Some(8 * 1024); // 8KB headers to minimize latency

        config
    }

    /// Set the user agent string
    ///
    /// Configures the User-Agent header that will be sent with requests.
    /// The user agent identifies the client to servers and can affect
    /// how requests are handled or rate-limited.
    ///
    /// # Arguments
    /// * `user_agent` - User agent string to use
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_user_agent("MyApp/1.0.0".to_string());
    /// assert_eq!(config.user_agent, "MyApp/1.0.0");
    /// ```
    #[must_use] 
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
}

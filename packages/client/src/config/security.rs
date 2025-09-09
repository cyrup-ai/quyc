//! Security and TLS configuration methods
//!
//! Provides builder methods for configuring security settings including
//! HTTPS enforcement, TLS options, and cryptographic preferences.

use super::core::HttpConfig;
use super::core::retry::RetryPolicy;

impl HttpConfig {
    /// Enable or disable HTTPS-only mode
    ///
    /// When enabled, all HTTP requests will be rejected and only HTTPS
    /// connections will be allowed. This provides transport security
    /// but may break compatibility with HTTP-only services.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enforce HTTPS-only connections
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_https_only(true);
    /// assert!(config.https_only);
    /// ```
    pub fn with_https_only(mut self, enabled: bool) -> Self {
        self.https_only = enabled;
        self
    }

    /// Enable or disable native certificate store usage
    ///
    /// When enabled, uses the operating system's certificate store for
    /// TLS verification. When disabled, uses a bundled certificate store.
    /// Native stores provide better compatibility with corporate environments.
    ///
    /// # Arguments
    /// * `enabled` - Whether to use native certificate store
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_native_certs(true);
    /// assert!(config.use_native_certs);
    /// ```
    pub fn with_native_certs(mut self, enabled: bool) -> Self {
        self.use_native_certs = enabled;
        self
    }

    /// Enable or disable TLS 1.3 early data (0-RTT)
    ///
    /// TLS 1.3 early data allows sending application data in the first
    /// round trip, reducing connection latency. However, it has replay
    /// attack implications and should be used carefully.
    ///
    /// # Security Considerations
    /// - Early data can be replayed by attackers
    /// - Only use for idempotent requests
    /// - Servers must handle potential replay attacks
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable TLS 1.3 early data
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_tls_early_data(true);
    /// assert!(config.tls_early_data);
    /// ```
    pub fn with_tls_early_data(mut self, enabled: bool) -> Self {
        self.tls_early_data = enabled;
        self
    }

    /// Enable or disable DNS over HTTPS (DoH)
    ///
    /// DNS over HTTPS encrypts DNS queries, preventing eavesdropping and
    /// manipulation of DNS traffic. This improves privacy and security
    /// but may increase latency and complexity.
    ///
    /// # Arguments
    /// * `enabled` - Whether to use DNS over HTTPS
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_dns_over_https(true);
    /// assert!(config.dns_over_https);
    /// ```
    pub fn with_dns_over_https(mut self, enabled: bool) -> Self {
        self.dns_over_https = enabled;
        self
    }

    /// Enable or disable BBR congestion control algorithm
    ///
    /// BBR (Bottleneck Bandwidth and Round-trip propagation time) is a
    /// congestion control algorithm that can provide better performance
    /// over high-latency networks compared to traditional CUBIC.
    ///
    /// # Security Note
    /// While not directly a security feature, BBR can provide more
    /// predictable performance characteristics which may help avoid
    /// certain timing-based attacks.
    ///
    /// # Arguments
    /// * `enabled` - Whether to use BBR congestion control
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_quic_congestion_bbr(true);
    /// assert!(config.quic_congestion_bbr);
    /// ```
    pub fn with_quic_congestion_bbr(mut self, enabled: bool) -> Self {
        self.quic_congestion_bbr = enabled;
        self
    }

    /// Set maximum HTTP/3 header field section size
    ///
    /// Limits the maximum size of HTTP/3 headers to prevent memory
    /// exhaustion attacks. Smaller limits provide better security
    /// but may break compatibility with services using large headers.
    ///
    /// # Security Considerations
    /// - Prevents header-based DoS attacks
    /// - Limits memory usage per request
    /// - Should be balanced with application needs
    ///
    /// # Arguments
    /// * `size` - Maximum header section size in bytes
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_h3_max_field_section_size(32 * 1024); // 32KB
    /// assert_eq!(config.h3_max_field_section_size, Some(32 * 1024));
    /// ```
    pub fn with_h3_max_field_section_size(mut self, size: u64) -> Self {
        self.h3_max_field_section_size = Some(size);
        self
    }

    /// Enable or disable HTTP/3 protocol grease
    ///
    /// Protocol grease sends random values in protocol fields to ensure
    /// extensibility and prevent ossification. This helps maintain
    /// protocol evolution but may rarely cause compatibility issues.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable HTTP/3 protocol grease
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_h3_enable_grease(true);
    /// assert!(config.h3_enable_grease);
    /// ```
    pub fn with_h3_enable_grease(mut self, enabled: bool) -> Self {
        self.h3_enable_grease = enabled;
        self
    }

    /// Set the retry policy with security considerations
    ///
    /// Configures how requests are retried on failure. Aggressive retry
    /// policies can improve reliability but may amplify DoS attacks
    /// or leak information through timing patterns.
    ///
    /// # Security Considerations
    /// - Limit retry attempts to prevent amplification attacks
    /// - Use jitter to prevent timing correlation
    /// - Consider backoff strategies for rate-limited services
    ///
    /// # Arguments
    /// * `policy` - Retry policy configuration
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::{HttpConfig, RetryPolicy, RetryableError};
    ///
    /// let policy = RetryPolicy {
    ///     max_retries: 3,
    ///     base_delay: Duration::from_millis(100),
    ///     max_delay: Duration::from_secs(30),
    ///     backoff_factor: 2.0,
    ///     jitter_factor: 0.1,
    ///     retry_on_status: vec![429, 500, 502, 503, 504],
    ///     retry_on_errors: vec![RetryableError::Network, RetryableError::Timeout],
    /// };
    ///
    /// let config = HttpConfig::default()
    ///     .with_retry_policy(policy);
    /// assert_eq!(config.retry_policy.max_retries, 3);
    /// ```
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Enable or disable Trust DNS resolver
    ///
    /// Trust DNS is a DNS resolver implementation that may provide
    /// better security and performance characteristics compared to
    /// system DNS resolvers in some environments.
    ///
    /// # Arguments
    /// * `enabled` - Whether to use Trust DNS resolver
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_trust_dns(true);
    /// assert!(config.trust_dns);
    /// ```
    pub fn with_trust_dns(mut self, enabled: bool) -> Self {
        self.trust_dns = enabled;
        self
    }
}

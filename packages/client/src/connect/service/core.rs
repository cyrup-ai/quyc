//! Core connector service structure and configuration
//!
//! Contains the main `ConnectorService` struct with TLS and proxy configuration
//! management for HTTP/3 connection establishment.

#![allow(dead_code)]

use std::time::Duration;


use hyper_util::client::legacy::connect::HttpConnector;
// native_tls removed - using rustls universally
#[cfg(feature = "__rustls")]
use rustls;

use super::super::proxy::Intercepted;
use crate::error::BoxError;

/// Configuration for connector service
///
/// Groups network and connection settings to reduce parameter count
/// and improve API ergonomics.
#[derive(Debug, Clone)]
pub struct ConnectorServiceConfig {
    /// Proxy configurations (max 4 proxies supported)
    pub proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
    /// User agent header value for requests
    pub user_agent: Option<http::HeaderValue>,
    /// Local IP address to bind to
    pub local_address: Option<std::net::IpAddr>,
    /// Network interface to bind to
    pub interface: Option<String>,
    /// Whether to disable Nagle's algorithm (`TCP_NODELAY`)
    pub nodelay: bool,
    /// Connection timeout duration
    pub connect_timeout: Option<Duration>,
    /// Happy eyeballs timeout for dual-stack connections
    pub happy_eyeballs_timeout: Option<Duration>,
    /// Whether to collect TLS connection information
    pub tls_info: bool,
}

impl Default for ConnectorServiceConfig {
    fn default() -> Self {
        Self {
            proxies: arrayvec::ArrayVec::new(),
            user_agent: None,
            local_address: None,
            interface: None,
            nodelay: true,
            connect_timeout: Some(Duration::from_secs(10)),
            happy_eyeballs_timeout: Some(Duration::from_millis(300)),
            tls_info: false,
        }
    }
}

/// Core connector service with zero-allocation streaming
#[derive(Clone, Debug)]
pub struct ConnectorService {
    pub(super) http: HttpConnector,
    // native_tls field removed - using TlsManager from src/tls/
    #[cfg(feature = "__rustls")]
    pub(super) rustls_config: Option<std::sync::Arc<rustls::ClientConfig>>,
    pub(super) intercepted: Intercepted,
    pub(super) user_agent: Option<http::HeaderValue>,
    pub(super) local_address: Option<std::net::IpAddr>,
    pub(super) interface: Option<String>,
    pub(super) nodelay: bool,
    pub(super) connect_timeout: Option<Duration>,
    pub(super) happy_eyeballs_timeout: Option<Duration>,
    pub(super) tls_info: bool,
}

impl ConnectorService {
    /// Create new connector service with configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Proxy configuration is invalid or malformed
    /// - Required proxy authentication information is missing or invalid
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        http: HttpConnector,
        #[cfg(feature = "__rustls")] rustls_config: Option<rustls::ClientConfig>,
        proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
        user_agent: Option<http::HeaderValue>,
        local_address: Option<std::net::IpAddr>,
        interface: Option<String>,
        nodelay: bool,
        connect_timeout: Option<Duration>,
        happy_eyeballs_timeout: Option<Duration>,
        tls_info: bool,
    ) -> Result<Self, BoxError> {
        let intercepted = if proxies.is_empty() {
            Intercepted::none()
        } else {
            Intercepted::from_proxies(proxies)?
        };

        Ok(Self {
            http,
            #[cfg(feature = "__rustls")]
            rustls_config: rustls_config.map(std::sync::Arc::new),
            intercepted,
            user_agent,
            local_address,
            interface,
            nodelay,
            connect_timeout,
            happy_eyeballs_timeout,
            tls_info,
        })
    }

    /// Create new connector service with configuration struct
    ///
    /// This is the recommended approach for new code as it reduces parameter count
    /// and improves API ergonomics.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Proxy configuration is invalid or malformed
    /// - Required proxy authentication information is missing or invalid
    pub fn with_config(
        http: HttpConnector,
        #[cfg(feature = "__rustls")] rustls_config: Option<rustls::ClientConfig>,
        config: ConnectorServiceConfig,
    ) -> Result<Self, BoxError> {
        Self::new(
            http,
            #[cfg(feature = "__rustls")]
            rustls_config,
            config.proxies,
            config.user_agent,
            config.local_address,
            config.interface,
            config.nodelay,
            config.connect_timeout,
            config.happy_eyeballs_timeout,
            config.tls_info,
        )
    }
}

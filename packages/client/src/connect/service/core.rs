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
}

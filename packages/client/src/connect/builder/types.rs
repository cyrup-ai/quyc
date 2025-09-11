//! Core `ConnectorBuilder` struct and basic configuration methods
//!
//! Provides the main builder struct and fundamental configuration options.

use std::time::Duration;

use hyper_util::client::legacy::connect::HttpConnector;

#[cfg(feature = "__rustls")]
use rustls;

/// Builder for HTTP/3 connectors with configuration options
#[derive(Clone, Debug)]
pub struct ConnectorBuilder {
    #[cfg(feature = "__tls")]
    pub(super) tls_built: bool,
    pub(super) connect_timeout: Option<Duration>,
    pub(super) happy_eyeballs_timeout: Option<Duration>,
    pub(super) nodelay: bool,
    pub(super) enforce_http: bool,
    pub(super) http_connector: Option<HttpConnector>,

    #[cfg(feature = "__rustls")]
    pub(super) rustls_config: Option<rustls::ClientConfig>,
    pub(super) proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
    pub(super) user_agent: Option<http::HeaderValue>,
    pub(super) local_address: Option<std::net::IpAddr>,
    pub(super) interface: Option<String>,
    pub(super) tls_info: bool,
}

impl ConnectorBuilder {
    /// Create a new connector builder with default settings
    #[must_use] 
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "__tls")]
            tls_built: false,
            connect_timeout: Some(Duration::from_secs(10)),
            happy_eyeballs_timeout: Some(Duration::from_millis(300)),
            nodelay: true,
            enforce_http: false,
            http_connector: None,

            #[cfg(feature = "__rustls")]
            rustls_config: None,
            proxies: arrayvec::ArrayVec::new(),
            user_agent: None,
            local_address: None,
            interface: None,
            tls_info: false,
        }
    }

    /// Sets the timeout for connection establishment.
    ///
    /// # Arguments
    /// * `timeout` - Duration to wait for connection establishment
    ///
    /// # Returns
    /// Self with the timeout configured
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Sets the connection timeout for HTTP connections.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Enables or disables `TCP_NODELAY` for connections.
    #[must_use]
    pub fn nodelay(mut self, nodelay: bool) -> Self {
        self.nodelay = nodelay;
        self
    }

    /// Sets the happy eyeballs timeout for dual-stack connections.
    #[must_use]
    pub fn happy_eyeballs_timeout(mut self, timeout: Duration) -> Self {
        self.happy_eyeballs_timeout = Some(timeout);
        self
    }

    /// Sets `TCP_NODELAY` option for connections.
    #[must_use]
    pub fn tcp_nodelay(mut self, nodelay: bool) -> Self {
        self.nodelay = nodelay;
        self
    }

    /// Enforce HTTP-only connections (disable HTTPS)
    #[must_use]
    pub fn enforce_http(mut self, enforce: bool) -> Self {
        self.enforce_http = enforce;
        self
    }

    /// Enable HTTPS or HTTP connections
    #[cfg(feature = "__tls")]
    #[must_use]
    pub fn https_or_http(mut self) -> Self {
        self.tls_built = true;
        self
    }
}

impl Default for ConnectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

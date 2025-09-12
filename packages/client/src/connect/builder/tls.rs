//! TLS configuration methods for `ConnectorBuilder`
//!
//! Provides TLS-specific constructors and configuration for both native-tls and rustls.



use hyper_util::client::legacy::connect::HttpConnector;

#[cfg(feature = "__rustls")]
use rustls;

use super::types::ConnectorBuilder;
use crate::error::BoxError;

/// Configuration for TLS connector builder
///
/// Groups network and TLS-specific settings to reduce parameter count
/// and improve API ergonomics.
#[derive(Debug, Clone)]
pub struct TlsConnectorConfig {
    /// Proxy configurations (max 4 proxies supported)
    pub proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
    /// User agent header value for requests
    pub user_agent: Option<http::HeaderValue>,
    /// Local IP address to bind to
    pub local_address: Option<std::net::IpAddr>,
    /// Network interface to bind to (platform-specific)
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia", 
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "solaris",
        target_os = "tvos",
        target_os = "visionos",
        target_os = "watchos",
    ))]
    pub interface: Option<String>,
    /// Whether to disable Nagle's algorithm (`TCP_NODELAY`)
    pub nodelay: bool,
    /// Whether to collect TLS connection information
    pub tls_info: bool,
}

impl Default for TlsConnectorConfig {
    fn default() -> Self {
        Self {
            proxies: arrayvec::ArrayVec::new(),
            user_agent: None,
            local_address: None,
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos", 
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "solaris",
                target_os = "tvos",
                target_os = "visionos",
                target_os = "watchos",
            ))]
            interface: None,
            nodelay: true,
            tls_info: false,
        }
    }
}

impl ConnectorBuilder {

    /// Create new connector with Rustls TLS
    ///
    /// # Errors
    ///
    /// Currently this function does not return errors, but the `Result` return type
    /// is used for API consistency and future extensibility. May return errors in
    /// future versions if TLS configuration validation is added.
    #[cfg(feature = "__rustls")]
    #[must_use = "Connector builders return a new connector and should be used"]
    #[allow(clippy::too_many_arguments)]
    pub fn new_rustls_tls(
        http: HttpConnector,
        config: rustls::ClientConfig,
        proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
        user_agent: Option<http::HeaderValue>,
        local_address: Option<std::net::IpAddr>,
        #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "solaris",
            target_os = "tvos",
            target_os = "visionos",
            target_os = "watchos",
        ))]
        interface: Option<&str>,
        nodelay: bool,
        tls_info: bool,
    ) -> Result<Self, BoxError> {
        let mut builder = Self::new();
        #[cfg(feature = "__tls")]
        {
            builder.tls_built = true;
        }
        builder.http_connector = Some(http);
        builder.rustls_config = Some(config);
        builder.proxies = proxies;
        builder.user_agent = user_agent;
        builder.local_address = local_address;
        if let Some(iface) = interface {
            builder.interface = Some(iface.to_string());
        }
        builder.nodelay = nodelay;
        builder.tls_info = tls_info;
        Ok(builder)
    }

    /// Creates a connector from pre-built Rustls TLS components.
    ///
    /// # Errors
    ///
    /// Currently this function does not return errors, but the `Result` return type
    /// is used for API consistency and future extensibility. May return errors in
    /// future versions if TLS configuration validation is added.
    #[cfg(feature = "__rustls")]
    #[must_use = "Connector builders return a new connector and should be used"]
    #[allow(clippy::too_many_arguments)]
    pub fn from_built_rustls_tls(
        http: HttpConnector,
        config: rustls::ClientConfig,
        proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
        user_agent: Option<http::HeaderValue>,
        local_address: Option<std::net::IpAddr>,
        #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "solaris",
            target_os = "tvos",
            target_os = "visionos",
            target_os = "watchos",
        ))]
        interface: Option<&str>,
        nodelay: bool,
        tls_info: bool,
    ) -> Result<Self, BoxError> {
        Self::new_rustls_tls(
            http,
            config,
            proxies,
            user_agent,
            local_address,
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "solaris",
                target_os = "tvos",
                target_os = "visionos",
                target_os = "watchos",
            ))]
            interface,
            nodelay,
            tls_info,
        )
    }

    /// Create new connector with Rustls TLS using config struct
    ///
    /// This is the recommended approach for new code as it reduces parameter count
    /// and improves API ergonomics.
    ///
    /// # Errors
    ///
    /// Currently this function does not return errors, but the `Result` return type
    /// is used for API consistency and future extensibility. May return errors in
    /// future versions if TLS configuration validation is added.
    #[cfg(feature = "__rustls")]
    #[must_use = "Connector builders return a new connector and should be used"]
    pub fn with_rustls_tls_config(
        http: HttpConnector,
        rustls_config: rustls::ClientConfig,
        tls_config: TlsConnectorConfig,
    ) -> Result<Self, BoxError> {
        Self::new_rustls_tls(
            http,
            rustls_config,
            tls_config.proxies,
            tls_config.user_agent,
            tls_config.local_address,
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "solaris",
                target_os = "tvos",
                target_os = "visionos",
                target_os = "watchos",
            ))]
            tls_config.interface.as_deref(),
            tls_config.nodelay,
            tls_config.tls_info,
        )
    }

    /// Creates a connector from pre-built Rustls TLS components using config struct
    ///
    /// This is the recommended approach for new code as it reduces parameter count
    /// and improves API ergonomics.
    ///
    /// # Errors
    ///
    /// Currently this function does not return errors, but the `Result` return type
    /// is used for API consistency and future extensibility. May return errors in
    /// future versions if TLS configuration validation is added.
    #[cfg(feature = "__rustls")]
    #[must_use = "Connector builders return a new connector and should be used"]
    pub fn from_built_rustls_tls_config(
        http: HttpConnector,
        rustls_config: rustls::ClientConfig,
        tls_config: TlsConnectorConfig,
    ) -> Result<Self, BoxError> {
        Self::from_built_rustls_tls(
            http,
            rustls_config,
            tls_config.proxies,
            tls_config.user_agent,
            tls_config.local_address,
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "solaris",
                target_os = "tvos",
                target_os = "visionos",
                target_os = "watchos",
            ))]
            tls_config.interface.as_deref(),
            tls_config.nodelay,
            tls_config.tls_info,
        )
    }
}

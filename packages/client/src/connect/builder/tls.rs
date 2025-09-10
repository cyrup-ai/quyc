//! TLS configuration methods for ConnectorBuilder
//!
//! Provides TLS-specific constructors and configuration for both native-tls and rustls.



use hyper_util::client::legacy::connect::HttpConnector;

#[cfg(feature = "__rustls")]
use rustls;

use super::types::ConnectorBuilder;
use crate::error::BoxError;

impl ConnectorBuilder {

    /// Create new connector with Rustls TLS
    #[cfg(feature = "__rustls")]
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
    #[cfg(feature = "__rustls")]
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
}

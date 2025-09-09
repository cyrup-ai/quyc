//! HTTP/3 connector types and service abstractions
//!
//! Provides the core connector types for establishing HTTP/3 connections
//! with support for different TLS configurations and service layers.

use crate::connect::service::ConnectorService;

/// HTTP/3 connection provider with zero-allocation streaming
#[derive(Clone, Debug)]
pub struct Connector {
    pub inner: ConnectorKind,
}

impl Default for Connector {
    fn default() -> Self {
        Self {
            inner: ConnectorKind::default(),
        }
    }
}

/// Enumeration of different connector implementations
#[derive(Clone, Debug)]
pub enum ConnectorKind {
    #[cfg(feature = "__tls")]
    BuiltDefault(ConnectorService),
    #[cfg(not(feature = "__tls"))]
    BuiltHttp(ConnectorService),
    WithLayers(BoxedConnectorService),
}

impl Default for ConnectorKind {
    fn default() -> Self {
        #[cfg(feature = "__tls")]
        {
            // Create default HttpConnector for TLS-enabled builds
            let http = hyper_util::client::legacy::connect::HttpConnector::new();
            let proxies = arrayvec::ArrayVec::new();
            
            match ConnectorService::new(
                http,
                #[cfg(feature = "default-tls")] None,
                #[cfg(feature = "__rustls")] None,
                proxies,
                None, // user_agent
                None, // local_address
                None, // interface
                true, // nodelay
                Some(std::time::Duration::from_secs(30)), // connect_timeout
                Some(std::time::Duration::from_millis(300)), // happy_eyeballs_timeout
                false, // tls_info
            ) {
                Ok(service) => Self::BuiltDefault(service),
                Err(_) => {
                    // Create minimal fallback connector with default configuration
                    let http = hyper_util::client::legacy::connect::HttpConnector::new();
                    let proxies = arrayvec::ArrayVec::new();
                    
                    Self::BuiltDefault(ConnectorService::new(
                        http,
                        #[cfg(feature = "default-tls")] None,
                        #[cfg(feature = "__rustls")] None,
                        proxies,
                        None,
                        None,
                        None,
                        true,
                        Some(std::time::Duration::from_secs(10)),
                        Some(std::time::Duration::from_millis(100)),
                        false,
                    ).expect("Fallback connector creation should never fail"))
                }
            }
        }
        #[cfg(not(feature = "__tls"))]
        {
            // Create default HttpConnector for non-TLS builds
            let http = hyper_util::client::legacy::connect::HttpConnector::new();
            let proxies = arrayvec::ArrayVec::new();
            
            match ConnectorService::new(
                http,
                #[cfg(feature = "default-tls")] None, // tls
                #[cfg(feature = "__rustls")] None, // rustls_config
                proxies,
                None, // user_agent
                None, // local_address
                None, // interface
                true, // nodelay
                Some(std::time::Duration::from_secs(30)), // connect_timeout
                Some(std::time::Duration::from_millis(300)), // happy_eyeballs_timeout
                false, // tls_info
            ) {
                Ok(service) => Self::BuiltHttp(service),
                Err(_) => {
                    // Create minimal fallback connector with default configuration
                    let http = hyper_util::client::legacy::connect::HttpConnector::new();
                    let proxies = arrayvec::ArrayVec::new();
                    
                    Self::BuiltHttp(ConnectorService::new(
                        http,
                        #[cfg(feature = "default-tls")] None, // tls
                        #[cfg(feature = "__rustls")] None, // rustls_config
                        proxies,
                        None,
                        None,
                        None,
                        true,
                        Some(std::time::Duration::from_secs(10)),
                        Some(std::time::Duration::from_millis(100)),
                        false,
                    ).expect("Fallback connector creation should never fail"))
                }
            }
        }
    }
}

/// Direct ConnectorService type - no more Service trait boxing needed
pub type BoxedConnectorService = ConnectorService;

/// Simplified approach: Use trait objects for connector layers
/// This provides the same functionality as tower::Layer but with AsyncStream services
/// Boxed connector layer type for composable connection handling.
pub type BoxedConnectorLayer =
    Box<dyn Fn(BoxedConnectorService) -> BoxedConnectorService + Send + Sync + 'static>;

/// Sealed module for internal traits.
pub mod sealed {
    /// Unnameable struct for internal use.
    #[derive(Default, Debug)]
    pub struct Unnameable;
}

pub use sealed::Unnameable;



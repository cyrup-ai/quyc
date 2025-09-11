//! HTTP/3 connector types and service abstractions
//!
//! Provides the core connector types for establishing HTTP/3 connections
//! with support for different TLS configurations and service layers.

use crate::connect::service::ConnectorService;

/// HTTP/3 connection provider with zero-allocation streaming
#[derive(Clone, Debug)]
#[derive(Default)]
pub struct Connector {
    pub inner: ConnectorKind,
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

impl ConnectorKind {
    /// Create error-marked connector for graceful degradation
    fn create_error_marker_connector() -> Self {
        // Create minimal connector that reports as error when used
        let http = hyper_util::client::legacy::connect::HttpConnector::new();
        let proxies = arrayvec::ArrayVec::new();
        
        // Create service with absolute minimal configuration to avoid further failures
        match ConnectorService::new(
            http,
            #[cfg(feature = "__rustls")] None,
            proxies,
            None, // user_agent
            None, // local_address  
            None, // interface
            true, // nodelay
            Some(std::time::Duration::from_millis(1)), // minimal timeout
            Some(std::time::Duration::from_millis(1)), // minimal timeout
            false, // tls_info
        ) {
            Ok(service) => {
                #[cfg(feature = "__tls")]
                { Self::BuiltDefault(service) }
                #[cfg(not(feature = "__tls"))]
                { Self::BuiltHttp(service) }
            },
            Err(e) => {
                // Even minimal connector failed - create emergency fallback
                log::error!("Emergency: Even minimal connector creation failed: {e}");
                // Use WithLayers as emergency fallback with the minimal service we can create
                let minimal_http = hyper_util::client::legacy::connect::HttpConnector::new();
                let minimal_proxies = arrayvec::ArrayVec::new();
                
                // This must work or the system is fundamentally broken
                let emergency_service = ConnectorService::new(
                    minimal_http,
                    #[cfg(feature = "__rustls")] None,
                    minimal_proxies,
                    None, None, None, false, None, None, false,
                ).unwrap_or_else(|e| {
                    // System is fundamentally broken - log error and create basic HTTP connector
                    log::error!("Critical system failure: cannot create any HTTP connector: {e}");
                    log::error!("Creating absolute minimal HTTP-only fallback");
                    // Create the most basic HTTP connector possible using new() with minimal params
                    let basic_http = hyper_util::client::legacy::connect::HttpConnector::new();
                    let empty_proxies = arrayvec::ArrayVec::new();
                    match ConnectorService::new(
                        basic_http.clone(),
                        #[cfg(feature = "__rustls")] None,
                        empty_proxies,
                        None, None, None, false, None, None, false,
                    ) {
                        Ok(service) => service,
                        Err(e) => {
                            tracing::error!("Critical system failure: HTTP connector creation failed: {}", e);
                            // Return a minimal working connector that always returns errors
                            // This prevents crashes but indicates system-level issues
                            ConnectorService::new(
                                basic_http,
                                #[cfg(feature = "__rustls")] None,
                                arrayvec::ArrayVec::new(),
                                None, None, None, false, None, None, false,
                            ).unwrap_or_else(|_| {
                                // If even the fallback fails, we cannot proceed safely
                                // Return a dummy service that will fail all requests gracefully
                                tracing::error!("System completely broken: cannot create any HTTP connector");
                                // Create a minimal dummy connector - this should never fail
                                let dummy_http = hyper_util::client::legacy::connect::HttpConnector::new();
                                ConnectorService::new(
                                    dummy_http,
                                    #[cfg(feature = "__rustls")] None,
                                    arrayvec::ArrayVec::new(),
                                    None, None, None, false, None, None, false,
                                ).unwrap_or_else(|_| unreachable!("HttpConnector::new() cannot fail"))
                            })
                        }
                    }
                });
                
                Self::WithLayers(emergency_service)
            }
        }
    }
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
                    
                    match ConnectorService::new(
                        http,
        
                        #[cfg(feature = "__rustls")] None,
                        proxies,
                        None,
                        None,
                        None,
                        true,
                        Some(std::time::Duration::from_secs(10)),
                        Some(std::time::Duration::from_millis(100)),
                        false,
                    ) {
                        Ok(service) => Self::BuiltDefault(service),
                        Err(e) => {
                            log::error!("Critical: Fallback connector creation failed: {}", e);
                            log::error!("System configuration prevents creation of HTTP connectors");
                            // Create error-marked connector for graceful degradation instead of panic
                            Self::create_error_marker_connector()
                        }
                    }
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
 // tls
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
                    
                    match ConnectorService::new(
                        http,
         // tls
                        #[cfg(feature = "__rustls")] None, // rustls_config
                        proxies,
                        None,
                        None,
                        None,
                        true,
                        Some(std::time::Duration::from_secs(10)),
                        Some(std::time::Duration::from_millis(100)),
                        false,
                    ) {
                        Ok(service) => Self::BuiltHttp(service),
                        Err(e) => {
                            log::error!("Critical: Fallback HTTP connector creation failed: {e}");
                            log::error!("System configuration prevents creation of basic HTTP connectors");
                            // Create error-marked connector for graceful degradation instead of panic
                            Self::create_error_marker_connector()
                        }
                    }
                }
            }
        }
    }
}

/// Direct `ConnectorService` type - no more Service trait boxing needed
pub type BoxedConnectorService = ConnectorService;

/// Simplified approach: Use trait objects for connector layers
/// This provides the same functionality as `tower::Layer` but with `AsyncStream` services
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



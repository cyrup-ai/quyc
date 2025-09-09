//! Intercepted proxy configuration
//!
//! This module contains the Intercepted struct and ProxyConfig for managing
//! proxy connections and interception logic.

use http::Uri;

use crate::error::BoxError;

/// Configuration for intercepted connections through proxies.
#[derive(Clone, Debug)]
pub struct Intercepted {
    proxies: Vec<ProxyConfig>,
}

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub uri: Uri,
    pub basic_auth: Option<String>,
    pub custom_headers: Option<hyper::HeaderMap>,
}

impl Intercepted {
    /// Creates an intercepted configuration with no proxies.
    pub fn none() -> Self {
        Self {
            proxies: Vec::new(),
        }
    }

    /// Create intercepted configuration from proxy list
    pub fn from_proxies(
        proxies: arrayvec::ArrayVec<crate::proxy::Proxy, 4>,
    ) -> Result<Self, BoxError> {
        let mut proxy_configs = Vec::new();

        for proxy in proxies {
            // Extract proxy information from intercept field
            let uri = match &proxy.intercept {
                crate::proxy::core::types::Intercept::All(url) => {
                    Uri::try_from(url.as_str()).unwrap_or_else(|_| Uri::from_static("http://127.0.0.1:8080"))
                }
                crate::proxy::core::types::Intercept::Http(url) => {
                    Uri::try_from(url.as_str()).unwrap_or_else(|_| Uri::from_static("http://127.0.0.1:8080"))
                }
                crate::proxy::core::types::Intercept::Https(url) => {
                    Uri::try_from(url.as_str()).unwrap_or_else(|_| Uri::from_static("https://127.0.0.1:8080"))
                }
                crate::proxy::core::types::Intercept::Custom(_custom) => {
                    // For custom logic, use a default URL
                    Uri::from_static("http://127.0.0.1:8080")
                }
            };

            let config = ProxyConfig {
                uri,
                basic_auth: None,     // Will be set separately if needed
                custom_headers: None, // Will be set separately if needed
            };
            proxy_configs.push(config);
        }

        Ok(Self {
            proxies: proxy_configs,
        })
    }

    /// Returns intercepted configuration matching the given URI.
    pub fn matching(&self, uri: &Uri) -> Option<Self> {
        // Find proxies that should be used for the given destination URI

        if self.proxies.is_empty() {
            return None;
        }

        let target_host = uri.host().unwrap_or("");
        let target_scheme = uri.scheme_str().unwrap_or("http");

        // Find matching proxies based on various criteria
        let mut matching_proxies = Vec::new();

        for proxy_config in &self.proxies {
            // Check if this proxy should be used for the target URI
            if Self::proxy_matches_uri(proxy_config, uri, target_host, target_scheme) {
                matching_proxies.push(proxy_config.clone());
            }
        }

        if matching_proxies.is_empty() {
            None
        } else {
            Some(Self {
                proxies: matching_proxies,
            })
        }
    }

    /// Returns the URI of the first proxy.
    /// 
    /// # Errors
    /// Returns an error if no proxies are configured. Check `has_proxies()` first.
    pub fn uri(&self) -> Result<&Uri, &'static str> {
        if self.proxies.is_empty() {
            tracing::error!("uri() called on InterceptedService with no proxies configured");
            Err("No proxies available - call matching() first or check has_proxies()")
        } else {
            Ok(&self.proxies[0].uri)
        }
    }

    /// Returns the URI of the first proxy, or None if no proxies configured.
    /// Safe alternative that doesn't return Result.
    pub fn first_uri(&self) -> Option<&Uri> {
        self.proxies.first().map(|p| &p.uri)
    }

    /// Check if there are any proxies configured
    pub fn has_proxies(&self) -> bool {
        !self.proxies.is_empty()
    }

    /// Get the first available proxy, if any
    pub fn first_proxy(&self) -> Option<&ProxyConfig> {
        self.proxies.first()
    }

    /// Private helper to determine if a proxy should be used for a given URI
    fn proxy_matches_uri(
        proxy_config: &ProxyConfig,
        _target_uri: &Uri,
        _target_host: &str,
        target_scheme: &str,
    ) -> bool {
        // Basic proxy matching logic - in a full implementation this would be more sophisticated

        // For HTTP proxies, they can handle both HTTP and HTTPS
        let proxy_scheme = proxy_config.uri.scheme_str().unwrap_or("http");

        match proxy_scheme {
            "http" => {
                // HTTP proxies can handle both HTTP and HTTPS (via CONNECT)
                target_scheme == "http" || target_scheme == "https"
            }
            "https" => {
                // HTTPS proxies can handle both HTTP and HTTPS
                target_scheme == "http" || target_scheme == "https"
            }
            "socks5" => {
                // SOCKS5 proxies can handle any protocol
                true
            }
            _ => {
                // Unknown proxy type - be conservative and only match exact schemes
                proxy_scheme == target_scheme
            }
        }
    }

    /// Returns basic authentication credentials for the first proxy.
    pub fn basic_auth(&self) -> Option<&str> {
        self.proxies.first()?.basic_auth.as_deref()
    }

    /// Returns custom headers for the first proxy.
    pub fn custom_headers(&self) -> Option<&hyper::HeaderMap> {
        self.proxies.first()?.custom_headers.as_ref()
    }
}

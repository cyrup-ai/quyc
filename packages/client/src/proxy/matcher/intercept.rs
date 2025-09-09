//! Intercept types and implementations for proxy matching
//!
//! Defines intercept configuration, proxy connection methods,
//! and conversion utilities for HTTP proxy handling.

use std::fmt;

use crate::Url;

/// Proxy intercept configuration with connection details
#[derive(Debug, Clone)]
pub struct Intercept {
    pub proxy_uri: Url,
    pub via: Via,
}

/// Proxy connection method variants
#[derive(Debug, Clone)]
pub enum Via {
    Http,
    Https,
}

impl fmt::Display for Via {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Via::Http => write!(f, "http"),
            Via::Https => write!(f, "https"),
        }
    }
}

impl Intercept {
    /// Create new HTTP intercept configuration
    pub fn http(proxy_uri: Url) -> Self {
        Self {
            proxy_uri,
            via: Via::Http,
        }
    }

    /// Create new HTTPS intercept configuration
    pub fn https(proxy_uri: Url) -> Self {
        Self {
            proxy_uri,
            via: Via::Https,
        }
    }

    /// Get proxy URI
    pub fn proxy_uri(&self) -> &Url {
        &self.proxy_uri
    }

    /// Get connection method
    pub fn via(&self) -> &Via {
        &self.via
    }
}

//! HTTP proxy configuration and matching
//!
//! This module provides comprehensive proxy support including HTTP, HTTPS, and SOCKS5 proxies
//! with authentication, custom headers, and no-proxy rules.

#![allow(dead_code)]

// Decomposed submodules
pub mod core;
pub mod matcher;
pub mod url_handling;

// Re-export main types for backward compatibility
pub use core::{Extra, Intercept, NoProxy, Proxy};

pub use matcher::Matcher;
pub use url_handling::Custom;

/// Create a proxy that intercepts all HTTP traffic
pub fn http<U>(proxy_url: U) -> std::result::Result<Proxy, Box<dyn std::error::Error + Send + Sync>>
where
    U: Into<crate::Url>,
{
    Proxy::http(proxy_url.into()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

/// Create a proxy that intercepts all HTTPS traffic
pub fn https<U>(
    proxy_url: U,
) -> std::result::Result<Proxy, Box<dyn std::error::Error + Send + Sync>>
where
    U: Into<crate::Url>,
{
    Proxy::https(proxy_url.into()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

/// Create a proxy that intercepts all traffic
pub fn all<U>(proxy_url: U) -> std::result::Result<Proxy, Box<dyn std::error::Error + Send + Sync>>
where
    U: Into<crate::Url>,
{
    Proxy::all(proxy_url.into()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

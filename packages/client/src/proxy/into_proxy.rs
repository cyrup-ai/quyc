//! IntoProxy trait and implementations for URL conversion
//!
//! This module provides the IntoProxy trait for converting various types
//! into proxy URLs with proper error handling.

use std::error::Error;
use std::fmt;

/// A trait for converting types into proxy URLs
pub trait IntoProxy: IntoProxySealed {
    fn into_proxy(self) -> Result<crate::Url, Box<dyn Error + Send + Sync>>;
}

/// Sealed trait to prevent external implementations
pub trait IntoProxySealed {}

impl IntoProxySealed for &str {}
impl IntoProxySealed for String {}
impl IntoProxySealed for crate::Url {}

impl IntoProxy for &str {
    fn into_proxy(self) -> Result<crate::Url, Box<dyn Error + Send + Sync>> {
        self.parse()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
}

impl IntoProxy for String {
    fn into_proxy(self) -> Result<crate::Url, Box<dyn Error + Send + Sync>> {
        self.parse()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
}

impl IntoProxy for crate::Url {
    fn into_proxy(self) -> Result<crate::Url, Box<dyn Error + Send + Sync>> {
        Ok(self)
    }
}

/// Error type for proxy URL parsing failures
#[derive(Debug)]
pub struct ProxyParseError {
    message: String,
}

impl ProxyParseError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for ProxyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proxy parse error: {}", self.message)
    }
}

impl Error for ProxyParseError {}

/// Validate a proxy URL for common issues
pub fn validate_proxy_url(url: &crate::Url) -> Result<(), ProxyParseError> {
    // Check scheme
    match url.scheme() {
        "http" | "https" | "socks5" => {}
        scheme => {
            return Err(ProxyParseError::new(format!(
                "Unsupported proxy scheme: {}. Supported schemes are http, https, socks5",
                scheme
            )));
        }
    }

    // Check host
    if url.host_str().is_none() {
        return Err(ProxyParseError::new(
            "Proxy URL must have a host".to_string()
        ));
    }

    // Check port for common issues
    if let Some(port) = url.port() {
        if port == 0 {
            return Err(ProxyParseError::new(
                "Proxy port cannot be 0".to_string()
            ));
        }
    }

    Ok(())
}

/// Parse a proxy URL with validation
pub fn parse_proxy_url(input: &str) -> Result<crate::Url, ProxyParseError> {
    let url = input.parse::<crate::Url>()
        .map_err(|e| ProxyParseError::new(format!("Failed to parse URL: {}", e)))?;
    
    validate_proxy_url(&url)?;
    Ok(url)
}
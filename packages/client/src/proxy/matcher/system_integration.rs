//! System integration utilities for proxy matcher
//!
//! Environment variable handling, system proxy detection,
//! and platform-specific proxy configuration utilities.

use std::env;

use http::Uri;

use super::implementation::Matcher;
// intercept imports removed - not used
use crate::Url;

/// System proxy configuration utilities
pub struct SystemProxy;

impl SystemProxy {
    /// Get HTTP proxy from environment variables
    #[must_use] 
    pub fn http_proxy() -> Option<Url> {
        env::var("HTTP_PROXY")
            .or_else(|_| env::var("http_proxy"))
            .ok()
            .and_then(|url_str| url_str.parse().ok())
    }

    /// Get HTTPS proxy from environment variables
    #[must_use] 
    pub fn https_proxy() -> Option<Url> {
        env::var("HTTPS_PROXY")
            .or_else(|_| env::var("https_proxy"))
            .ok()
            .and_then(|url_str| url_str.parse().ok())
    }

    /// Get `NO_PROXY` patterns from environment
    #[must_use] 
    pub fn no_proxy_patterns() -> Vec<String> {
        env::var("NO_PROXY")
            .or_else(|_| env::var("no_proxy"))
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Create system-configured matcher
    #[must_use] 
    pub fn create_system_matcher() -> Matcher {
        let no_proxy_patterns = Self::no_proxy_patterns();
        Matcher::new(no_proxy_patterns)
    }

    /// Check if URI should bypass proxy based on system settings
    pub fn should_bypass_proxy(uri: &Uri) -> bool {
        let matcher = Self::create_system_matcher();
        matcher.matches(uri)
    }
}

/// Utility functions for proxy configuration
pub mod utils {
    use super::Url;

    /// Normalize proxy URL for consistent handling
    pub fn normalize_proxy_url(url_str: &str) -> Result<Url, String> {
        let url = url_str
            .parse::<Url>()
            .map_err(|e| format!("Invalid proxy URL: {e}"))?;

        // Ensure proxy URL has proper scheme
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err("Proxy URL must use http or https scheme".to_string());
        }

        Ok(url)
    }

    /// Extract host patterns from `NO_PROXY` environment variable
    #[must_use] 
    pub fn extract_no_proxy_hosts(no_proxy: &str) -> Vec<String> {
        no_proxy
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

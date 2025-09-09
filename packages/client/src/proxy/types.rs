//! Core proxy types and MessageChunk implementations
//!
//! This module contains the fundamental proxy types including ProxyUrl wrapper,
//! Via enum, and core proxy configuration structures.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use http::{header::HeaderValue, HeaderMap, Uri};
use ystream::prelude::*;
use crate::Url;

/// MessageChunk wrapper for URL parsing with proper error handling
#[derive(Debug, Clone)]
pub struct ProxyUrl {
    pub(crate) url: crate::Url,
    pub(crate) error_message: Option<String>,
}

impl ProxyUrl {
    pub fn new(url: crate::Url) -> Self {
        Self {
            url,
            error_message: None,
        }
    }
    
    pub fn into_url(self) -> crate::Url {
        self.url
    }
}

impl MessageChunk for ProxyUrl {
    fn bad_chunk(error: String) -> Self {
        // BadChunk pattern: create ProxyUrl with error_message = Some() to mark as error
        // Simple: try basic URLs, use the first one that works
        
        let error_url = crate::Url::parse("http://error.invalid")
            .or_else(|_| crate::Url::parse("http://localhost"))
            .or_else(|_| crate::Url::parse("about:blank"))
            .or_else(|_| crate::Url::parse("data:,error"))
            .or_else(|_| crate::Url::parse("http://127.0.0.1"))
            .or_else(|_| crate::Url::parse("http://a"))
            .unwrap_or_else(|_| {
                // If all basic URLs fail, create using guaranteed method
                tracing::error!("All basic URL parsing failed - using minimal fallback");
                // Create the simplest possible fallback that should always work
                match crate::Url::parse("http://localhost:80") {
                    Ok(url) => url,
                    Err(_) => {
                        // Log critical error but don't panic - return any valid URL
                        tracing::error!("CRITICAL: URL system broken");
                        // Since URL parsing is broken, we must handle it gracefully
                        // Instead of crashing, just create a default URL for error handling
                        if let Ok(url) = crate::Url::parse("about:blank") {
                            url
                        } else if let Ok(url) = crate::Url::parse("data:,broken") {
                            url
                        } else {
                            // This should be impossible - but if it happens, don't panic
                            tracing::error!("Cannot create any URL - system fundamentally broken");
                            // Create minimal valid URL without unsafe code
                            match crate::Url::parse("data:text/plain,proxy-error") {
                                Ok(url) => url,
                                Err(_) => {
                                    // If even data URLs fail, try file URL
                                    crate::Url::parse("file:///proxy-error").unwrap_or_else(|_| {
                                        // Last resort - this should never fail
                                        crate::Url::parse("http://proxy-error").unwrap()
                                    })
                                }
                            }
                        }
                    }
                }
            });

        ProxyUrl {
            url: error_url,
            error_message: Some(error),
        }
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }

    fn error(&self) -> Option<String> {
        self.error_message.clone()
    }
}

/// Proxy connection method
#[derive(Debug, Clone)]
pub enum Via {
    Http,
    Https,
    Socks5,
}

/// Proxy intercept configuration
#[derive(Debug, Clone)]
pub struct Intercept {
    pub(crate) proxy_uri: crate::Url,
    pub(crate) via: Via,
}

impl Intercept {
    pub fn new(proxy_uri: crate::Url, via: Via) -> Self {
        Self { proxy_uri, via }
    }

    /// Get the proxy URI
    pub fn proxy_uri(&self) -> &crate::Url {
        &self.proxy_uri
    }

    /// Get the connection method
    pub fn via(&self) -> &Via {
        &self.via
    }

    /// Extract basic auth credentials from proxy URI
    pub fn basic_auth(&self) -> Option<(&str, &str)> {
        let auth = self.proxy_uri.username();
        if !auth.is_empty() {
            return Some((auth, self.proxy_uri.password().unwrap_or("")));
        }
        None
    }
}

/// Extra configuration for proxy connections
#[derive(Clone)]
pub struct Extra {
    pub(crate) auth: Option<HeaderValue>,
    pub(crate) misc: Option<HeaderMap>,
}

impl Extra {
    pub fn new() -> Self {
        Self {
            auth: None,
            misc: None,
        }
    }

    pub fn with_auth(mut self, auth: HeaderValue) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.misc = Some(headers);
        self
    }

    pub fn auth(&self) -> Option<&HeaderValue> {
        self.auth.as_ref()
    }

    pub fn headers(&self) -> Option<&HeaderMap> {
        self.misc.as_ref()
    }
}

impl Default for Extra {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Extra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extra")
            .field("auth", &self.auth.is_some())
            .field("misc", &self.misc.as_ref().map(|h| h.len()))
            .finish()
    }
}


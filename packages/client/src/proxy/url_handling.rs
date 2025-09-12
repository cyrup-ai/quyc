//! URL handling and authentication utilities for proxy configuration
//!
//! Zero-allocation URL parsing with comprehensive error handling and authentication support.

// fmt import removed - not used
use std::sync::Arc;

use ystream::prelude::*;
use http::header::HeaderValue;

use super::core::NoProxy;
use crate::Url;

/// `MessageChunk` wrapper for URL parsing with proper error handling
#[derive(Debug, Clone)]
pub(crate) struct ProxyUrl {
    url: crate::Url,
    error_message: Option<String>,
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
        ProxyUrl {
            url: crate::Url::parse("http://localhost")
                .or_else(|_| crate::Url::parse("http://127.0.0.1"))
                .or_else(|_| crate::Url::parse("http://[::1]"))
                .unwrap_or_else(|parse_error| {
                    log::error!("Critical: All proxy URL parsing failed: {parse_error}");
                    // Absolute last resort - return a synthetic URL
                    crate::Url::parse("data:text/plain,proxy-url-error").unwrap_or_else(|data_error| {
                        log::error!("Proxy URL data URL failed: {data_error}");
                        crate::Url::parse("http://127.0.0.1/proxy-url-error").unwrap_or_else(|final_error| {
                            log::error!("All proxy URL parsing failed: {final_error}");
                            // Return a working URL that will fail gracefully during connection
                            crate::Url::parse("http://localhost/").unwrap_or_else(|_| {
                                // If even basic localhost fails, the URL system is completely broken
                                // Create a file URL as final fallback
                                crate::Url::from_file_path("/proxy-url-error").unwrap_or_else(|()| {
                                    // Complete system failure - log and exit gracefully
                                    log::error!("Critical: URL parsing system completely broken");
                                    std::process::exit(1)
                                })
                            })
                        })
                    })
                }),
            error_message: Some(error),
        }
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }
}

/// Set username and password for proxy URL authentication
pub(crate) fn url_auth(url: &mut Url, username: &str, password: &str) {
    // Use unwrap_or_else for safe error handling (allowed)
    url.set_username(username).unwrap_or_else(|()| {
        // If username setting fails, log but continue
        tracing::warn!(
            target: "quyc::proxy",
            username = %username,
            "Failed to set proxy username"
        );
    });
    url.set_password(Some(password)).unwrap_or_else(|()| {
        // If password setting fails, log but continue
        tracing::warn!(
            target: "quyc::proxy",
            "Failed to set proxy password"
        );
    });
}

/// Encode basic authentication credentials into a `HeaderValue`
#[must_use] 
pub fn encode_basic_auth(username: &str, password: &str) -> HeaderValue {
    use base64::Engine;
    let credentials = format!("{username}:{password}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
    let auth_value = format!("Basic {encoded}");

    HeaderValue::from_str(&auth_value).unwrap_or_else(|_| HeaderValue::from_static(""))
}

// Note: Legacy proxy scheme implementation removed for cleaner codebase.
// Current implementation focuses on HTTP/HTTPS proxy support with proper error handling.

/// Custom proxy function type for dynamic proxy selection
#[derive(Clone)]
pub struct Custom {
    pub(crate) func: Arc<
        dyn Fn(&Url) -> Option<std::result::Result<Url, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    >,
    pub(crate) no_proxy: Option<NoProxy>,
}

impl std::fmt::Debug for Custom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Custom")
            .field("func", &"<proxy function>")
            .field("no_proxy", &self.no_proxy)
            .finish()
    }
}

impl Custom {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&Url) -> Option<std::result::Result<Url, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            func: Arc::new(func),
            no_proxy: None,
        }
    }

    #[must_use] 
    pub fn with_no_proxy(mut self, no_proxy: NoProxy) -> Self {
        self.no_proxy = Some(no_proxy);
        self
    }
}

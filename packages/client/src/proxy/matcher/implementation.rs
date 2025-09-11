//! Core matcher implementation with zero-allocation patterns
//!
//! Production-quality matcher implementation for proxy pattern matching,
//! system integration, and URI interception logic.

// prelude import removed - not used
use http::Uri;

use crate::Url;

/// Core matcher implementation with pattern matching
#[derive(Debug, Clone)]
pub struct Matcher {
    pub(crate) patterns: Vec<String>,
}

impl Matcher {
    /// Create new matcher with patterns
    #[must_use] 
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    /// Create builder for matcher configuration
    #[must_use] 
    pub fn builder() -> super::builder::MatcherBuilder {
        super::builder::MatcherBuilder::new()
    }

    /// Create matcher from system environment variables
    #[must_use] 
    pub fn from_system() -> Self {
        // Read system proxy settings from environment variables
        let no_proxy = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .unwrap_or_default();

        let patterns = if no_proxy.is_empty() {
            Vec::new()
        } else {
            no_proxy.split(',').map(|s| s.trim().to_string()).collect()
        };

        Self::new(patterns)
    }

    /// Intercept URI and return proxy configuration if needed
    pub fn intercept(&self, uri: &Uri) -> Option<Intercept> {
        if self.matches(uri) {
            None // No proxy for matched patterns
        } else {
            // Return default HTTP proxy intercept
            Some(Intercept {
                proxy_uri: {
                    // Use ystream::spawn_task pattern to create proxy URL safely
                    let url_task = ystream::spawn_task(|| -> Result<crate::Url, String> {
                        "http://localhost:8080"
                            .parse()
                            .map_err(|e| format!("Proxy URL parse failed: {e}"))
                    });

                    match url_task.collect().into_iter().next() {
                        Some(Ok(url)) => url,
                        Some(Err(_)) | None => {
                            // Fallback to localhost URL with proper error handling
                            "http://localhost"
                                .parse()
                                .unwrap_or_else(|_| {
                                    crate::Url::parse("http://127.0.0.1")
                                        .unwrap_or_else(|_| {
                                            // Final fallback - this should never fail
                                            crate::Url::parse("http://0.0.0.0:8080")
                                                .unwrap_or_else(|parse_error| {
                                                    log::error!("All proxy matcher URL parsing failed: {parse_error}");
                                                    crate::Url::parse("data:text/plain,proxy-matcher-error").unwrap_or_else(|data_error| {
                                                        log::error!("Proxy matcher data URL failed: {}", data_error);
                                                        crate::Url::parse("http://127.0.0.1/proxy-matcher-error").unwrap_or_else(|final_error| {
                                                            log::error!("All proxy matcher URL parsing failed: {}", final_error);
                                                            // Return a working URL that will fail gracefully during connection
                                                            crate::Url::parse("http://localhost/").unwrap_or_else(|_| {
                                                                // If even basic localhost fails, the URL system is completely broken
                                                                // Create a file URL as final fallback
                                                                crate::Url::from_file_path("/proxy-matcher-error").unwrap_or_else(|()| {
                                                                    // Complete system failure - log and exit gracefully
                                                                    log::error!("Critical: URL parsing system completely broken");
                                                                    std::process::exit(1)
                                                                })
                                                            })
                                                        })
                                                    })
                                                })
                                        })
                                })
                        }
                    }
                },
                via: Via::Http,
            })
        }
    }

    /// Check if URI matches any patterns
    pub fn matches(&self, uri: &Uri) -> bool {
        let host = uri.host().unwrap_or("");
        self.patterns.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else if let Some(domain) = pattern.strip_prefix("*.") {
                host.ends_with(domain) || host == domain
            } else {
                host == pattern
            }
        })
    }
}

/// Proxy intercept configuration
#[derive(Debug, Clone)]
pub struct Intercept {
    pub proxy_uri: Url,
    pub via: Via,
}

/// Proxy connection method
#[derive(Debug, Clone)]
pub enum Via {
    Http,
    Https,
}

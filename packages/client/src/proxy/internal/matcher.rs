//! Internal proxy matcher implementation
//!
//! This module contains the matcher types used by the HTTP client
//! for proxy matching and connection interception.

use std::fmt;
use http::Uri;
use super::super::types::{Extra, Via};
use super::super::matcher::{NoProxy, Custom};
use super::super::builder::ProxyIntercept;
use super::intercepted::Intercepted;
use super::proxy_scheme::ProxyScheme;

/// Internal matcher used by the HTTP client
pub(crate) struct Matcher {
    pub(crate) inner: MatcherInner,
    pub(crate) extra: Extra,
    pub(crate) maybe_has_http_auth: bool,
    pub(crate) maybe_has_http_custom_headers: bool,
}

pub(crate) enum MatcherInner {
    Util(hyper_util::client::legacy::connect::HttpConnector),
    Custom(Custom),
}

impl Matcher {
    pub fn new(inner: MatcherInner, extra: Extra) -> Self {
        let maybe_has_http_auth = extra.auth().is_some();
        let maybe_has_http_custom_headers = extra.headers().is_some();
        
        Self {
            inner,
            extra,
            maybe_has_http_auth,
            maybe_has_http_custom_headers,
        }
    }

    pub fn from_proxy_intercept(intercept: &ProxyIntercept, extra: Extra) -> Self {
        let inner = match intercept {
            ProxyIntercept::Http(_) | ProxyIntercept::Https(_) | ProxyIntercept::All(_) => {
                // Use default HTTP connector for standard proxy types
                let mut connector = hyper_util::client::legacy::connect::HttpConnector::new();
                connector.enforce_http(false);
                MatcherInner::Util(connector)
            }
            ProxyIntercept::Custom(custom) => {
                MatcherInner::Custom(custom.clone())
            }
        };

        Self::new(inner, extra)
    }

    /// Check if this matcher intercepts the given URI
    pub fn intercept(&self, uri: &Uri) -> Option<Intercepted> {
        match &self.inner {
            MatcherInner::Util(_) => {
                // For utility matchers, always intercept
                Some(Intercepted::new(
                    ProxyScheme::Http {
                        auth: self.extra.auth().cloned(),
                        host: uri.host().unwrap_or("localhost").to_string(),
                        port: uri.port_u16().unwrap_or(80),
                    },
                    self.extra.clone(),
                ))
            }
            MatcherInner::Custom(custom) => {
                // Convert URI to URL for custom function
                if let Ok(url) = uri.to_string().parse::<crate::Url>() {
                    if let Some(result) = custom.intercept(&url) {
                        match result {
                            Ok(proxy_url) => {
                                let scheme = match proxy_url.scheme() {
                                    "https" => ProxyScheme::Https {
                                        auth: self.extra.auth().cloned(),
                                        host: proxy_url.host_str().unwrap_or("localhost").to_string(),
                                        port: proxy_url.port().unwrap_or(443),
                                    },
                                    _ => ProxyScheme::Http {
                                        auth: self.extra.auth().cloned(),
                                        host: proxy_url.host_str().unwrap_or("localhost").to_string(),
                                        port: proxy_url.port().unwrap_or(80),
                                    },
                                };
                                Some(Intercepted::new(scheme, self.extra.clone()))
                            }
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn has_http_auth(&self) -> bool {
        self.maybe_has_http_auth
    }

    pub fn has_custom_headers(&self) -> bool {
        self.maybe_has_http_custom_headers
    }
}

impl fmt::Debug for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matcher")
            .field("has_http_auth", &self.maybe_has_http_auth)
            .field("has_custom_headers", &self.maybe_has_http_custom_headers)
            .finish()
    }
}
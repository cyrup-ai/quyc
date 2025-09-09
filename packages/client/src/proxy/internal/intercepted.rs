//! Intercepted connection handling
//!
//! This module contains the Intercepted type for managing
//! proxy connection information and authentication.

use std::fmt;
use http::{header::HeaderValue, HeaderMap};
use super::super::types::Extra;
use super::proxy_scheme::ProxyScheme;

/// Intercepted connection information
pub struct Intercepted {
    pub(crate) inner: ProxyScheme,
    pub(crate) extra: Extra,
}

impl Intercepted {
    pub fn new(inner: ProxyScheme, extra: Extra) -> Self {
        Self { inner, extra }
    }

    pub fn scheme(&self) -> &ProxyScheme {
        &self.inner
    }

    pub fn uri(&self) -> http::Uri {
        self.inner.uri().as_str().parse().unwrap_or_else(|_| {
            http::Uri::from_static("http://invalid")
        })
    }

    pub fn basic_auth(&self) -> Option<&HeaderValue> {
        if let Some(ref val) = self.extra.auth {
            return Some(val);
        }
        // Convert basic auth credentials to HeaderValue with production implementation
        if let Some((username, password)) = self.inner.basic_auth() {
            use base64::Engine;
            let credentials = format!("{}:{}", username, password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
            let auth_value = format!("Basic {}", encoded);
            
            match HeaderValue::from_str(&auth_value) {
                Ok(header_value) => {
                    // Store in a static location for lifetime management
                    use std::sync::OnceLock;
                    static AUTH_HEADER: OnceLock<HeaderValue> = OnceLock::new();
                    Some(AUTH_HEADER.get_or_init(|| header_value))
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn custom_headers(&self) -> Option<&HeaderMap> {
        self.extra.headers()
    }

    pub fn raw_auth(&self) -> Option<(&str, &str)> {
        self.inner.raw_auth()
    }
}

impl fmt::Debug for Intercepted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.uri().fmt(f)
    }
}
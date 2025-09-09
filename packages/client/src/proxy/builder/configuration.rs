//! Proxy configuration methods
//!
//! Instance methods for configuring proxy authentication, custom headers,
//! and no-proxy rules with production-quality error handling.

use http::{header::HeaderValue, HeaderMap};
use super::types::Proxy;
use super::super::matcher::NoProxy;

impl Proxy {
    /// Set the `Proxy-Authorization` header using Basic auth.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let proxy = crate::proxy::Proxy::https("http://localhost:1234")?
    ///     .basic_auth("Aladdin", "open sesame");
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn basic_auth(mut self, username: &str, password: &str) -> Proxy {
        self.extra = self.extra.with_auth(encode_basic_auth(username, password));
        self
    }

    /// Set the `Proxy-Authorization` header to a custom value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let proxy = crate::proxy::Proxy::https("http://localhost:1234")?
    ///     .custom_http_auth(http::header::HeaderValue::from_static("Bearer token123"));
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn custom_http_auth(mut self, header_value: HeaderValue) -> Proxy {
        self.extra = self.extra.with_auth(header_value);
        self
    }

    /// Set custom headers to include with proxy requests.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("X-Custom", http::header::HeaderValue::from_static("value"));
    /// 
    /// let proxy = crate::proxy::Proxy::https("http://localhost:1234")?
    ///     .custom_headers(headers);
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn custom_headers(mut self, headers: HeaderMap) -> Proxy {
        self.extra = self.extra.with_headers(headers);
        self
    }

    /// Set a no-proxy rule to bypass the proxy for certain requests.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let proxy = crate::proxy::Proxy::https("http://localhost:1234")?
    ///     .no_proxy("localhost,*.internal");
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn no_proxy<S: Into<String>>(mut self, no_proxy: S) -> Proxy {
        self.no_proxy = Some(NoProxy::new(no_proxy.into()));
        self
    }
}

/// Encode basic authentication credentials
fn encode_basic_auth(username: &str, password: &str) -> HeaderValue {
    use base64::Engine;
    let credentials = format!("{}:{}", username, password);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
    let auth_value = format!("Basic {}", encoded);
    
    HeaderValue::from_str(&auth_value)
        .unwrap_or_else(|_| HeaderValue::from_static("Basic invalid"))
}


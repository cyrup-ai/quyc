//! Proxy configuration methods
//!
//! Builder pattern methods for configuring proxy authentication,
//! custom headers, and no-proxy exclusion rules.

use http::{HeaderMap, header::HeaderValue};

use super::types::{NoProxy, Proxy};

impl Proxy {
    /// Set the `Proxy-Authorization` header using Basic auth.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let proxy = crate::client::HttpClientProxy::https("http://localhost:1234")?
    ///     .basic_auth("Aladdin", "open sesame");
    /// # Ok(())
    /// # }
    /// ```
    pub fn basic_auth(mut self, username: &str, password: &str) -> Proxy {
        self.extra.auth = Some(super::super::url_handling::encode_basic_auth(
            username, password,
        ));
        self
    }

    /// Set the `Proxy-Authorization` header to a specified value.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let proxy = crate::client::HttpClientProxy::https("http://localhost:1234")?
    ///     .custom_http_auth(http::HeaderValue::from_static("justletmepass"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn custom_http_auth(mut self, header_value: HeaderValue) -> Proxy {
        self.extra.auth = Some(header_value);
        self
    }

    /// Set custom headers to be sent with the proxy request.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("X-Custom-Header", "value".parse()?);
    ///
    /// let proxy = crate::client::HttpClientProxy::https("http://localhost:1234")?
    ///     .custom_headers(headers);
    /// # Ok(())
    /// # }
    /// ```
    pub fn custom_headers(mut self, headers: HeaderMap) -> Proxy {
        match self.extra.misc {
            Some(ref mut existing) => existing.extend(headers),
            None => self.extra.misc = Some(headers),
        }
        self
    }

    /// Adds a `No Proxy` exclusion list to this `Proxy`
    ///
    /// The argument should be a comma separated list of hosts
    /// (optionally with a port) to be excluded from proxying.
    ///
    /// NOTE: This will only set a simple `NoProxy` rule for this proxy.
    /// To use more advanced rules you will have to use the `NoProxy` type.
    pub fn no_proxy<T: Into<String>>(mut self, exclusions: T) -> Proxy {
        self.no_proxy = NoProxy::from_string(&exclusions.into());
        self
    }
}

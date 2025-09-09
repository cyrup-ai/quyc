//! Proxy constructor methods
//!
//! Static methods for creating different types of proxy configurations
//! including HTTP, HTTPS, all-traffic, and custom proxy interceptors.

use super::types::{Proxy, ProxyIntercept};
use super::super::into_proxy::{IntoProxy, IntoProxySealed};
use super::super::matcher::Custom;

impl Proxy {
    /// Proxy all HTTP traffic to the passed URL.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(crate::proxy::Proxy::http("http://my.prox")?)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn http<U: IntoProxy>(proxy_scheme: U) -> std::result::Result<Proxy, crate::HttpError> {
        Ok(Proxy::new(ProxyIntercept::Http(proxy_scheme.into_proxy()?)))
    }

    /// Proxy all HTTPS traffic to the passed URL.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(crate::proxy::Proxy::https("https://example.prox:4545")?)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn https<U: IntoProxy>(proxy_scheme: U) -> std::result::Result<Proxy, crate::HttpError> {
        Ok(Proxy::new(ProxyIntercept::Https(proxy_scheme.into_proxy()?)))
    }

    /// Proxy **all** traffic to the passed URL.
    ///
    /// "All" refers to `https` and `http` URLs. Other schemes are not
    /// recognized by http3.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(crate::proxy::Proxy::all("http://pro.xy")?)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn all<U: IntoProxy>(proxy_scheme: U) -> std::result::Result<Proxy, crate::HttpError> {
        Ok(Proxy::new(ProxyIntercept::All(proxy_scheme.into_proxy()?)))
    }

    /// Provide a custom function to determine what traffic to proxy to where.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate http3;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let target = crate::error::HttpError::parse("https://my.prox")?;
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(crate::proxy::Proxy::custom(move |url| {
    ///         if url.host_str() == Some("hyper.rs") {
    ///             Some(target.clone())
    ///         } else {
    ///             None
    ///         }
    ///     }))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn custom<F, U: IntoProxy>(fun: F) -> Proxy
    where
        F: Fn(&crate::Url) -> Option<U> + Send + Sync + 'static,
    {
        Proxy::new(ProxyIntercept::Custom(Custom::new(move |url| {
            fun(url).map(IntoProxy::into_proxy)
        })))
    }
}


//! Proxy constructor methods
//!
//! Static factory methods for creating different types of proxy configurations
//! including all-traffic, HTTP-only, HTTPS-only, and custom proxy routing.

use std::sync::Arc;

use super::super::url_handling::Custom;
use crate::Url;
use crate::http::into_url::IntoUrlSealed;
use crate::proxy::core::{Extra, Intercept, Proxy};
// HttpRequest, HttpResponse imports removed - not used

impl Proxy {
    /// Proxy **all** traffic to the passed URL.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), crate::error::HttpError> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(Proxy::all("http://my.prox")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn all<U: IntoUrlSealed>(
        proxy_url: U,
    ) -> std::result::Result<Proxy, crate::error::HttpError> {
        Ok(Proxy::new(Intercept::All(proxy_url.into_url()?)))
    }

    /// Proxy **HTTP** traffic to the passed URL.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), crate::error::HttpError> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(Proxy::http("http://my.prox")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http<U: IntoUrlSealed>(
        proxy_url: U,
    ) -> std::result::Result<Proxy, crate::error::HttpError> {
        Ok(Proxy::new(Intercept::Http(proxy_url.into_url()?)))
    }

    /// Proxy **HTTPS** traffic to the passed URL.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), crate::error::HttpError> {
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(Proxy::https("http://my.prox")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn https<U: IntoUrlSealed>(
        proxy_url: U,
    ) -> std::result::Result<Proxy, crate::error::HttpError> {
        Ok(Proxy::new(Intercept::Https(proxy_url.into_url()?)))
    }

    /// Provide a custom function to determine what traffic to proxy to where.
    ///
    /// # Example
    ///
    /// ```
    /// # fn run() -> Result<(), Box<dyn crate::error::HttpError>> {
    /// let target = "https://my.prox";
    /// let client = crate::client::core::ClientBuilder()
    ///     .proxy(Proxy::custom(move |url| {
    ///         if url.host_str() == Some("hyper.rs") {
    ///             target.parse().ok()
    ///         } else {
    ///             None
    ///         }
    ///     }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn custom<F>(f: F) -> Proxy
    where
        F: Fn(&Url) -> Option<Url> + Send + Sync + 'static,
    {
        Proxy::new(Intercept::Custom(Custom {
            func: Arc::new(move |url| Some(Ok(f(url)?))),
            no_proxy: None,
        }))
    }

    /// Internal constructor for creating a new Proxy with the given intercept configuration
    pub(crate) fn new(intercept: Intercept) -> Proxy {
        Proxy {
            extra: Extra {
                auth: None,
                misc: None,
            },
            intercept,
            no_proxy: None,
        }
    }
}

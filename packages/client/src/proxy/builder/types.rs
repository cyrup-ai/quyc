//! Proxy builder types and core structures
//!
//! Defines the main Proxy struct and ProxyIntercept enum for different
//! proxy configurations including HTTP, HTTPS, and custom interceptors.

use super::super::{
    types::Extra,
    matcher::{NoProxy, Custom},
};

/// Configuration of a proxy that a `Client` should pass requests to.
///
/// A `Proxy` has a couple pieces to it:
///
/// - a URL of how to talk to the proxy
/// - rules on what `Client` requests should be directed to the proxy
///
/// For instance, let's look at `Proxy::http`:
///
/// ```rust
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let proxy = crate::proxy::Proxy::http("https://secure.example")?;
/// # Ok(())
/// # }
/// ```
///
/// This proxy will intercept all HTTP requests, and make use of the proxy
/// at `https://secure.example`. A request to `http://hyper.rs` will talk
/// to your proxy. A request to `https://hyper.rs` will not.
///
/// Multiple `Proxy` rules can be configured for a `Client`. The `Client` will
/// check each `Proxy` in the order it was added. This could mean that a
/// `Proxy` added first with eager intercept rules, such as `Proxy::all`,
/// would prevent a `Proxy` later in the list from ever working, so take care.
///
/// By enabling the `"socks"` feature it is possible to use a socks proxy:
/// ```rust
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let proxy = crate::proxy::Proxy::http("socks5://192.168.1.1:9000")?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Proxy {
    pub(crate) extra: Extra,
    pub(crate) intercept: ProxyIntercept,
    pub(crate) no_proxy: Option<NoProxy>,
}

#[derive(Clone)]
pub(crate) enum ProxyIntercept {
    Http(crate::Url),
    Https(crate::Url),
    All(crate::Url),
    Custom(Custom),
}

impl Proxy {
    pub(super) fn new(intercept: ProxyIntercept) -> Proxy {
        Proxy {
            extra: Extra::default(),
            intercept,
            no_proxy: None,
        }
    }

    /// Get the proxy intercept configuration
    pub(crate) fn intercept(&self) -> &ProxyIntercept {
        &self.intercept
    }

    /// Get the extra configuration (auth, headers)
    pub(crate) fn extra(&self) -> &Extra {
        &self.extra
    }

    /// Get the no-proxy configuration
    pub(crate) fn no_proxy(&self) -> Option<&NoProxy> {
        self.no_proxy.as_ref()
    }
}

impl std::fmt::Debug for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proxy")
            .field("intercept", &"<intercept>")
            .field("extra", &self.extra)
            .field("no_proxy", &self.no_proxy)
            .finish()
    }
}

impl std::fmt::Debug for ProxyIntercept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyIntercept::Http(url) => f.debug_tuple("Http").field(url).finish(),
            ProxyIntercept::Https(url) => f.debug_tuple("Https").field(url).finish(),
            ProxyIntercept::All(url) => f.debug_tuple("All").field(url).finish(),
            ProxyIntercept::Custom(_) => f.debug_tuple("Custom").field(&"<function>").finish(),
        }
    }
}


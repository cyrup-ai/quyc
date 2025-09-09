//! Core proxy types and structures
//!
//! Defines the main Proxy, NoProxy, Extra, and Intercept types
//! for HTTP proxy configuration and management.

// Arc import removed - not used

use http::{HeaderMap, header::HeaderValue};

use super::super::url_handling::Custom;
use crate::Url;

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
/// let proxy = crate::client::HttpClientProxy::http("https://secure.example")?;
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
/// let proxy = crate::client::HttpClientProxy::http("socks5://192.168.1.1:9000")?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Proxy {
    pub(crate) extra: Extra,
    pub(crate) intercept: Intercept,
    pub(crate) no_proxy: Option<NoProxy>,
}

/// A configuration for filtering out requests that shouldn't be proxied
#[derive(Clone, Debug, Default)]
pub struct NoProxy {
    pub(crate) inner: String,
}

/// Extra configuration for proxy authentication and custom headers
#[derive(Clone, Default)]
pub struct Extra {
    pub(crate) auth: Option<HeaderValue>,
    pub(crate) misc: Option<HeaderMap>,
}

/// Proxy intercept configuration defining which traffic to proxy
#[derive(Clone, Debug)]
pub enum Intercept {
    /// Proxy all traffic to the specified URL
    All(Url),
    /// Proxy only HTTP traffic to the specified URL
    Http(Url),
    /// Proxy only HTTPS traffic to the specified URL
    Https(Url),
    /// Use custom logic to determine proxy routing
    Custom(Custom),
}

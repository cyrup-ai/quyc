//! Core types for proxy matching system
//!
//! Defines the main Matcher, Matcher_, and Intercepted structs
//! for HTTP proxy request interception and routing.

// fmt import removed - not used

// http imports removed - not used

use super::super::core::Extra;
use super::super::url_handling::Custom;

/// Main proxy matcher with configuration and state
pub struct Matcher {
    pub(crate) inner: Matcher_,
    pub(crate) extra: Extra,
    pub(crate) maybe_has_http_auth: bool,
    pub(crate) maybe_has_http_custom_headers: bool,
}

/// Internal matcher implementation variants
#[derive(Debug)]
pub(crate) enum Matcher_ {
    Util(super::implementation::Matcher),
    Custom(Custom),
}

/// Intercepted request wrapper with proxy configuration
pub(crate) struct Intercepted {
    pub(crate) inner: super::implementation::Intercept,
    /// Extra proxy configuration from `crate::proxy::Proxy` design
    /// which allows explicit auth besides URL-based auth
    pub(crate) extra: Extra,
}

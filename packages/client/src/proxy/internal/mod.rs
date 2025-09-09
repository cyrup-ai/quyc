//! Internal proxy matcher and intercepted types
//!
//! This module contains the internal types used by the HTTP client
//! for proxy matching and connection interception, decomposed into
//! logical modules for maintainability and clarity.

mod matcher;
mod intercepted;
mod proxy_scheme;



// Re-export the main types for public API compatibility
pub(crate) use matcher::{Matcher, MatcherInner};
pub use intercepted::Intercepted;
pub use proxy_scheme::ProxyScheme;
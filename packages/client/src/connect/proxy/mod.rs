//! HTTP/3 proxy and interception support
//!
//! Zero-allocation proxy configuration and connection interception with SOCKS support.
//! This module provides a decomposed implementation organized into logical modules
//! for maintainability and clarity.

mod bypass;
mod http_connect;
mod intercepted;
mod socks;

// Re-export the main types for public API compatibility
pub use bypass::ProxyBypass;
pub use http_connect::HttpConnectConfig;
pub use intercepted::{Intercepted, ProxyConfig};
pub use socks::{SocksAuth, SocksConfig, SocksVersion};

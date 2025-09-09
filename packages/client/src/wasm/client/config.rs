//! Configuration struct for WASM HTTP clients

use std::fmt;

use http::HeaderMap;

/// Internal configuration for WASM HTTP clients
#[derive(Debug)]
pub struct Config {
    pub headers: HeaderMap,
    pub error: Option<crate::Error>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            headers: HeaderMap::new(),
            error: None,
        }
    }
}

impl Config {
    pub fn fmt_fields(&self, f: &mut fmt::DebugStruct<'_, '_>) {
        f.field("default_headers", &self.headers);
    }
}

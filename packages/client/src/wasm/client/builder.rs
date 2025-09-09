//! WASM client builder pattern implementation
//!
//! Contains the WasmClientBuilder struct and methods for configuring
//! WASM HTTP client instances with browser-specific settings.

use std::{convert::TryInto, fmt};

use http::{HeaderMap, HeaderValue, header::USER_AGENT};

use super::config::Config;

/// WASM client builder for configuring browser-specific HTTP client features
pub struct WasmClientBuilder {
    pub(super) config: Config,
}

impl WasmClientBuilder {
    /// Create new WASM client builder
    pub fn new() -> Self {
        WasmClientBuilder {
            config: Config::default(),
        }
    }

    /// Build WasmClient with configured settings
    pub fn build(mut self) -> Result<super::core::WasmClient, crate::Error> {
        if let Some(err) = self.config.error {
            return Err(err);
        }

        let config = std::mem::take(&mut self.config);
        Ok(super::core::WasmClient::new_with_config(config))
    }

    /// Set User-Agent header for WASM client
    pub fn user_agent<V>(mut self, value: V) -> WasmClientBuilder
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<http::Error>,
    {
        match value.try_into() {
            Ok(value) => {
                self.config.headers.insert(USER_AGENT, value);
            }
            Err(e) => {
                self.config.error = Some(crate::Error::from(format!("Invalid user agent: {}", e.into())));
            }
        }
        self
    }

    /// Set default headers for all WASM requests
    pub fn default_headers(mut self, headers: HeaderMap) -> WasmClientBuilder {
        for (key, value) in headers.iter() {
            self.config.headers.insert(key, value.clone());
        }
        self
    }
}

impl Default for WasmClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WasmClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("WasmClientBuilder");
        self.config.fmt_fields(&mut builder);
        builder.finish()
    }
}

//! Authentication methods for HTTP requests
//!
//! Provides convenient methods for setting authentication headers including
//! API keys, basic authentication, and bearer token authentication.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http::HeaderValue;

use crate::builder::core::Http3Builder;
use crate::builder::headers::header;

impl<S> Http3Builder<S> {
    /// Set API key authentication header
    ///
    /// Adds an `X-API-Key` header with the provided API key value.
    ///
    /// # Arguments
    /// * `key` - The API key value
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::json()
    ///     .api_key("your-api-key-here")
    ///     .get("https://api.example.com/protected");
    /// ```
    #[must_use]
    pub fn api_key(self, key: &str) -> Self {
        use std::str::FromStr;
        match HeaderValue::from_str(key) {
            Ok(header_value) => {
                use http::HeaderName;
                match HeaderName::from_str(header::X_API_KEY) {
                    Ok(name) => self.header(name, header_value),
                    Err(_) => self,
                }
            }
            Err(_) => self, // Skip invalid header value
        }
    }

    /// Set basic authentication header
    ///
    /// Creates a Basic Authentication header using the provided username and password.
    /// The credentials are automatically base64 encoded as required by the HTTP specification.
    ///
    /// # Arguments
    /// * `auth_config` - Authentication configuration using [(\"user\", \"password\")] syntax
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    /// use hashbrown::HashMap;
    ///
    /// let auth = HashMap::from([(\"username\", \"password\")]);
    /// let response = Http3Builder::json()
    ///     .basic_auth(auth)
    ///     .get("https://api.example.com/protected");
    /// ```
    #[must_use]
    pub fn basic_auth(
        self,
        auth_config: impl Into<hashbrown::HashMap<&'static str, &'static str>>,
    ) -> Self {
        let auth_config = auth_config.into();
        if let Some((user, pass)) = auth_config.into_iter().next() {
            let auth_string = format!("{user}:{pass}");
            let encoded = STANDARD.encode(auth_string);
            let header_value = format!("Basic {encoded}");
            return match HeaderValue::from_str(&header_value) {
                Ok(value) => self.header(header::AUTHORIZATION, value),
                Err(_) => self, // Skip invalid header value
            };
        }
        self
    }

    /// Set bearer token authentication header
    ///
    /// Creates a Bearer token authentication header for OAuth2 and similar token-based
    /// authentication schemes.
    ///
    /// # Arguments
    /// * `token` - The bearer token to use for authentication
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::json()
    ///     .bearer_auth("your-oauth-token-here")
    ///     .get("https://api.example.com/protected");
    /// ```
    #[must_use]
    pub fn bearer_auth(self, token: &str) -> Self {
        let header_value = format!("Bearer {token}");
        match HeaderValue::from_str(&header_value) {
            Ok(value) => self.header(header::AUTHORIZATION, value),
            Err(_) => self, // Skip invalid header value
        }
    }
}
//! Provides convenient methods for setting authentication headers including
//! API keys, basic authentication, and bearer token authentication.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::builder::Http3Builder;

/// Trait for authentication methods
pub trait AuthMethod {
    /// Set API key authentication header
    fn api_key(self, key: &str) -> Self;

    /// Set bearer token authentication header
    fn bearer_auth(self, token: &str) -> Self;

    /// Set basic authentication header
    fn basic_auth(self, username: &str, password: Option<&str>) -> Self;
}

/// Bearer authentication implementation
pub struct BearerAuth;

/// Basic authentication implementation
pub struct BasicAuth;

impl<S> AuthMethod for Http3Builder<S> {
    #[inline]
    fn api_key(self, key: &str) -> Self {
        self.header("X-API-Key", key)
    }

    #[inline]
    fn bearer_auth(self, token: &str) -> Self {
        let auth_value = format!("Bearer {token}");
        self.header("Authorization", &auth_value)
    }

    #[inline]
    fn basic_auth(self, username: &str, password: Option<&str>) -> Self {
        let credentials = match password {
            Some(pwd) => format!("{username}:{pwd}"),
            None => format!("{username}:"),
        };

        let encoded = STANDARD.encode(credentials.as_bytes());
        let auth_value = format!("Basic {encoded}");
        self.header("Authorization", &auth_value)
    }
}

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
    #[inline]
    pub fn api_key(self, key: &str) -> Self {
        self.header("X-API-Key", key)
    }

    /// Set bearer token authentication header
    ///
    /// Adds an `Authorization: Bearer <token>` header.
    ///
    /// # Arguments
    /// * `token` - The bearer token value
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::json()
    ///     .bearer_auth("your-bearer-token")
    ///     .get("https://api.example.com/protected");
    /// ```
    #[inline]
    pub fn bearer_auth(self, token: &str) -> Self {
        let auth_value = format!("Bearer {token}");
        self.header("Authorization", &auth_value)
    }

    /// Set basic authentication header
    ///
    /// Adds an `Authorization: Basic <credentials>` header where credentials
    /// are base64-encoded username:password.
    ///
    /// # Arguments
    /// * `username` - The username for basic auth
    /// * `password` - Optional password (if None, only username is used)
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::json()
    ///     .basic_auth("username", Some("password"))
    ///     .get("https://api.example.com/protected");
    /// ```
    #[inline]
    pub fn basic_auth(self, username: &str, password: Option<&str>) -> Self {
        let credentials = match password {
            Some(pwd) => format!("{username}:{pwd}"),
            None => format!("{username}:"),
        };

        let encoded = STANDARD.encode(credentials.as_bytes());
        let auth_value = format!("Basic {encoded}");
        self.header("Authorization", &auth_value)
    }
}

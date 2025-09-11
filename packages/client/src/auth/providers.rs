//! Authentication Module - Bearer tokens, API keys, Basic auth, OAuth 2.0, JWT


use http::{HeaderMap, HeaderName, HeaderValue};
use url::Url;

use crate::prelude::*;

/// Authentication provider trait for different auth types
pub trait AuthProvider {
    /// Apply authentication to headers
    /// 
    /// # Errors
    /// 
    /// Returns `HttpError` if authentication fails or headers cannot be modified
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError>;

    /// Apply authentication to URL (for query parameter auth)
    /// 
    /// # Errors
    /// 
    /// Returns `HttpError` if authentication fails or URL cannot be modified
    fn apply_url_auth(&self, _url: &mut Url) -> Result<(), HttpError> {
        // Default implementation does nothing - most auth is header-based
        Ok(())
    }

    /// Get authentication method name
    fn auth_type(&self) -> &'static str;
}

/// Bearer token authentication
pub struct BearerToken {
    token: String,
}

impl BearerToken {
    /// Create new bearer token auth
    #[must_use]
    #[inline]
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl AuthProvider for BearerToken {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        if self.token.is_empty() {
            Ok(())
        } else {
            let auth_header = format!("Bearer {}", self.token);
            match HeaderValue::from_str(&auth_header) {
                Ok(header_value) => {
                    headers.insert(http::header::AUTHORIZATION, header_value);
                    Ok(())
                }
                Err(e) => Err(crate::error::configuration(format!(
                    "Invalid bearer token: {e}"
                ))),
            }
        }
    }

    #[inline]
    fn auth_type(&self) -> &'static str {
        "Bearer"
    }
}

/// API key authentication (header or query parameter)
pub struct ApiKey {
    key: String,
    placement: ApiKeyPlacement,
}

/// Where to place the API key
pub enum ApiKeyPlacement {
    /// Place API key in HTTP header with specified name
    Header(String),
    /// Place API key in URL query parameter with specified name
    Query(String),
}

impl ApiKey {
    /// Create new API key auth
    #[must_use]
    #[inline]
    pub fn new(key: String, placement: ApiKeyPlacement) -> Self {
        Self { key, placement }
    }
}

impl AuthProvider for ApiKey {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        if let ApiKeyPlacement::Header(header_name) = &self.placement {
            if self.key.is_empty() {
                Ok(())
            } else {
                match HeaderName::from_bytes(header_name.as_bytes()) {
                    Ok(name) => match HeaderValue::from_str(&self.key) {
                        Ok(value) => {
                            headers.insert(name, value);
                            Ok(())
                        }
                        Err(e) => Err(crate::error::configuration(format!(
                            "Invalid API key value: {e}"
                        ))),
                    },
                    Err(e) => Err(crate::error::configuration(format!(
                        "Invalid header name: {e}"
                    ))),
                }
            }
        } else {
            // Query parameter auth is handled in apply_url_auth
            Ok(())
        }
    }

    fn apply_url_auth(&self, url: &mut Url) -> Result<(), HttpError> {
        if let ApiKeyPlacement::Query(param_name) = &self.placement
            && !self.key.is_empty() {
            url.query_pairs_mut().append_pair(param_name, &self.key);
        }
        Ok(())
    }

    #[inline]
    fn auth_type(&self) -> &'static str {
        "ApiKey"
    }
}



/// Authentication errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    /// Authentication token is malformed or invalid
    #[error("Invalid authentication token")]
    InvalidToken,
    /// Provided credentials do not match expected values
    #[error("Invalid credentials")]
    InvalidCredentials,
    /// Authentication token has passed its expiration time
    #[error("Authentication token expired")]
    TokenExpired,
    /// Error occurred while encoding authentication data
    #[error("Authentication encoding error")]
    EncodingError,
    /// Request requires authentication but none was provided
    #[error("Authentication required")]
    AuthRequired,
}

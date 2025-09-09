//! Authentication Module - Bearer tokens, API keys, Basic auth, OAuth 2.0, JWT

use base64::{Engine as _, engine::general_purpose};
use http::{HeaderMap, HeaderName, HeaderValue};

use crate::prelude::*;

/// Authentication provider trait for different auth types
pub trait AuthProvider {
    /// Apply authentication to headers
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError>;

    /// Get authentication method name
    fn auth_type(&self) -> &'static str;
}

/// Bearer token authentication
pub struct BearerToken {
    token: String,
}

impl BearerToken {
    /// Create new bearer token auth
    #[inline(always)]
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl AuthProvider for BearerToken {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        if !self.token.is_empty() {
            let auth_header = format!("Bearer {}", self.token);
            match HeaderValue::from_str(&auth_header) {
                Ok(header_value) => {
                    headers.insert(http::header::AUTHORIZATION, header_value);
                    Ok(())
                }
                Err(e) => Err(crate::error::configuration(format!(
                    "Invalid bearer token: {}",
                    e
                ))),
            }
        } else {
            Ok(())
        }
    }

    #[inline(always)]
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
    #[inline(always)]
    pub fn new(key: String, placement: ApiKeyPlacement) -> Self {
        Self { key, placement }
    }
}

impl AuthProvider for ApiKey {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        if let ApiKeyPlacement::Header(header_name) = &self.placement {
            if !self.key.is_empty() {
                match HeaderName::from_bytes(header_name.as_bytes()) {
                    Ok(name) => match HeaderValue::from_str(&self.key) {
                        Ok(value) => {
                            headers.insert(name, value);
                            Ok(())
                        }
                        Err(e) => Err(crate::error::configuration(format!(
                            "Invalid API key value: {}",
                            e
                        ))),
                    },
                    Err(e) => Err(crate::error::configuration(format!(
                        "Invalid header name: {}",
                        e
                    ))),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn auth_type(&self) -> &'static str {
        "ApiKey"
    }
}

/// Basic authentication
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}

impl BasicAuth {
    /// Create new basic auth
    #[inline(always)]
    pub fn new(username: String, password: Option<String>) -> Self {
        Self { username, password }
    }
}

impl AuthProvider for BasicAuth {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        if !self.username.is_empty() {
            let credentials = format!(
                "{}:{}",
                self.username,
                self.password.as_deref().unwrap_or_default()
            );
            let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
            let auth_header = format!("Basic {}", encoded);
            match HeaderValue::from_str(&auth_header) {
                Ok(header_value) => {
                    headers.insert(http::header::AUTHORIZATION, header_value);
                    Ok(())
                }
                Err(e) => Err(crate::error::configuration(format!(
                    "Invalid basic auth header: {}",
                    e
                ))),
            }
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn auth_type(&self) -> &'static str {
        "Basic"
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

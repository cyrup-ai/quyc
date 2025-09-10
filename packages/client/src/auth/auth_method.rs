//! `AuthMethod` - Legacy compatibility wrapper for authentication
//!
//! This module provides a simplified `AuthMethod` interface that wraps the
//! more comprehensive `AuthProvider` trait system for backward compatibility.

use http::HeaderMap;

use crate::auth::auth::BasicAuth;
use crate::auth::{ApiKey, ApiKeyPlacement, AuthProvider, BearerToken};

/// Legacy-compatible `AuthMethod` wrapper
pub enum AuthMethod {
    /// Bearer token authentication
    Bearer(BearerToken),
    /// API key authentication
    ApiKey(ApiKey),
    /// Basic authentication
    Basic(BasicAuth),
}

impl AuthMethod {
    /// Create bearer token authentication
    pub fn bearer_token(token: impl Into<String>) -> Self {
        Self::Bearer(BearerToken::new(token.into()))
    }

    /// Create API key header authentication
    pub fn api_key(key: impl Into<String>) -> Self {
        Self::ApiKey(ApiKey::new(
            key.into(),
            ApiKeyPlacement::Header("X-API-Key".to_string()),
        ))
    }

    /// Create API key query parameter authentication
    pub fn query_param(param_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self::ApiKey(ApiKey::new(
            key.into(),
            ApiKeyPlacement::Query(param_name.into()),
        ))
    }

    /// Create basic authentication
    pub fn basic_auth(username: impl Into<String>, password: Option<String>) -> Self {
        Self::Basic(BasicAuth::new(username.into(), password))
    }
}

impl AuthProvider for AuthMethod {
    fn apply_auth(&self, headers: &mut HeaderMap) -> Result<(), crate::error::HttpError> {
        match self {
            Self::Bearer(bearer) => bearer.apply_auth(headers),
            Self::ApiKey(api_key) => api_key.apply_auth(headers),
            Self::Basic(basic) => basic.apply_auth(headers),
        }
    }

    fn auth_type(&self) -> &'static str {
        match self {
            Self::Bearer(bearer) => bearer.auth_type(),
            Self::ApiKey(api_key) => api_key.auth_type(),
            Self::Basic(basic) => basic.auth_type(),
        }
    }
}

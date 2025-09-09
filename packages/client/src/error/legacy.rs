use super::types::HttpError;

// Error helper functions for legacy compatibility
pub fn builder<E: std::fmt::Display>(e: E) -> HttpError {
    HttpError::builder(e.to_string())
}

pub fn request<E: std::fmt::Display>(e: E) -> HttpError {
    HttpError::request(e.to_string())
}

// TimedOut and body functions for legacy compatibility
pub fn TimedOut() -> HttpError {
    HttpError::timeout("Request timed out".to_string())
}

pub fn body<E: std::fmt::Display>(e: E) -> HttpError {
    HttpError::body(e.to_string())
}

// Re-export internal hyper error types and helpers for legacy paths expecting `crate::error::*`
pub(crate) use crate::error::helpers::BadScheme;

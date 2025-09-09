//! WASM-specific error handling utilities
//!
//! This module provides error conversion and handling specifically for WebAssembly
//! environments, converting JavaScript errors to HTTP errors.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use super::types::HttpError;

/// Convert a JavaScript error to an HTTP error
#[cfg(target_arch = "wasm32")]
pub fn wasm(js_error: JsValue) -> HttpError {
    let error_message = if let Some(string) = js_error.as_string() {
        string
    } else {
        format!("JavaScript error: {:?}", js_error)
    };

    HttpError::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        error_message,
    ))
}

/// Convert a JavaScript error to an HTTP error (non-WASM fallback)
#[cfg(not(target_arch = "wasm32"))]
pub fn wasm<T>(_js_error: T) -> HttpError {
    HttpError::from(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "WASM error handling not available on this platform",
    ))
}

//! ContentTypes - Legacy compatibility wrapper for content type constants
//!
//! This module provides a simplified ContentTypes interface that wraps the
//! ContentType enum for backward compatibility.

use crate::builder::ContentType;

/// Legacy-compatible ContentTypes constants
pub struct ContentTypes;

impl ContentTypes {
    /// JSON content type
    pub const JSON: ContentType = ContentType::ApplicationJson;

    /// Form URL-encoded content type
    pub const FORM: ContentType = ContentType::ApplicationFormUrlEncoded;

    /// Plain text content type
    pub const TEXT: ContentType = ContentType::TextPlain;

    /// Binary/octet-stream content type
    pub const BINARY: ContentType = ContentType::ApplicationOctetStream;

    /// Multipart form data content type
    pub const MULTIPART: ContentType = ContentType::MultipartFormData;
}

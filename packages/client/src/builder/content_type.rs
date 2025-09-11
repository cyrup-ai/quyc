//! Content type enumeration and conversions for HTTP requests
//!
//! Provides elegant `ContentType` enum with string conversions and common MIME types.

/// Content type enumeration for elegant API
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    /// application/json content type
    ApplicationJson,
    /// application/x-www-form-urlencoded content type
    ApplicationFormUrlEncoded,
    /// application/octet-stream content type
    ApplicationOctetStream,
    /// text/plain content type
    TextPlain,
    /// text/html content type
    TextHtml,
    /// multipart/form-data content type
    MultipartFormData,
}

impl ContentType {
    /// Convert content type to string representation
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ContentType::ApplicationJson => "application/json",
            ContentType::ApplicationFormUrlEncoded => "application/x-www-form-urlencoded",
            ContentType::ApplicationOctetStream => "application/octet-stream",
            ContentType::TextPlain => "text/plain",
            ContentType::TextHtml => "text/html",
            ContentType::MultipartFormData => "multipart/form-data",
        }
    }
}

impl From<&str> for ContentType {
    fn from(s: &str) -> Self {
        match s {
            "application/x-www-form-urlencoded" => ContentType::ApplicationFormUrlEncoded,
            "application/octet-stream" => ContentType::ApplicationOctetStream,
            "text/plain" => ContentType::TextPlain,
            "text/html" => ContentType::TextHtml,
            "multipart/form-data" => ContentType::MultipartFormData,
            _ => ContentType::ApplicationJson, // Default fallback including "application/json"
        }
    }
}

//! Provides methods for setting and managing HTTP headers including
//! common headers like Content-Type, Accept, and custom headers.

use http::{HeaderName, HeaderValue};

use crate::builder::core::{ContentType, Http3Builder};

/// Trait for header building operations
pub trait HeaderBuilder {
    /// Set a custom header
    fn header(self, name: &str, value: &str) -> Self;

    /// Set Accept header
    fn accept(self, content_type: ContentType) -> Self;

    /// Set User-Agent header
    fn user_agent(self, agent: &str) -> Self;
}

/// Header configuration settings
pub struct HeaderConfig {
    pub default_headers: Vec<(String, String)>,
    pub override_defaults: bool,
}

impl Default for HeaderConfig {
    #[inline]
    fn default() -> Self {
        Self {
            default_headers: Vec::new(),
            override_defaults: false,
        }
    }
}

/// Helper type for accept method that can handle both strings and ContentType enums
pub enum AcceptValue {
    /// String representation of content type
    String(String),
    /// ContentType enum variant
    ContentType(ContentType),
}

impl AcceptValue {
    /// Convert to string representation
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            AcceptValue::String(s) => s,
            AcceptValue::ContentType(ct) => ct.as_str(),
        }
    }
}

impl From<&str> for AcceptValue {
    #[inline]
    fn from(s: &str) -> Self {
        AcceptValue::String(s.to_string())
    }
}

impl From<String> for AcceptValue {
    #[inline]
    fn from(s: String) -> Self {
        AcceptValue::String(s)
    }
}

impl From<ContentType> for AcceptValue {
    #[inline]
    fn from(ct: ContentType) -> Self {
        AcceptValue::ContentType(ct)
    }
}

impl<S> HeaderBuilder for Http3Builder<S> {
    #[inline]
    fn header(self, name: &str, value: &str) -> Self {
        self.set_header(name, value)
    }

    #[inline]
    fn accept(self, content_type: ContentType) -> Self {
        self.set_accept(content_type)
    }

    #[inline]
    fn user_agent(self, agent: &str) -> Self {
        self.set_user_agent(agent)
    }
}

impl<S> Http3Builder<S> {
    /// Set a custom header
    ///
    /// # Arguments
    /// * `name` - Header name
    /// * `value` - Header value
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3;
    ///
    /// let response = Http3::json()
    ///     .header("X-Custom-Header", "custom-value")
    ///     .get("https://api.example.com/data");
    /// ```
    #[inline]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap_or_else(|_| {
            log::error!("Invalid header name: {}", name);
            HeaderName::from_static("x-invalid")
        });

        let header_value = HeaderValue::from_str(value).unwrap_or_else(|_| {
            log::error!("Invalid header value: {}", value);
            HeaderValue::from_static("")
        });

        self.request.headers_mut().insert(header_name, header_value);

        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set header {} = {}", name, value);
        }

        self
    }

    /// Set Accept header with ContentType enum
    ///
    /// # Arguments
    /// * `content_type` - The content type to accept
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::{Http3, ContentType};
    ///
    /// let response = Http3::new()
    ///     .accept(ContentType::ApplicationJson)
    ///     .get("https://api.example.com/data");
    /// ```
    #[inline]
    pub fn accept(self, content_type: ContentType) -> Self {
        self.header("Accept", content_type.as_str())
    }

    /// Set User-Agent header
    ///
    /// # Arguments
    /// * `agent` - User agent string
    ///
    /// # Returns
    /// `Self` for method chaining
    #[inline]
    pub fn user_agent(self, agent: &str) -> Self {
        self.header("User-Agent", agent)
    }

    /// Set Content-Length header
    ///
    /// # Arguments
    /// * `length` - Content length in bytes
    ///
    /// # Returns
    /// `Self` for method chaining
    #[inline]
    pub fn content_length(self, length: usize) -> Self {
        self.header("Content-Length", &length.to_string())
    }

    /// Set Cache-Control header
    ///
    /// # Arguments
    /// * `directive` - Cache control directive
    ///
    /// # Returns
    /// `Self` for method chaining
    #[inline]
    pub fn cache_control(self, directive: &str) -> Self {
        self.header("Cache-Control", directive)
    }

    /// Internal method to set header
    #[inline]
    fn set_header(mut self, name: &str, value: &str) -> Self {
        let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap_or_else(|_| {
            log::error!("Invalid header name: {}", name);
            HeaderName::from_static("x-invalid")
        });

        let header_value = HeaderValue::from_str(value).unwrap_or_else(|_| {
            log::error!("Invalid header value: {}", value);
            HeaderValue::from_static("")
        });

        self.request.headers_mut().insert(header_name, header_value);

        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set header {} = {}", name, value);
        }

        self
    }

    /// Internal method to set Accept header
    #[inline]
    fn set_accept(self, content_type: ContentType) -> Self {
        self.header("Accept", content_type.as_str())
    }

    /// Internal method to set User-Agent header
    #[inline]
    fn set_user_agent(self, agent: &str) -> Self {
        self.header("User-Agent", agent)
    }
}

/// Helper function to create header value from string
#[inline]
pub fn header(name: &str, value: &str) -> (HeaderName, HeaderValue) {
    let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap_or_else(|_| {
        log::error!("Invalid header name: {}", name);
        HeaderName::from_static("x-invalid")
    });

    let header_value = HeaderValue::from_str(value).unwrap_or_else(|_| {
        log::error!("Invalid header value: {}", value);
        HeaderValue::from_static("")
    });

    (header_name, header_value)
}

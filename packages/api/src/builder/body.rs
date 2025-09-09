//! Request body handling functionality
//!
//! Provides methods for setting request bodies with automatic serialization
//! for JSON, form-urlencoded, and other content types.

// PhantomData import removed - not used

use bytes::Bytes;
use serde::Serialize;

use crate::builder::core::{BodyNotSet, BodySet, Http3Builder, ContentType};

impl Http3Builder<BodyNotSet> {
    /// Set the request body with automatic serialization
    ///
    /// Automatically serializes the body based on the Content-Type header:
    /// - `application/json`: JSON serialization (default)
    /// - `application/x-www-form-urlencoded`: Form serialization
    ///
    /// # Arguments
    /// * `body` - The data to serialize and set as request body
    ///
    /// # Returns
    /// `Http3Builder<BodySet>` for chaining to terminal methods
    ///
    /// # Type Parameters
    /// * `T` - Type that implements Serialize
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user = User {
    ///     name: "John Doe".to_string(),
    ///     email: "john@example.com".to_string(),
    /// };
    ///
    /// let response = Http3Builder::json()
    ///     .body(&user)
    ///     .post("https://api.example.com/users");
    /// ```
    #[must_use]
    pub fn body<T: Serialize>(self, body: &T) -> Http3Builder<BodySet> {
        let content_type = self
            .request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json");

        let body_bytes = if content_type.contains("application/x-www-form-urlencoded") {
            // Serialize as form-urlencoded
            serde_urlencoded::to_string(body)
                .map(std::string::String::into_bytes)
                .unwrap_or_default()
        } else {
            // Default to JSON serialization
            serde_json::to_vec(body).unwrap_or_default()
        };

        if self.debug_enabled {
            log::debug!(
                "HTTP3 Builder: Set request body ({} bytes, content-type: {})",
                body_bytes.len(),
                content_type
            );
        }

        let request = self.request.body_bytes(Bytes::from(body_bytes));

        Http3Builder {
            client: self.client,
            request,
            state: BodySet,
            debug_enabled: self.debug_enabled,
            chunk_handler: self.chunk_handler,
        }
    }

    /// Set raw bytes as request body
    ///
    /// Sets the request body directly as raw bytes without any serialization.
    /// Useful for binary data, pre-serialized content, or custom formats.
    ///
    /// # Arguments
    /// * `bytes` - Raw bytes to set as request body
    ///
    /// # Returns
    /// `Http3Builder<BodySet>` for chaining to terminal methods
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let raw_data = b"custom binary data";
    /// let response = Http3Builder::new(&client)
    ///     .content_type(ContentType::ApplicationOctetStream)
    ///     .raw_body(raw_data.to_vec())
    ///     .post("https://api.example.com/upload");
    /// ```
    #[must_use]
    pub fn raw_body(self, bytes: Vec<u8>) -> Http3Builder<BodySet> {
        if self.debug_enabled {
            log::debug!(
                "HTTP3 Builder: Set raw request body ({} bytes)",
                bytes.len()
            );
        }

        let request = self.request.body_bytes(Bytes::from(bytes));

        Http3Builder {
            client: self.client,
            request,
            state: BodySet,
            debug_enabled: self.debug_enabled,
            chunk_handler: self.chunk_handler,
        }
    }

    /// Set text content as request body
    ///
    /// Sets the request body as UTF-8 encoded text. Automatically sets
    /// Content-Type to text/plain if not already set.
    ///
    /// # Arguments
    /// * `text` - Text content to set as request body
    ///
    /// # Returns
    /// `Http3Builder<BodySet>` for chaining to terminal methods
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::new(&client)
    ///     .text_body("Hello, World!")
    ///     .post("https://api.example.com/messages");
    /// ```
    #[must_use]
    pub fn text_body(self, text: &str) -> Http3Builder<BodySet> {
        if self.debug_enabled {
            log::debug!(
                "HTTP3 Builder: Set text request body ({} chars)",
                text.len()
            );
        }

        // Set content type to text/plain if not already set
        let builder = if self.request.headers().get("content-type").is_none() {
            self.content_type(ContentType::TextPlain)
        } else {
            self
        };

        let request = builder.request.body_text(text);

        Http3Builder {
            client: builder.client,
            request,
            state: BodySet,
            debug_enabled: builder.debug_enabled,
            chunk_handler: builder.chunk_handler,
        }
    }
}
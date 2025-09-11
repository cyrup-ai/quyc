//! Provides methods for setting request bodies with automatic serialization
//! for JSON, form-urlencoded, and other content types.

use std::marker::PhantomData;

use serde::Serialize;

use crate::builder::core::{BodyNotSet, BodySet, Http3Builder};

/// Trait for body building operations
pub trait BodyBuilder {
    /// Set the request body with automatic serialization
    fn body<T: Serialize>(self, body: &T) -> Http3Builder<BodySet>;
}

/// JSON body implementation
pub struct JsonBody;

/// Text body implementation
pub struct TextBody;

impl BodyBuilder for Http3Builder<BodyNotSet> {
    #[inline]
    fn body<T: Serialize>(self, body: &T) -> Http3Builder<BodySet> {
        self.serialize_body(body)
    }
}

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
    /// struct CreateUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user = CreateUser {
    ///     name: "Alice".to_string(),
    ///     email: "alice@example.com".to_string(),
    /// };
    ///
    /// let response = Http3Builder::json()
    ///     .body(&user)
    ///     .post("https://api.example.com/users");
    /// ```
    #[inline]
    pub fn body<T: Serialize>(self, body: &T) -> Http3Builder<BodySet> {
        self.serialize_body(body)
    }

    /// Set raw text body
    ///
    /// Sets the request body to the provided text string without serialization.
    ///
    /// # Arguments
    /// * `text` - The text content to set as request body
    ///
    /// # Returns
    /// `Http3Builder<BodySet>` for chaining to terminal methods
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::new()
    ///     .header("Content-Type", "text/plain")
    ///     .text_body("Hello, World!")
    ///     .post("https://api.example.com/echo");
    /// ```
    #[inline]
    pub fn text_body(mut self, text: &str) -> Http3Builder<BodySet> {
        self.request = self.request.body_text(text);

        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set text body ({} bytes)", text.len());
        }

        Http3Builder {
            client: self.client,
            request: self.request,
            debug_enabled: self.debug_enabled,
            state: PhantomData,
            jsonpath_config: None,
            chunk_handler: None,
            protocol_config: self.protocol_config,
        }
    }

    /// Set raw bytes body
    ///
    /// Sets the request body to the provided byte array without any processing.
    ///
    /// # Arguments
    /// * `bytes` - The byte data to set as request body
    ///
    /// # Returns
    /// `Http3Builder<BodySet>` for chaining to terminal methods
    #[inline]
    pub fn bytes_body(mut self, bytes: Vec<u8>) -> Http3Builder<BodySet> {
        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set bytes body ({} bytes)", bytes.len());
        }

        self.request = self.request.body_bytes(bytes);

        Http3Builder {
            client: self.client,
            request: self.request,
            debug_enabled: self.debug_enabled,
            state: PhantomData,
            jsonpath_config: None,
            chunk_handler: None,
            protocol_config: self.protocol_config,
        }
    }

    /// Internal method to serialize body based on content type
    #[inline]
    fn serialize_body<T: Serialize>(mut self, body: &T) -> Http3Builder<BodySet> {
        let content_type = self
            .request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json");

        let serialized_body = match content_type {
            "application/x-www-form-urlencoded" => match serde_urlencoded::to_string(body) {
                Ok(form_data) => form_data.into_bytes(),
                Err(e) => {
                    log::error!("Failed to serialize form data: {e}");
                    Vec::new()
                }
            },
            _ => {
                // Default to JSON serialization
                match serde_json::to_vec(body) {
                    Ok(json_data) => json_data,
                    Err(e) => {
                        log::error!("Failed to serialize JSON: {e}");
                        Vec::new()
                    }
                }
            }
        };

        if self.debug_enabled {
            log::debug!(
                "HTTP3 Builder: Serialized body as {} ({} bytes)",
                content_type,
                serialized_body.len()
            );
        }

        self.request = self.request.body_bytes(serialized_body);

        Http3Builder {
            client: self.client,
            request: self.request,
            debug_enabled: self.debug_enabled,
            state: PhantomData,
            jsonpath_config: None,
            chunk_handler: None,
            protocol_config: self.protocol_config,
        }
    }
}

//! Core `Http3Builder` with pure `AsyncStream` architecture - NO Futures
//!
//! ALL methods return `AsyncStream<T, CAP>` directly from ystream
//! NO middleware, NO abstractions - pure streaming protocols

use std::marker::PhantomData;
use std::sync::Arc;


// Removed unused imports
use http::Method;
use url::Url;

pub use super::content_type::ContentType;
pub use super::state_types::{BodyNotSet, BodySet, JsonPathStreaming};
use crate::prelude::*;

/// Chunk handler function type for error handling in streaming
type ChunkHandler = Arc<dyn Fn(Result<HttpChunk, HttpError>) -> HttpChunk + Send + Sync + 'static>;

/// HTTP/3 and HTTP/2 configuration parameters
#[derive(Debug, Clone, Default)]
pub struct ProtocolConfig {
    /// HTTP/3 stream receive window size
    pub h3_stream_receive_window: Option<u32>,
    /// HTTP/3 connection receive window size  
    pub h3_conn_receive_window: Option<u32>,
    /// HTTP/3 send window size
    pub h3_send_window: Option<u32>,
    /// Enable BBR congestion control for HTTP/3
    pub h3_congestion_bbr: Option<bool>,
    /// Maximum field section size for HTTP/3
    pub h3_max_field_section_size: Option<u64>,
    /// Enable GREASE sending for HTTP/3
    pub h3_send_grease: Option<bool>,
    /// Enable HTTP/2 adaptive window
    pub h2_adaptive_window: Option<bool>,
    /// HTTP/2 maximum frame size
    pub h2_max_frame_size: Option<u32>,
}

/// Main Http3 builder for constructing HTTP requests with fluent API
///
/// Type parameter `S` tracks the body state:
/// - `BodyNotSet`: Default state, body methods available
/// - `BodySet`: Body has been set, only execution methods available
/// - `JsonPathStreaming`: Configured for `JSONPath` array streaming
#[derive(Clone)]
#[must_use = "builders do nothing unless you call a build method"]
pub struct Http3Builder<S = BodyNotSet> {
    /// HTTP client instance for making requests
    pub(crate) client: HttpClient,
    /// Request being built
    pub(crate) request: HttpRequest,
    /// Type state marker
    pub(crate) state: PhantomData<S>,
    /// Debug logging enabled flag
    pub(crate) debug_enabled: bool,
    /// `JSONPath` streaming configuration
    #[allow(dead_code)]
    pub(crate) jsonpath_config: Option<JsonPathStreaming>,
    /// Chunk handler for error handling in streaming
    pub(crate) chunk_handler: Option<ChunkHandler>,
    /// Protocol-specific configuration parameters
    pub(crate) protocol_config: ProtocolConfig,
}

impl Http3Builder<BodyNotSet> {
    /// Start building a new request with a default client instance
    pub fn new() -> Self {
        let client = HttpClient::default();
        Self::with_client(&client)
    }

    /// Start building a new request with a shared client instance
    /// Create a new builder with the given HTTP client
    ///
    /// # Panics
    /// Panics if the URL parsing system is completely broken (should never happen in practice)
    pub fn with_client(client: &HttpClient) -> Self {
        Self {
            client: client.clone(),
            request: {
                // Use a simple approach that cannot panic
                // Try a few basic URLs, but accept failure gracefully
                // Use the safest URL that should always work
                // If this fails, the URL system is fundamentally broken
                let default_url = match Url::parse("https://localhost") {
                    Ok(url) => url,
                    Err(_) => {
                        // Try simpler URLs in order of preference
                        Url::parse("http://localhost")
                            .or_else(|_| Url::parse("data:,"))
                            .or_else(|_| Url::from_file_path("/"))
                            .unwrap_or_else(|()| {
                                // This represents a system failure but we cannot panic
                                // Create a URL that will error during HTTP operations
                                // Use the most basic URL format that should always parse
                                tracing::error!("URL system failure - creating fallback URL");
                                // We know this URL format should always work
                                Url::parse("file:///").expect("basic file URL should always parse")
                            })
                    }
                };
                HttpRequest::new(Method::GET, default_url, None, None, None)
            },
            state: PhantomData,
            debug_enabled: false,
            jsonpath_config: None,
            chunk_handler: None,
            protocol_config: ProtocolConfig::default(),
        }
    }

    /// Shorthand for setting Content-Type to application/json
    ///
    /// # Returns
    /// `Self` for method chaining
    pub fn json() -> Self {
        Self::new().content_type(ContentType::ApplicationJson)
    }

    /// Shorthand for setting Content-Type to application/x-www-form-urlencoded
    ///
    /// # Returns
    /// `Self` for method chaining
    pub fn form_urlencoded() -> Self {
        Self::new().content_type(ContentType::ApplicationFormUrlEncoded)
    }

    /// Configure `JSONPath` streaming for array responses
    ///
    /// Transforms the builder to stream individual objects from JSON arrays
    /// matching the provided `JSONPath` expression.
    ///
    /// # Arguments
    /// * `jsonpath` - `JSONPath` expression to filter array elements
    ///
    /// # Returns
    /// `Http3Builder<JsonPathStreaming>` for streaming operations
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let stream = Http3Builder::json()
    ///     .array_stream("$.items[*]")
    ///     .get("https://api.example.com/data");
    /// ```
    pub fn array_stream(self, jsonpath: &str) -> Http3Builder<JsonPathStreaming> {
        Http3Builder {
            client: self.client,
            request: self.request,
            state: PhantomData,
            debug_enabled: self.debug_enabled,
            jsonpath_config: Some(JsonPathStreaming {
                jsonpath_expr: jsonpath.to_string(),
            }),
            chunk_handler: self.chunk_handler,
            protocol_config: self.protocol_config,
        }
    }
}

impl<S> Http3Builder<S> {
    /// Set the target URL for the request
    ///
    /// # Arguments
    /// * `url` - The complete URL to send the request to
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3Builder;
    ///
    /// let response = Http3Builder::json()
    ///     .url("https://api.example.com/users")
    ///     .get("");
    /// ```
    pub fn url(mut self, url: &str) -> Self {
        match Url::parse(url) {
            Ok(parsed_url) => {
                self.request = self.request.with_url(parsed_url);
                self
            },
            Err(parse_error) => {
                tracing::error!(
                    target: "quyc::builder",
                    url = %url,
                    error = %parse_error,
                    "Invalid URL provided to builder, using fallback"
                );
                // Return self unchanged rather than panicking - graceful degradation
                self
            }
        }
    }

    /// Set content type using the `ContentType` enum
    ///
    /// # Arguments
    /// * `content_type` - The content type to set for the request
    ///
    /// # Returns
    /// `Self` for method chaining
    pub fn content_type(self, content_type: ContentType) -> Self {
        use std::str::FromStr;

        use http::{HeaderName, HeaderValue};

        let content_type_str = content_type.as_str();
        match (
            HeaderName::from_str("content-type"),
            HeaderValue::from_str(content_type_str),
        ) {
            (Ok(name), Ok(value)) => self.header(name.as_str(), value.to_str().unwrap_or_default()),
            _ => self,
        }
    }


}

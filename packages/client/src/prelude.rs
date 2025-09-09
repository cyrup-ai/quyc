//! Fluent AI HTTP3 Prelude
//!
//! This module contains the essential types that end users need for HTTP operations.
//! Only canonical types that are part of the public API belong here.

// Essential HTTP types - the core of what users interact with
pub use crate::http::request::HttpRequest;
pub use crate::http::response::{HttpResponse, HttpStatus, HttpHeader, HttpBodyChunk, HttpChunk, HttpDownloadChunk, HttpDownloadStream};

// Type alias for backward compatibility with example
pub type BadChunk = HttpChunk;

// Error types
pub use crate::error::{Error, HttpError};

// Core client for making requests
pub use crate::client::HttpClient;
pub use crate::config::HttpConfig;

// Builder for fluent API
pub use crate::builder::Http3Builder;
pub use crate::builder::state_types::{BodySet, BodyNotSet, JsonPathStreaming};
pub use crate::builder::execution::{HttpStream, HttpStreamExt};
pub use crate::builder::content_type::ContentType;

// Essential async streaming types
pub use ystream::{AsyncStream, prelude::MessageChunk};

// HTTP standard types from http crate
pub use ::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Version};

// URL handling
pub use url::Url;

// Telemetry types
pub use crate::telemetry::{ClientStats, ClientStatsSnapshot};

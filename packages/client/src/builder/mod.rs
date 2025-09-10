//! `Http3Builder` module with decomposed components
//!
//! Provides the main `Http3Builder` struct and related types for constructing
//! HTTP requests with zero allocation and elegant fluent interface.

pub mod body;
pub mod builder_core;
pub mod configuration;
pub mod content_type;
pub mod content_types;
pub mod execution;
pub mod fluent;
pub mod headers;
pub mod methods;
pub mod state_types;
pub mod streaming;
pub mod trait_impls;

// Re-export builder_core as core for backward compatibility
// Re-export all public types and traits
pub use body::{BodyBuilder, JsonBody, TextBody};
pub use builder_core as core;
pub use builder_core::{BodyNotSet, BodySet, ContentType, Http3Builder, JsonPathStreaming};
pub use configuration::{BuilderConfig, RequestConfig};
pub use content_types::ContentTypes;
pub use execution::{ExecutionContext, RequestExecution};
pub use fluent::{DownloadBuilder, FluentBuilder};
pub use headers::{HeaderBuilder, HeaderConfig};
// methods module is now empty - implementations moved to API package
pub use streaming::{StreamConfig, StreamingBuilder};
pub use trait_impls::{BuilderExt, RequestBuilderExt};

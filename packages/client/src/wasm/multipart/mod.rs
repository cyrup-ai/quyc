//! multipart/form-data
//!
//! Decomposed multipart form handling for WASM environments.
//! Provides Form and Part types for constructing multipart/form-data requests
//! compatible with the browser fetch API.

mod form;
mod metadata;
mod part;
mod types;

// Re-export main types
pub use types::{Form, Part};
// Re-export internal types for crate use
pub(crate) use types::{FormParts, PartMetadata, PartProps};

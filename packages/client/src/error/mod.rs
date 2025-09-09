pub mod classification;
pub mod constructors;
pub mod helpers;
pub mod types;

// Re-export main types and functions for backward compatibility
pub use constructors::*;
#[cfg(target_arch = "wasm32")]
pub use helpers::wasm;
pub use helpers::{
    BadScheme, ConnectionClosed, IncompleteMessage, OperationCanceled, TimedOut, UnexpectedMessage,
    decode, status_code,
};
pub use types::{Error, Inner, Kind, Result};

// Type aliases for compatibility with existing codebase
pub type HttpError = Error;

// Re-export internal types needed by other modules
pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Re-export classification methods through the Error type
// (these are implemented as inherent methods on Error in classification.rs)

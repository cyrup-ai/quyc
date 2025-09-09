pub mod auth;
pub mod auth_method;
pub mod basic_auth;
pub mod builder;

// Re-export specific types to avoid conflicts
pub use auth::{AuthProvider, BearerToken, ApiKey, ApiKeyPlacement, AuthError};
pub use basic_auth::{basic_auth, encode_basic_auth, decode_basic_auth, BasicAuth};
pub use builder::*;

//! TLS Builder Interface
//!
//! This module provides a fluent, secure-by-default certificate management API.
//! All internal complexity is hidden behind the builder interface.

#![allow(dead_code)]

// Internal modules - not exposed publicly
pub(crate) mod certificate;
pub(crate) mod crl_cache;
pub mod errors;
pub(crate) mod key_encryption;
pub(crate) mod ocsp;

pub(crate) mod tls_manager;
pub(crate) mod types;

// Public builder interface - the only public API
pub mod builder;
pub use builder::{CertificateAuthority, Tls};

// Public TLS manager for enterprise connections
pub use tls_manager::{TlsManager, TlsConfig};

// Public error types for TLS operations
pub use errors::TlsError;

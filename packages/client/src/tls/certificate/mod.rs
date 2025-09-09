//! Certificate management module
//!
//! This module provides comprehensive certificate lifecycle management including:
//! - Certificate generation and loading
//! - Certificate parsing and validation
//! - Certificate chain verification
//! - Wildcard certificate support

pub mod generation;
pub mod parser;
pub mod parsing;
pub mod validation;
pub mod wildcard;

// Re-export main certificate functions
// generation::new re-export removed - not used
// Re-export internal parsing function for use within certificate module
pub use parser::parse_certificate_from_pem;
pub use parsing::{
    validate_basic_constraints, validate_certificate_time,
    validate_key_usage,
};
pub use validation::validate_certificate_chain;
// wildcard function re-export removed - not used

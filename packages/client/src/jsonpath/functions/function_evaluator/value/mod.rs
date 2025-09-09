//! RFC 9535 Section 2.4.8: value() function module
//!
//! This module provides the value() function implementation for JSONPath operations.

pub mod core;
pub mod property_access;

pub use core::evaluate_value_function;

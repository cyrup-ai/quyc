//! HTTP Cookie management and utilities
//!
//! This module provides comprehensive cookie handling functionality including:
//! - Cookie parsing and validation
//! - Cookie store implementations
//! - Cookie header utilities
//! - RFC 6265 compliant cookie handling

#![allow(dead_code)]

pub mod core;
pub mod utils;

// Re-export all public types and functions
pub use core::*;

pub use utils::*;

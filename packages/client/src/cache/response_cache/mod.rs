//! Response cache modules
//!
//! Lock-free HTTP response cache implementation with zero-allocation patterns
//! and production-quality concurrent operations.
//!
//! The functionality is organized into logical modules:
//!
//! - `core`: ResponseCache struct and basic initialization
//! - `operations`: Cache get/put operations and HTTP semantics
//! - `eviction`: LRU eviction and expired entry cleanup
//!
//! All modules maintain lock-free concurrent access using crossbeam SkipMap.

pub mod core;
pub mod eviction;
pub mod operations;

// Re-export the main type for backward compatibility
pub use core::ResponseCache;



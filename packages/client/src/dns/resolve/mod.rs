//! DNS resolution module for HTTP3 client
//!
//! Provides zero-allocation DNS resolution with streaming architecture,
//! caching, timeout handling, and address preference sorting.
//!
//! This module implements RFC-compliant DNS resolution using the ystream
//! streaming architecture with error-as-data patterns.

pub mod resolvers;
pub mod traits;
pub mod types;
pub mod utilities;

// Re-export core types and traits for public API
pub use resolvers::DynResolver as Resolver;
pub use resolvers::{DnsResolverWithOverrides, DynResolver, GaiResolver};
pub use traits::{Addrs, Resolve, Resolving};
// Legacy compatibility exports to maintain existing API
pub use types::Name as NameType;
pub use types::{DnsResult, HyperName, Name};
pub use utilities::{
    DnsResolverWithOverridesImpl, is_ip_address, resolve_host_to_addrs,
    socket_addr_from_ip_literal, sort_addresses_by_preference, validate_hostname,
};



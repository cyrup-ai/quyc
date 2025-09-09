//! Core DNS types and traits for resolution
//!
//! Defines fundamental types like Name, DnsResult, and the Resolve trait
//! used throughout the DNS resolution system.

use std::net::SocketAddr;

use arrayvec::{ArrayVec, IntoIter as ArrayIntoIter};
use ystream::{AsyncStream, prelude::MessageChunk};

// Use ArrayVec-based iterator for zero-allocation
pub type SocketAddrIter = ArrayIntoIter<SocketAddr, 8>;

/// DNS name representation for hostname resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name(s.to_string())
    }
}

pub type HyperName = Name;

// Wrapper type for DNS resolution results to avoid orphan rule violations
#[derive(Debug)]
pub struct DnsResult {
    pub addrs: ArrayVec<SocketAddr, 8>,
}

impl DnsResult {
    pub fn new() -> Self {
        Self {
            addrs: ArrayVec::new(),
        }
    }

    pub fn from_vec(vec: Vec<SocketAddr>) -> Self {
        let mut addrs = ArrayVec::new();
        for addr in vec.into_iter().take(8) {
            if addrs.try_push(addr).is_err() {
                break;
            }
        }
        Self { addrs }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SocketAddr> {
        self.addrs.iter()
    }
}

impl MessageChunk for DnsResult {
    fn bad_chunk(_error: String) -> Self {
        Self::new()
    }

    fn is_error(&self) -> bool {
        self.addrs.is_empty()
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            Some("DNS resolution failed")
        } else {
            None
        }
    }
}

impl Default for DnsResult {
    fn default() -> Self {
        Self::new()
    }
}

/// An iterator of resolved socket addresses.
pub type Addrs = DnsResult;

/// Type alias for DNS resolution result streams.
pub type Resolving = AsyncStream<Addrs, 1024>;

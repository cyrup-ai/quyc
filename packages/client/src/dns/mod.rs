//! DNS resolution

pub use resolve::{Addrs, DynResolver, GaiResolver, Name, Resolve, Resolving};

pub(crate) mod gai;
pub(crate) mod hickory;
pub(crate) mod resolve;

// REMOVED: DnsResult<T> type alias - violates fluent-ai pure streaming architecture
// All DNS operations now return pure AsyncStream<T> where T implements MessageChunk

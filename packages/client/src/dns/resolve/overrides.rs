//! DNS resolver with hostname overrides
//!
//! Zero-allocation DNS resolver with hostname overrides for testing
//! and custom routing scenarios.

use ystream::{AsyncStream, emit, spawn_task};
use std::sync::Arc;
use std::net::SocketAddr;
use std::thread;

use super::traits::Resolve;
use super::types::{DnsResult, HyperName};

/// Zero-allocation DNS resolver with hostname overrides for testing and custom routing.
pub(crate) struct DnsResolverWithOverridesImpl {
    pub dns_resolver: Arc<dyn Resolve>,
    pub overrides: Arc<std::collections::HashMap<String, arrayvec::ArrayVec<SocketAddr, 8>>>,
}

impl Resolve for DnsResolverWithOverridesImpl {
    fn resolve(&self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        let hostname = name.as_str().to_string();
        let overrides = self.overrides.clone();
        let dns_resolver = self.dns_resolver.clone();
        
        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                // Check for override first
                if let Some(addrs) = overrides.get(&hostname) {
                    emit!(sender, DnsResult { addrs: addrs.clone() });
                    return;
                }
                
                // Fall back to actual DNS resolution
                let resolver_stream = dns_resolver.resolve(HyperName::from(hostname.clone()));
                spawn_task(move || {
                    for result in resolver_stream {
                        emit!(sender, result);
                    }
                });
            });
        })
    }
}
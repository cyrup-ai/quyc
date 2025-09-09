//! DNS resolution via the [hickory-resolver](https://github.com/hickory-dns/hickory-dns) crate

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use ystream::{AsyncStream, emit};
use hickory_resolver::{
    Resolver, ResolveError, config::{ResolverConfig, ResolverOpts, LookupIpStrategy}, 
    lookup_ip::LookupIpIntoIter,
};
use once_cell::sync::OnceCell;

use super::{Addrs, Name, Resolve, DnsResult};
use crate::prelude::*;

/// Iterator wrapper for hickory DNS results
pub struct SocketAddrs {
    pub iter: LookupIpIntoIter,
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Wrapper around a synchronous `Resolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub(crate) struct HickoryDnsResolver {
    /// Since we use synchronous Resolver, we delay the actual construction.
    state: Arc<OnceCell<Resolver>>,
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

#[derive(Debug)]
struct HickoryDnsSystemConfError(ResolveError);

impl Resolve for HickoryDnsResolver {
    fn resolve(&self, name: Name) -> AsyncStream<DnsResult, 1024> {
        let resolver = self.clone();
        let hostname = name.as_str().to_string();

        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                // Initialize resolver if needed
                let resolver_instance = match resolver.state.get_or_try_init(new_resolver) {
                    Ok(resolver) => resolver,
                    Err(e) => {
                        let error_msg = format!("Failed to initialize resolver: {}", e);
                        emit!(sender, DnsResult::bad_chunk(error_msg));
                        return;
                    }
                };

                // Perform synchronous DNS lookup
                match resolver_instance.lookup_ip(hostname.as_str()) {
                    Ok(lookup) => {
                        let socket_addrs: Vec<SocketAddr> = lookup.into_iter()
                            .map(|ip_addr| SocketAddr::new(ip_addr, 0))
                            .collect();
                        let dns_result = DnsResult::from_vec(socket_addrs);
                        emit!(sender, dns_result);
                    }
                    Err(e) => {
                        let error_msg = format!("DNS lookup failed: {}", e);
                        emit!(sender, DnsResult::bad_chunk(error_msg));
                    }
                }
            });
        })
    }
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

/// Create a new synchronous resolver with the default configuration,
/// which reads from `/etc/resolve.conf`. The options are
/// overridden to look up for both IPv4 and IPv6 addresses
/// to work with "happy eyeballs" algorithm.
fn new_resolver() -> Result<Resolver, HickoryDnsSystemConfError> {
    let config = ResolverConfig::default();
    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    
    Resolver::new(config, opts).map_err(HickoryDnsSystemConfError)
}

impl fmt::Display for HickoryDnsSystemConfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("error reading DNS system conf for hickory-dns")
    }
}

impl std::error::Error for HickoryDnsSystemConfError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

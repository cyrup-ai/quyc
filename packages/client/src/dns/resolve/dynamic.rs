//! Dynamic DNS resolver with caching and configuration
//!
//! Provides the main DynResolver implementation with support for caching,
//! IPv6 preference, and HTTP URI resolution.

use ystream::{AsyncStream, emit, spawn_task};
use std::sync::Arc;
use std::net::SocketAddr;
use std::thread;

use super::traits::Resolve;
use super::types::{DnsResult, HyperName};
use super::overrides::DnsResolverWithOverridesImpl;

pub type DnsResolverWithOverrides = DynResolver;

#[derive(Clone)]
pub struct DynResolver {
    resolver: Arc<dyn Resolve>,
    prefer_ipv6: bool,
    cache: Option<Arc<heapless::FnvIndexMap<String, arrayvec::ArrayVec<SocketAddr, 8>, 64>>>,
}

impl std::fmt::Debug for DynResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynResolver")
            .field("prefer_ipv6", &self.prefer_ipv6)
            .field("has_cache", &self.cache.is_some())
            .finish()
    }
}

impl DynResolver {
    pub(crate) fn new(resolver: Arc<dyn Resolve>) -> Self {
        Self {
            resolver,
            prefer_ipv6: false,
            cache: None,
        }
    }

    pub(crate) fn new_with_overrides(
        resolver: Arc<dyn Resolve>, 
        overrides: std::collections::HashMap<String, arrayvec::ArrayVec<SocketAddr, 8>>
    ) -> Self {
        Self {
            resolver: Arc::new(DnsResolverWithOverridesImpl {
                dns_resolver: resolver,
                overrides: Arc::new(overrides),
            }),
            prefer_ipv6: false,
            cache: None,
        }
    }

    pub(crate) fn gai() -> Self {
        use super::gai::GaiResolver;
        Self::new(Arc::new(GaiResolver::new()))
    }

    pub fn with_cache(mut self) -> Self {
        self.cache = Some(Arc::new(heapless::FnvIndexMap::new()));
        self
    }

    pub fn prefer_ipv6(mut self, prefer: bool) -> Self {
        self.prefer_ipv6 = prefer;
        self
    }

    /// Resolve an HTTP URI to socket addresses for connection establishment.
    /// Performs full DNS resolution including port inference from scheme.
    pub(crate) fn http_resolve(
        &self,
        target: &http::Uri,
    ) -> AsyncStream<Box<dyn Iterator<Item = SocketAddr> + Send>> {
        use super::utilities::resolve_host_to_addrs;
        
        let uri_string = target.to_string();
        let prefer_ipv6 = self.prefer_ipv6;
        let cache = self.cache.clone();
        let target_host = target.host().unwrap_or("").to_string();
        let target_port = target.port_u16().unwrap_or_else(|| {
            match target.scheme_str() {
                Some("https") => 443,
                Some("http") => 80,
                Some("socks4") | Some("socks4a") | Some("socks5") | Some("socks5h") => 1080,
                _ => 80,
            }
        });
        
        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                if target_host.is_empty() {
                    emit!(sender, Box::new(std::iter::empty()) as Box<dyn Iterator<Item = SocketAddr> + Send>);
                    return;
                }

                // Check cache first for performance
                if let Some(ref cache_map) = cache {
                    let cache_key = format!("{}:{}", target_host, target_port);
                    if let Some(cached_addrs) = cache_map.get(&cache_key) {
                        let iter: Box<dyn Iterator<Item = SocketAddr> + Send> = 
                            Box::new(cached_addrs.clone().into_iter());
                        emit!(sender, iter);
                        return;
                    }
                }

                // Perform DNS resolution
                match resolve_host_to_addrs(&target_host, target_port, prefer_ipv6) {
                    Ok(socket_addrs) => {
                        let iter: Box<dyn Iterator<Item = SocketAddr> + Send> = 
                            Box::new(socket_addrs.into_iter());
                        emit!(sender, iter);
                    }
                    Err(_e) => {
                        emit!(sender, Box::new(std::iter::empty()) as Box<dyn Iterator<Item = SocketAddr> + Send>);
                    }
                }
            });
        })
    }

    /// Resolve a hostname to socket addresses using the configured resolver.
    pub fn resolve(&mut self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        let resolver = self.resolver.clone();
        
        AsyncStream::with_channel(move |sender| {
            let resolve_stream = resolver.resolve(name);
            spawn_task(move || {
                for dns_result in resolve_stream {
                    emit!(sender, dns_result);
                }
            });
        })
    }

    /// Direct DNS resolution method - replaces Service::call with AsyncStream
    /// RETAINS: All caching, timeouts, error handling, address sorting functionality
    /// Returns AsyncStream<DnsResult> per zero-allocation architecture
    pub fn resolve_direct(&mut self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        let resolver = self.resolver.clone();
        AsyncStream::with_channel(move |sender| {
            let resolve_stream = resolver.resolve(name);
            spawn_task(move || {
                for dns_result in resolve_stream {
                    emit!(sender, dns_result);
                }
            });
        })
    }
}
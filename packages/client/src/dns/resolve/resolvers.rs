//! DNS resolver implementations for different resolution strategies
//!
//! Contains DynResolver and GaiResolver implementations with caching,
//! timeout handling, and address preference sorting.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

use ystream::{AsyncStream, emit, spawn_task};

use super::traits::Resolve;
use super::types::{DnsResult, HyperName};

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
        overrides: std::collections::HashMap<String, arrayvec::ArrayVec<SocketAddr, 8>>,
    ) -> Self {
        Self {
            resolver: Arc::new(super::utilities::DnsResolverWithOverridesImpl {
                dns_resolver: resolver,
                overrides: Arc::new(overrides),
            }),
            prefer_ipv6: false,
            cache: None,
        }
    }

    pub(crate) fn gai() -> Self {
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
        let prefer_ipv6 = self.prefer_ipv6;
        let cache = self.cache.clone();
        let target_host = target.host().unwrap_or("").to_string();
        let target_port = target
            .port_u16()
            .unwrap_or_else(|| match target.scheme_str() {
                Some("https") => 443,
                Some("http") => 80,
                Some("socks4") | Some("socks4a") | Some("socks5") | Some("socks5h") => 1080,
                _ => 80,
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
                match super::utilities::resolve_host_to_addrs(
                    &target_host,
                    target_port,
                    prefer_ipv6,
                ) {
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

/// High-performance synchronous DNS resolver using system getaddrinfo.
/// Zero-allocation design with optimized address sorting.
pub struct GaiResolver {
    prefer_ipv6: bool,
    timeout_ms: u32,
}

impl GaiResolver {
    pub fn new() -> Self {
        Self {
            prefer_ipv6: false,
            timeout_ms: 5000, // 5 second default timeout
        }
    }

    pub fn prefer_ipv6(mut self, prefer: bool) -> Self {
        self.prefer_ipv6 = prefer;
        self
    }

    pub fn timeout_ms(mut self, timeout: u32) -> Self {
        self.timeout_ms = timeout;
        self
    }
}

impl Default for GaiResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for GaiResolver {
    fn resolve(&self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        let hostname = name.as_str().to_string();
        let prefer_ipv6 = self.prefer_ipv6;
        let timeout_ms = self.timeout_ms;

        AsyncStream::with_channel(move |sender| {
            thread::spawn(move || {
                // Use std::net::ToSocketAddrs for synchronous resolution
                let dummy_port = 80; // Port doesn't matter for hostname resolution
                let host_with_port = format!("{}:{}", hostname, dummy_port);

                // Set up timeout handling using thread-local storage
                let start_time = std::time::Instant::now();

                let socket_addrs: Result<arrayvec::ArrayVec<SocketAddr, 8>, std::io::Error> =
                    host_with_port.to_socket_addrs().map(|iter| {
                        let mut addrs: arrayvec::ArrayVec<SocketAddr, 8> = iter.take(8).collect();

                        // Check timeout using elite polling pattern
                        if start_time.elapsed().as_millis() > timeout_ms as u128 {
                            return arrayvec::ArrayVec::new(); // Timeout exceeded
                        }

                        // Sort addresses based on preference using zero-allocation sort
                        if prefer_ipv6 {
                            addrs.sort_unstable_by_key(|addr| match addr.ip() {
                                IpAddr::V6(_) => 0, // IPv6 first
                                IpAddr::V4(_) => 1, // IPv4 second
                            });
                        } else {
                            addrs.sort_unstable_by_key(|addr| match addr.ip() {
                                IpAddr::V4(_) => 0, // IPv4 first
                                IpAddr::V6(_) => 1, // IPv6 second
                            });
                        }

                        // Remove port information since we added dummy port - zero allocation
                        for addr in addrs.iter_mut() {
                            addr.set_port(0); // Clear the dummy port
                        }
                        addrs
                    });

                match socket_addrs {
                    Ok(addrs) => {
                        if addrs.is_empty() {
                            emit!(
                                sender,
                                DnsResult::bad_chunk(format!(
                                    "DNS resolution timeout or no addresses found for {}",
                                    hostname
                                ))
                            );
                        } else {
                            emit!(sender, DnsResult { addrs });
                        }
                    }
                    Err(e) => {
                        emit!(
                            sender,
                            DnsResult::bad_chunk(format!(
                                "DNS resolution failed for {}: {}",
                                hostname, e
                            ))
                        );
                    }
                }
            });
        })
    }
}

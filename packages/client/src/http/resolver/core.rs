//! Main DNS resolver implementation
//!
//! This module provides the primary Resolver struct and public API for DNS resolution.

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use ystream::prelude::*;
use ystream::thread_pool::global_executor;
use tracing::{debug, error};

use super::{
    cache::{DnsCache, DnsCacheEntry},
    config::{CacheConfig, RetryConfig},
    engine::ResolutionEngine,
    rate_limiter::RateLimiter,
    stats::ResolverStats,
    validation::validate_hostname,
};
use crate::prelude::*;
use std::sync::Arc;

/// Resolved DNS address with hostname reference
#[derive(Debug, Clone)]
pub struct ResolvedAddress {
    pub address: SocketAddr,
    pub hostname: Arc<str>,
}

impl ResolvedAddress {
    pub fn new(address: SocketAddr, hostname: Arc<str>) -> Self {
        Self { address, hostname }
    }
}

impl ystream::prelude::MessageChunk for ResolvedAddress {
    fn bad_chunk(error: String) -> Self {
        // Create an invalid address for error cases
        use std::net::{IpAddr, Ipv4Addr};
        Self {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            hostname: Arc::from(error),
        }
    }

    fn is_error(&self) -> bool {
        // Consider 0.0.0.0:0 as error address
        self.address.ip().is_unspecified() && self.address.port() == 0
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            Some(&self.hostname)
        } else {
            None
        }
    }
}

impl Default for ResolvedAddress {
    fn default() -> Self {
        use std::net::{IpAddr, Ipv4Addr};
        Self {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80),
            hostname: Arc::from("localhost"),
        }
    }
}

/// Production-quality DNS resolver with integrated client resource protection
#[derive(Debug, Clone)]
pub struct Resolver {
    timeout: Duration,
    ipv6_preference: bool,

    // DNS response cache with TTL
    dns_cache: Arc<DnsCache>,

    // Resolution engine
    engine: Arc<ResolutionEngine>,
    
    // Rate limiter for DNS queries
    rate_limiter: Arc<RateLimiter>,

    // Lock-free performance counters
    request_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU64>,
    failure_count: Arc<AtomicU64>,

    // Integrated canonical HTTP client for resource protection
    #[allow(dead_code)]
    http_client: Arc<HttpClient>,
}

impl Resolver {
    /// Create new resolver with default timeout
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            ipv6_preference: false,
            dns_cache: Arc::new(DnsCache::new(CacheConfig::default())),
            engine: Arc::new(ResolutionEngine::new(RetryConfig::default())),
            rate_limiter: Arc::new(RateLimiter::new(100)), // 100 requests per second default
            request_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
            http_client: Arc::new(HttpClient::with_config(crate::HttpConfig::default())),
        }
    }

    /// Create resolver with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::new()
        }
    }

    /// Configure IPv6 preference
    pub fn with_ipv6_preference(mut self, prefer_ipv6: bool) -> Self {
        self.ipv6_preference = prefer_ipv6;
        self
    }

    /// Configure DNS retry behavior
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.engine = Arc::new(ResolutionEngine::new(retry_config));
        self
    }

    /// Configure DNS response caching
    pub fn with_cache_config(mut self, cache_config: CacheConfig) -> Self {
        self.dns_cache = Arc::new(DnsCache::new(cache_config));
        self
    }

    /// Resolve hostname to IP addresses using production DNS resolution
    pub fn resolve(&self, hostname: &str, port: u16) -> AsyncStream<ResolvedAddress, 1024> {
        // Input validation
        if let Err(err) = validate_hostname(hostname) {
            return AsyncStream::with_channel(move |sender| {
                emit!(sender, ResolvedAddress::bad_chunk(err));
            });
        }

        // Check DNS cache first
        let cache_key = format!("{}:{}", hostname, port);
        if let Some(entry) = self.dns_cache.get(&cache_key) {
            debug!("DNS cache hit for {}:{}", hostname, port);
            let hostname_arc: Arc<str> = Arc::from(hostname);
            let addresses = entry.addresses;
            return AsyncStream::with_channel(move |sender| {
                for addr in addresses {
                    emit!(sender, ResolvedAddress::new(addr, hostname_arc.clone()));
                }
            });
        }

        // Increment request count for statistics (cache misses only)
        self.request_count.fetch_add(1, Ordering::Relaxed);

        // Rate limiting check - query type reflects IPv6 preference for protection stats
        let query_type = if self.ipv6_preference { "AAAA" } else { "A" };
        if let Err(err) = self.check_rate_limit(hostname, query_type) {
            return AsyncStream::with_channel(move |sender| {
                emit!(sender, ResolvedAddress::bad_chunk(err));
            });
        }

        let hostname: Arc<str> = Arc::from(hostname);
        let timeout = self.timeout;
        let ipv6_preference = self.ipv6_preference;
        let dns_cache = Arc::clone(&self.dns_cache);
        let engine = Arc::clone(&self.engine);
        let success_count = Arc::clone(&self.success_count);
        let failure_count = Arc::clone(&self.failure_count);

        debug!("Starting DNS resolution for {}:{}", hostname, port);

        AsyncStream::with_channel(move |sender| {
            // Use global thread pool instead of spawning new threads
            global_executor().execute(move || {
                // IP address fast path - avoid DNS lookup
                if let Ok(ip) = IpAddr::from_str(&hostname) {
                    debug!("IP address fast path for {}", hostname);
                    let addr = SocketAddr::new(ip, port);
                    success_count.fetch_add(1, Ordering::Relaxed);
                    emit!(sender, ResolvedAddress::new(addr, hostname));
                    return;
                }

                // Perform real DNS resolution with timeout and retry logic
                match engine.resolve_with_timeout_and_retry(
                    &hostname,
                    port,
                    timeout,
                    ipv6_preference,
                ) {
                    Ok(addresses) => {
                        if addresses.is_empty() {
                            failure_count.fetch_add(1, Ordering::Relaxed);
                            error!("No addresses resolved for {}", hostname);
                            emit!(
                                sender,
                                ResolvedAddress::bad_chunk(format!(
                                    "No addresses found for hostname: {}",
                                    hostname
                                ))
                            );
                        } else {
                            success_count.fetch_add(1, Ordering::Relaxed);
                            debug!("Resolved {} to {} addresses", hostname, addresses.len());

                            // Store in cache
                            let cache_key = format!("{}:{}", hostname, port);
                            let cache_entry =
                                DnsCacheEntry::new(addresses.clone(), dns_cache.config.ttl_secs);
                            dns_cache.insert(cache_key, cache_entry);

                            // Emit individual address chunks
                            for addr in addresses {
                                emit!(sender, ResolvedAddress::new(addr, hostname.clone()));
                            }
                        }
                    }
                    Err(err) => {
                        failure_count.fetch_add(1, Ordering::Relaxed);
                        error!("DNS resolution failed for {}: {}", hostname, err);
                        emit!(
                            sender,
                            ResolvedAddress::bad_chunk(format!(
                                "DNS resolution failed for {}: {}",
                                hostname, err
                            ))
                        );
                    }
                }
            });
        })
    }
    /// Check rate limiting using integrated protection system
    fn check_rate_limit(&self, hostname: &str, query_type: &str) -> Result<(), String> {
        // Use the rate limiter
        self.rate_limiter.check_rate_limit(hostname, query_type)
    }

    /// Get resolver statistics
    pub fn stats(&self) -> ResolverStats {
        ResolverStats::new(
            self.request_count.load(Ordering::Acquire),
            self.success_count.load(Ordering::Acquire),
            self.failure_count.load(Ordering::Acquire),
            self.timeout,
        )
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

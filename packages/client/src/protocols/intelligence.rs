//! Domain protocol intelligence cache
//!
//! Per-domain protocol capability tracking with lock-free atomic operations.
//! Maintains hot cache of domains and their supported protocols ensuring we never
//! try the incorrect protocol twice for a domain.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::protocols::core::HttpVersion;

/// Domain protocol intelligence cache with atomic operations for lock-free access
///
/// Tracks protocol capabilities, success/failure rates, and last attempt timestamps
/// for each domain to enable intelligent protocol selection and prevent retrying
/// failed protocols.
#[derive(Debug)]
pub struct ProtocolIntelligence {
    /// Domain capability cache with atomic access
    domains: Arc<RwLock<HashMap<String, Arc<DomainCapabilities>>>>,
    /// Global statistics for cache performance
    stats: ProtocolIntelligenceStats,
    /// Cache configuration
    config: IntelligenceConfig,
}

/// Per-domain protocol capabilities with atomic tracking
#[derive(Debug)]
pub struct DomainCapabilities {
    /// Domain name
    pub domain: String,
    /// HTTP/3 support tracking
    pub h3_support: AtomicProtocolSupport,
    /// HTTP/2 support tracking
    pub h2_support: AtomicProtocolSupport,
    /// HTTP/1.1 support tracking (always assumed available)
    pub h1_support: AtomicProtocolSupport,
    /// Alt-Svc discovered endpoints
    pub alt_svc_endpoints: Arc<RwLock<HashMap<String, AltSvcEndpoint>>>,
    /// Last successful protocol used
    pub last_successful_protocol: Arc<RwLock<Option<HttpVersion>>>,
    /// Domain discovery timestamp
    pub discovered_at: SystemTime,
    /// Last update timestamp
    pub last_updated: Arc<RwLock<SystemTime>>,
}

/// Atomic protocol support tracking
#[derive(Debug)]
pub struct AtomicProtocolSupport {
    /// Whether protocol is supported (None=unknown, Some(true)=supported, Some(false)=not supported)
    pub is_supported: AtomicBool,
    /// Whether support status is known
    pub is_known: AtomicBool,
    /// Success count
    pub success_count: AtomicUsize,
    /// Failure count
    pub failure_count: AtomicUsize,
    /// Last attempt timestamp (nanoseconds since `UNIX_EPOCH`)
    pub last_attempt: AtomicU64,
    /// Last success timestamp (nanoseconds since `UNIX_EPOCH`)
    pub last_success: AtomicU64,
}

/// Global protocol intelligence statistics
#[derive(Debug)]
pub struct ProtocolIntelligenceStats {
    /// Total domains tracked
    pub domains_tracked: AtomicUsize,
    /// Cache hits
    pub cache_hits: AtomicUsize,
    /// Cache misses
    pub cache_misses: AtomicUsize,
    /// Protocol discoveries
    pub protocol_discoveries: AtomicUsize,
    /// Failed protocol attempts prevented
    pub failed_attempts_prevented: AtomicUsize,
}

/// Configuration for protocol intelligence cache
#[derive(Debug, Clone)]
pub struct IntelligenceConfig {
    /// Maximum domains to track in cache
    pub max_domains: usize,
    /// Time before retrying a failed protocol
    pub retry_after_failure: Duration,
    /// Time before considering cached data stale
    pub cache_expiry: Duration,
    /// Minimum attempts before marking protocol as unsupported
    pub min_attempts_for_failure: usize,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            max_domains: 10000,
            retry_after_failure: Duration::from_secs(300), // 5 minutes
            cache_expiry: Duration::from_secs(3600),       // 1 hour
            min_attempts_for_failure: 3,
        }
    }
}

/// Alt-Svc endpoint validation status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AltSvcValidationStatus {
    /// Endpoint not yet tested
    Unknown,
    /// Endpoint successfully validated
    Valid,
    /// Endpoint failed validation
    Invalid,
    /// Endpoint expired per max-age
    Expired,
}

/// Alt-Svc discovered endpoint with RFC 7838 compliance
#[derive(Debug, Clone)]
pub struct AltSvcEndpoint {
    /// Protocol identifier ("h3", "h2", etc.)
    pub protocol: String,
    /// Alternative host (None means same host as original)
    pub host: Option<String>,
    /// Alternative port
    pub port: u16,
    /// Max age in seconds from Alt-Svc header
    pub max_age: Duration,
    /// When this endpoint was discovered
    pub discovered_at: SystemTime,
    /// Last successful validation timestamp
    pub last_validated: Option<SystemTime>,
    /// Current validation status
    pub validation_status: AltSvcValidationStatus,
}

impl AltSvcEndpoint {
    /// Check if endpoint has expired based on `max_age`
    #[must_use] 
    pub fn is_expired(&self) -> bool {
        match self.discovered_at.elapsed() {
            Ok(elapsed) => elapsed > self.max_age,
            Err(_) => true, // Clock issues, consider expired
        }
    }
    
    /// Check if endpoint is currently valid for use
    #[must_use] 
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.validation_status == AltSvcValidationStatus::Valid
    }
    
    /// Update validation status
    pub fn set_validation_status(&mut self, status: AltSvcValidationStatus) {
        self.validation_status = status;
        if status == AltSvcValidationStatus::Valid {
            self.last_validated = Some(SystemTime::now());
        }
    }
}

impl Default for AtomicProtocolSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicProtocolSupport {
    /// Create new atomic protocol support tracker
    #[must_use] 
    pub fn new() -> Self {
        Self {
            is_supported: AtomicBool::new(false),
            is_known: AtomicBool::new(false),
            success_count: AtomicUsize::new(0),
            failure_count: AtomicUsize::new(0),
            last_attempt: AtomicU64::new(0),
            last_success: AtomicU64::new(0),
        }
    }

    /// Mark protocol as supported
    pub fn mark_supported(&self) {
        self.is_supported.store(true, Ordering::Relaxed);
        self.is_known.store(true, Ordering::Relaxed);
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.last_success.store(current_timestamp_nanos(), Ordering::Relaxed);
        self.last_attempt.store(current_timestamp_nanos(), Ordering::Relaxed);
    }

    /// Mark protocol as not supported
    pub fn mark_not_supported(&self) {
        self.is_supported.store(false, Ordering::Relaxed);
        self.is_known.store(true, Ordering::Relaxed);
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.last_attempt.store(current_timestamp_nanos(), Ordering::Relaxed);
    }

    /// Check if protocol is known to be supported
    pub fn is_supported(&self) -> Option<bool> {
        if self.is_known.load(Ordering::Relaxed) {
            Some(self.is_supported.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let successes = self.success_count.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let total = successes + failures;

        if total == 0 {
            0.0
        } else {
            // Precision loss acceptable for protocol success rate statistics
            #[allow(clippy::cast_precision_loss)]
            { successes as f64 / total as f64 }
        }
    }

    /// Check if enough time has passed since last failure to retry
    pub fn can_retry_after_failure(&self, retry_duration: Duration) -> bool {
        let last_attempt = self.last_attempt.load(Ordering::Relaxed);
        let now = current_timestamp_nanos();
        let elapsed_nanos = now.saturating_sub(last_attempt);
        let elapsed_duration = Duration::from_nanos(elapsed_nanos);

        elapsed_duration >= retry_duration
    }
}

impl DomainCapabilities {
    /// Create new domain capabilities tracker
    #[must_use] 
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            h3_support: AtomicProtocolSupport::new(),
            h2_support: AtomicProtocolSupport::new(),
            h1_support: AtomicProtocolSupport::new(),
            alt_svc_endpoints: Arc::new(RwLock::new(HashMap::new())),
            last_successful_protocol: Arc::new(RwLock::new(None)),
            discovered_at: SystemTime::now(),
            last_updated: Arc::new(RwLock::new(SystemTime::now())),
        }
    }

    /// Get protocol support for specific version
    pub fn get_protocol_support(&self, version: HttpVersion) -> &AtomicProtocolSupport {
        match version {
            HttpVersion::Http3 => &self.h3_support,
            HttpVersion::Http2 => &self.h2_support,
        }
    }

    /// Track successful protocol usage
    pub fn track_success(&self, protocol: HttpVersion) {
        self.get_protocol_support(protocol).mark_supported();
        
        // Update last successful protocol
        if let Ok(mut last_successful) = self.last_successful_protocol.write() {
            *last_successful = Some(protocol);
        }
        
        // Update last updated timestamp
        if let Ok(mut last_updated) = self.last_updated.write() {
            *last_updated = SystemTime::now();
        }
    }

    /// Track failed protocol attempt
    pub fn track_failure(&self, protocol: HttpVersion) {
        self.get_protocol_support(protocol).mark_not_supported();
        
        // Update last updated timestamp
        if let Ok(mut last_updated) = self.last_updated.write() {
            *last_updated = SystemTime::now();
        }
    }

    /// Get preferred protocol order based on historical success
    pub fn get_preferred_protocols(&self) -> Vec<HttpVersion> {
        let mut protocols = vec![
            (HttpVersion::Http3, self.h3_support.success_rate()),
            (HttpVersion::Http2, self.h2_support.success_rate()),
        ];

        // Sort by success rate (descending)
        protocols.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        protocols.into_iter().map(|(version, _)| version).collect()
    }

    /// Check if protocol should be retried based on failure history
    pub fn should_retry_protocol(&self, protocol: HttpVersion, config: &IntelligenceConfig) -> bool {
        let support = self.get_protocol_support(protocol);
        
        // If protocol is known to be supported, always retry
        if let Some(true) = support.is_supported() {
            return true;
        }

        // If protocol is known to be unsupported, check retry conditions
        if let Some(false) = support.is_supported() {
            let failure_count = support.failure_count.load(Ordering::Relaxed);
            
            // If we haven't reached minimum attempts threshold, allow retry
            if failure_count < config.min_attempts_for_failure {
                return true;
            }
            
            // Check if enough time has passed since last failure
            return support.can_retry_after_failure(config.retry_after_failure);
        }

        // Protocol support is unknown, allow attempt
        true
    }
    
    /// Update Alt-Svc endpoints from RFC 7838 header value
    pub fn update_alt_svc_endpoints(&self, alt_svc_header: &str) -> Result<(), String> {
        let endpoints = Self::parse_alt_svc_header(alt_svc_header)?;
        
        let mut alt_svc_map = match self.alt_svc_endpoints.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    target: "quyc::protocols::intelligence",
                    domain = %self.domain,
                    "Alt-Svc endpoints mutex poisoned, recovering"
                );
                poisoned.into_inner()
            }
        };
        
        for endpoint in endpoints {
            let key = format!("{}:{}", endpoint.protocol, endpoint.port);
            alt_svc_map.insert(key, endpoint);
        }
        
        // Update last updated timestamp
        if let Ok(mut last_updated) = self.last_updated.write() {
            *last_updated = SystemTime::now();
        }
        
        Ok(())
    }
    
    /// Get all valid (non-expired, validated) Alt-Svc endpoints
    pub fn get_valid_alt_svc_endpoints(&self) -> Vec<AltSvcEndpoint> {
        let alt_svc_map = match self.alt_svc_endpoints.read() {
            Ok(guard) => guard,
            Err(_poisoned) => {
                tracing::error!(
                    target: "quyc::protocols::intelligence",
                    domain = %self.domain,
                    "Alt-Svc endpoints read mutex poisoned, returning empty"
                );
                return Vec::new();
            }
        };
        
        alt_svc_map
            .values()
            .filter(|endpoint| endpoint.is_valid())
            .cloned()
            .collect()
    }
    
    /// Validate Alt-Svc endpoint format and feasibility (does not perform connection testing)
    pub fn validate_alt_svc_endpoint(&self, endpoint: &mut AltSvcEndpoint) -> bool {
        // Check if endpoint has expired
        if endpoint.is_expired() {
            endpoint.set_validation_status(AltSvcValidationStatus::Expired);
            tracing::debug!(
                target: "quyc::protocols::intelligence",
                domain = %self.domain,
                protocol = %endpoint.protocol,
                port = endpoint.port,
                "Alt-Svc endpoint expired"
            );
            return false;
        }
        
        // Validate protocol support
        if !matches!(endpoint.protocol.as_str(), "h3" | "h2" | "h1") {
            endpoint.set_validation_status(AltSvcValidationStatus::Invalid);
            tracing::warn!(
                target: "quyc::protocols::intelligence",
                domain = %self.domain,
                protocol = %endpoint.protocol,
                port = endpoint.port,
                "Unsupported Alt-Svc protocol"
            );
            return false;
        }
        
        // Validate port range
        if endpoint.port == 0 {
            endpoint.set_validation_status(AltSvcValidationStatus::Invalid);
            tracing::warn!(
                target: "quyc::protocols::intelligence",
                domain = %self.domain,
                protocol = %endpoint.protocol,
                port = endpoint.port,
                "Invalid Alt-Svc port: zero not allowed"
            );
            return false;
        }
        
        // Validate that test URL can be constructed
        match self.build_alt_svc_test_url(endpoint) {
            Ok(_) => {
                // Format validation successful - mark as unknown pending connection test
                endpoint.set_validation_status(AltSvcValidationStatus::Unknown);
                
                tracing::debug!(
                    target: "quyc::protocols::intelligence",
                    domain = %self.domain,
                    protocol = %endpoint.protocol,
                    port = endpoint.port,
                    "Alt-Svc endpoint format validation successful"
                );
                
                true
            },
            Err(e) => {
                endpoint.set_validation_status(AltSvcValidationStatus::Invalid);
                tracing::error!(
                    target: "quyc::protocols::intelligence",
                    domain = %self.domain,
                    protocol = %endpoint.protocol,
                    port = endpoint.port,
                    error = %e,
                    "Failed to build test URL for Alt-Svc endpoint"
                );
                false
            }
        }
    }
    
    /// Parse RFC 7838 Alt-Svc header value
    fn parse_alt_svc_header(header_value: &str) -> Result<Vec<AltSvcEndpoint>, String> {
        let mut endpoints = Vec::new();
        
        // Handle "clear" directive
        if header_value.trim().eq_ignore_ascii_case("clear") {
            return Ok(endpoints); // Return empty list to clear endpoints
        }
        
        // Split by comma to handle multiple Alt-Svc entries
        for entry in header_value.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            
            // Parse entry format: protocol="host:port"; ma=max_age; other=params
            let parts: Vec<&str> = entry.splitn(2, ';').collect();
            if parts.is_empty() {
                continue;
            }
            
            let protocol_part = parts[0].trim();
            
            // Extract protocol and endpoint
            let (protocol, host, port) = Self::parse_protocol_endpoint(protocol_part)?;
            
            // Parse parameters (ma=max_age, etc.)
            let mut max_age = Duration::from_secs(86400); // Default 24 hours
            
            if parts.len() > 1 {
                for param in parts[1].split(';') {
                    let param = param.trim();
                    if param.starts_with("ma=")
                        && let Ok(seconds) = param[3..].parse::<u64>() {
                            max_age = Duration::from_secs(seconds);
                        }
                }
            }
            
            endpoints.push(AltSvcEndpoint {
                protocol,
                host,
                port,
                max_age,
                discovered_at: SystemTime::now(),
                last_validated: None,
                validation_status: AltSvcValidationStatus::Unknown,
            });
        }
        
        Ok(endpoints)
    }
    
    /// Parse protocol and endpoint from Alt-Svc entry
    fn parse_protocol_endpoint(protocol_part: &str) -> Result<(String, Option<String>, u16), String> {
        let eq_pos = protocol_part.find('=')
            .ok_or("Invalid Alt-Svc format: missing '=' separator")?;
        
        let protocol = protocol_part[..eq_pos].trim().to_string();
        let endpoint = protocol_part[eq_pos + 1..].trim();
        
        // Remove quotes if present
        let endpoint = if endpoint.starts_with('"') && endpoint.ends_with('"') {
            &endpoint[1..endpoint.len() - 1]
        } else {
            endpoint
        };
        
        // Parse endpoint - can be ":port" or "host:port"
        if let Some(port_str) = endpoint.strip_prefix(':') {
            // Same host, different port
            let port = port_str.parse::<u16>()
                .map_err(|_| format!("Invalid port in Alt-Svc: {port_str}"))?;
            Ok((protocol, None, port))
        } else {
            // Different host and port
            let colon_pos = endpoint.rfind(':')
                .ok_or("Invalid Alt-Svc endpoint: missing port")?;
            
            let host = endpoint[..colon_pos].trim().to_string();
            let port_str = &endpoint[colon_pos + 1..];
            let port = port_str.parse::<u16>()
                .map_err(|_| format!("Invalid port in Alt-Svc: {port_str}"))?;
            
            Ok((protocol, Some(host), port))
        }
    }
    
    /// Build test URL for Alt-Svc endpoint validation
    fn build_alt_svc_test_url(&self, endpoint: &AltSvcEndpoint) -> Result<String, String> {
        use crate::http::url::{parse_url};
        
        // Create base URL for this domain
        let base_url = format!("https://{}", self.domain);
        let mut url = parse_url(&base_url)
            .map_err(|_| "Failed to parse base URL for Alt-Svc test")?;
        
        // Set alternative host if specified
        if let Some(ref alt_host) = endpoint.host {
            url.set_host(Some(alt_host))
                .map_err(|_| "Failed to set alternative host for Alt-Svc test")?;
        }
        
        // Set alternative port
        url.set_port(Some(endpoint.port))
            .map_err(|()| "Failed to set alternative port for Alt-Svc test")?;
        
        Ok(url.to_string())
    }
}

impl ProtocolIntelligence {
    /// Create new protocol intelligence cache
    #[must_use] 
    pub fn new() -> Self {
        Self::with_config(IntelligenceConfig::default())
    }

    /// Create protocol intelligence cache with custom configuration
    #[must_use] 
    pub fn with_config(config: IntelligenceConfig) -> Self {
        Self {
            domains: Arc::new(RwLock::new(HashMap::new())),
            stats: ProtocolIntelligenceStats {
                domains_tracked: AtomicUsize::new(0),
                cache_hits: AtomicUsize::new(0),
                cache_misses: AtomicUsize::new(0),
                protocol_discoveries: AtomicUsize::new(0),
                failed_attempts_prevented: AtomicUsize::new(0),
            },
            config,
        }
    }

    /// Track successful protocol usage for domain
    pub fn track_success(&self, domain: &str, protocol: HttpVersion) {
        let capabilities = self.get_or_create_domain_capabilities(domain);
        capabilities.track_success(protocol);
        self.stats.protocol_discoveries.fetch_add(1, Ordering::Relaxed);
    }

    /// Track failed protocol attempt for domain
    pub fn track_failure(&self, domain: &str, protocol: HttpVersion) {
        let capabilities = self.get_or_create_domain_capabilities(domain);
        capabilities.track_failure(protocol);
    }

    /// Get preferred protocol for domain based on historical data
    pub fn get_preferred_protocol(&self, domain: &str) -> HttpVersion {
        if let Some(capabilities) = self.get_domain_capabilities(domain) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            let preferred_protocols = capabilities.get_preferred_protocols();
            
            // Return first protocol that should be retried
            for protocol in preferred_protocols {
                if capabilities.should_retry_protocol(protocol, &self.config) {
                    return protocol;
                }
            }
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Default fallback: HTTP/3 -> HTTP/2 -> HTTP/1.1
        HttpVersion::Http3
    }

    /// Check if protocol should be retried for domain
    pub fn should_retry_protocol(&self, domain: &str, protocol: HttpVersion) -> bool {
        if let Some(capabilities) = self.get_domain_capabilities(domain) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            let should_retry = capabilities.should_retry_protocol(protocol, &self.config);
            
            if !should_retry {
                self.stats.failed_attempts_prevented.fetch_add(1, Ordering::Relaxed);
            }
            
            should_retry
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
            true // Allow attempt for unknown domains
        }
    }

    /// Get domain capabilities (internal)
    fn get_domain_capabilities(&self, domain: &str) -> Option<Arc<DomainCapabilities>> {
        self.domains.read().ok()?.get(domain).cloned()
    }

    /// Get or create domain capabilities (internal)
    fn get_or_create_domain_capabilities(&self, domain: &str) -> Arc<DomainCapabilities> {
        // Fast path: try to get existing capabilities
        if let Some(capabilities) = self.get_domain_capabilities(domain) {
            return capabilities;
        }

        // Slow path: create new capabilities
        let mut domains = match self.domains.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    target: "quyc::protocols::intelligence",
                    domain = %domain,
                    "Domain capabilities cache mutex poisoned, recovering"
                );
                poisoned.into_inner()
            }
        };
        
        // Double-check after acquiring write lock
        if let Some(capabilities) = domains.get(domain) {
            return capabilities.clone();
        }

        // Create new capabilities
        let capabilities = Arc::new(DomainCapabilities::new(domain.to_string()));
        domains.insert(domain.to_string(), capabilities.clone());
        self.stats.domains_tracked.fetch_add(1, Ordering::Relaxed);

        // Enforce cache size limit
        if domains.len() > self.config.max_domains {
            self.evict_oldest_domains(&mut domains);
        }

        capabilities
    }

    /// Evict oldest domains to maintain cache size limit
    fn evict_oldest_domains(&self, domains: &mut HashMap<String, Arc<DomainCapabilities>>) {
        // Remove 10% of cache when limit is exceeded
        // Precision loss acceptable for cache size calculations
        #[allow(clippy::cast_precision_loss)]
        let target_size = (self.config.max_domains as f64 * 0.9) as usize;
        let to_remove = domains.len().saturating_sub(target_size);

        if to_remove == 0 {
            return;
        }

        // Collect domains with their last updated times
        let mut domain_times: Vec<(String, SystemTime)> = domains
            .iter()
            .filter_map(|(domain, capabilities)| {
                capabilities.last_updated.read().ok()
                    .map(|last_updated| (domain.clone(), *last_updated))
            })
            .collect();

        // Sort by last updated time (oldest first)
        domain_times.sort_by_key(|(_, time)| *time);

        // Remove oldest domains
        for (domain, _) in domain_times.into_iter().take(to_remove) {
            domains.remove(&domain);
        }
    }

    /// Update Alt-Svc endpoints for domain from response header
    /// 
    /// Parses RFC 7838 Alt-Svc header and updates domain capabilities with discovered endpoints.
    pub fn update_alt_svc_for_domain(&self, domain: &str, alt_svc_header: &str) -> Result<(), String> {
        let capabilities = self.get_or_create_domain_capabilities(domain);
        capabilities.update_alt_svc_endpoints(alt_svc_header)
    }

    /// Get valid Alt-Svc endpoints for domain
    /// 
    /// Returns all non-expired, validated Alt-Svc endpoints for the specified domain.
    pub fn get_alt_svc_endpoints_for_domain(&self, domain: &str) -> Vec<AltSvcEndpoint> {
        if let Some(capabilities) = self.get_domain_capabilities(domain) {
            capabilities.get_valid_alt_svc_endpoints()
        } else {
            Vec::new()
        }
    }

    /// Update Alt-Svc endpoint validation status after connection testing
    ///
    /// Updates the validation status of a specific Alt-Svc endpoint for a domain.
    /// This is called after attempting to validate an Alt-Svc endpoint to record
    /// whether the connection attempt was successful or failed.
    pub fn update_alt_svc_endpoint_validation_status(
        &self, 
        domain: &str, 
        protocol: &str, 
        port: u16, 
        status: AltSvcValidationStatus
    ) -> Result<(), String> {
        let capabilities = self.get_or_create_domain_capabilities(domain);
        
        let mut alt_svc_map = match capabilities.alt_svc_endpoints.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    target: "quyc::protocols::intelligence",
                    domain = %domain,
                    "Alt-Svc endpoints mutex poisoned during status update, recovering"
                );
                poisoned.into_inner()
            }
        };
        
        let key = format!("{protocol}:{port}");
        
        if let Some(endpoint) = alt_svc_map.get_mut(&key) {
            endpoint.set_validation_status(status);
            
            tracing::debug!(
                target: "quyc::protocols::intelligence",
                domain = %domain,
                protocol = %protocol,
                port = port,
                status = ?status,
                "Alt-Svc endpoint validation status updated"
            );
            
            // Update last updated timestamp for the domain
            if let Ok(mut last_updated) = capabilities.last_updated.write() {
                *last_updated = SystemTime::now();
            }
            
            Ok(())
        } else {
            Err(format!(
                "Alt-Svc endpoint {protocol}:{port} not found for domain {domain}"
            ))
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> ProtocolIntelligenceStats {
        ProtocolIntelligenceStats {
            domains_tracked: AtomicUsize::new(self.stats.domains_tracked.load(Ordering::Relaxed)),
            cache_hits: AtomicUsize::new(self.stats.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicUsize::new(self.stats.cache_misses.load(Ordering::Relaxed)),
            protocol_discoveries: AtomicUsize::new(self.stats.protocol_discoveries.load(Ordering::Relaxed)),
            failed_attempts_prevented: AtomicUsize::new(self.stats.failed_attempts_prevented.load(Ordering::Relaxed)),
        }
    }

    /// Clear cache (for testing or maintenance)
    pub fn clear(&self) {
        if let Ok(mut domains) = self.domains.write() {
            domains.clear();
            self.stats.domains_tracked.store(0, Ordering::Relaxed);
        }
    }
}

/// Get current timestamp in nanoseconds since `UNIX_EPOCH`
fn current_timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

impl Default for ProtocolIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProtocolIntelligenceStats {
    fn clone(&self) -> Self {
        Self {
            domains_tracked: AtomicUsize::new(self.domains_tracked.load(Ordering::Relaxed)),
            cache_hits: AtomicUsize::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicUsize::new(self.cache_misses.load(Ordering::Relaxed)),
            protocol_discoveries: AtomicUsize::new(self.protocol_discoveries.load(Ordering::Relaxed)),
            failed_attempts_prevented: AtomicUsize::new(self.failed_attempts_prevented.load(Ordering::Relaxed)),
        }
    }
}
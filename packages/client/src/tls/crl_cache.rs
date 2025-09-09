//! CRL cache implementation and validation logic

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use base64::engine::Engine;


use x509_parser::prelude::*;

// AsyncStream wrappers removed - using direct async methods per cryypt pattern
use super::errors::TlsError;
use super::types::{CrlCacheEntry, ParsedCertificate};

/// CRL validation status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrlStatus {
    Valid,
    Revoked,
    Unknown,
}

// MessageChunk trait implementation removed - direct async methods per cryypt pattern

#[derive(Clone)]
pub struct CrlCache {
    cache: Arc<RwLock<std::collections::HashMap<String, CrlCacheEntry>>>,
    http_client: crate::HttpClient,
    /// Cache hit statistics
    cache_hits: Arc<AtomicUsize>,
    /// Cache miss statistics
    cache_misses: Arc<AtomicUsize>,
}

impl std::fmt::Debug for CrlCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cache_size = match self.cache.read() {
            Ok(cache) => cache.len(),
            Err(_) => 0, // Graceful fallback for poisoned lock
        };
        f.debug_struct("CrlCache")
            .field("cache_size", &cache_size)
            .field("cache_hits", &self.cache_hits.load(std::sync::atomic::Ordering::Relaxed))
            .field("cache_misses", &self.cache_misses.load(std::sync::atomic::Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl CrlCache {
    pub fn new() -> Self {
        let http_client = crate::HttpClient::new();

        Self {
            cache: Arc::new(RwLock::new(std::collections::HashMap::with_capacity(64))),
            http_client,
            cache_hits: Arc::new(AtomicUsize::new(0)),
            cache_misses: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get cache statistics (hits, misses)
    pub fn get_stats(&self) -> (usize, usize) {
        (
            self.cache_hits.load(Ordering::Relaxed),
            self.cache_misses.load(Ordering::Relaxed),
        )
    }

    /// Get current cache size (number of entries)
    pub fn get_cache_size(&self) -> usize {
        match self.cache.read() {
            Ok(cache) => cache.len(),
            Err(poisoned) => {
                tracing::warn!("CRL cache read lock poisoned during size check, recovering");
                poisoned.into_inner().len()
            }
        }
    }

    // Streaming wrapper removed - using direct async methods per cryypt pattern

    /// Check certificate status against specific CRL URL - used by TLS verifier
    pub async fn check_certificate_status(
        &self,
        serial_number: &[u8],
        crl_url: &str,
    ) -> Result<CrlStatus, TlsError> {
        match self.check_against_crl_sync(serial_number, crl_url) {
            Ok(is_revoked) => {
                if is_revoked {
                    Ok(CrlStatus::Revoked)
                } else {
                    Ok(CrlStatus::Valid)
                }
            },
            Err(e) => {
                tracing::warn!("CRL validation error for {}: {}", crl_url, e);
                Ok(CrlStatus::Unknown)
            }
        }
    }

    /// Check if certificate serial number is revoked using CRL
    pub async fn check_certificate_revocation(
        &self,
        cert: &ParsedCertificate,
    ) -> Result<bool, TlsError> {
        if cert.crl_urls.is_empty() {
            tracing::warn!("No CRL URLs found in certificate, skipping CRL validation");
            return Ok(false); // Not revoked (no CRL available)
        }

        // Try each CRL URL until one succeeds
        for crl_url in &cert.crl_urls {
            match self.check_against_crl_sync(&cert.serial_number, crl_url) {
                Ok(is_revoked) => {
                    if is_revoked {
                        tracing::warn!(
                            "Certificate serial {:?} found in CRL from {}",
                            hex::encode(&cert.serial_number),
                            crl_url
                        );
                        return Ok(true);
                    }
                    tracing::info!(
                        "Certificate serial {:?} not found in CRL from {}",
                        hex::encode(&cert.serial_number),
                        crl_url
                    );
                }
                Err(e) => {
                    tracing::warn!("CRL validation failed for URL {}: {}", crl_url, e);
                    continue;
                }
            }
        }

        // If all CRLs were checked and certificate not found in any, it's not revoked
        Ok(false)
    }

    fn check_against_crl_sync(
        &self,
        serial_number: &[u8],
        crl_url: &str,
    ) -> Result<bool, TlsError> {
        let cache_key = crl_url.to_string();

        // Check cache first
        if let Some(cached_crl) = self.get_cached_crl(&cache_key) {
            if !Self::is_crl_cache_expired(&cached_crl) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("CRL cache hit for URL: {}", crl_url);
                return Ok(cached_crl.revoked_serials.contains(serial_number));
            }
        }

        // Cache miss - increment counter
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Download and parse CRL
        let crl_entry = self.download_and_parse_crl_sync(crl_url)?;

        // Cache the CRL
        self.cache_crl(cache_key, crl_entry.clone());

        // Check if certificate is revoked
        Ok(crl_entry.revoked_serials.contains(serial_number))
    }

    #[inline]
    fn get_cached_crl(&self, cache_key: &str) -> Option<CrlCacheEntry> {
        match self.cache.read() {
            Ok(cache) => cache.get(cache_key).cloned(),
            Err(poisoned) => {
                tracing::warn!("CRL cache read lock poisoned, recovering");
                poisoned.into_inner().get(cache_key).cloned()
            }
        }
    }

    fn is_crl_cache_expired(entry: &CrlCacheEntry) -> bool {
        let now = SystemTime::now();

        // Check if we have next_update time and it's passed
        if let Some(next_update) = entry.next_update {
            return now > next_update;
        }

        // Default cache expiry: 24 hours (CRLs are typically updated daily)
        let cache_duration = Duration::from_secs(24 * 3600);
        now.duration_since(entry.cached_at)
            .unwrap_or(Duration::ZERO)
            > cache_duration
    }

    #[inline]
    fn cache_crl(&self, cache_key: String, entry: CrlCacheEntry) {
        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(cache_key, entry);
            }
            Err(poisoned) => {
                tracing::warn!("CRL cache write lock poisoned, recovering");
                poisoned.into_inner().insert(cache_key, entry);
            }
        }
    }

    fn download_and_parse_crl_sync(&self, crl_url: &str) -> Result<CrlCacheEntry, TlsError> {
        use http::Method;
        use url::Url;
        
        // Parse URL
        let url = Url::parse(crl_url)
            .map_err(|e| TlsError::NetworkError(format!("Invalid CRL URL: {}", e)))?;
        
        // Download CRL using execute()
        let client = crate::client::HttpClient::default();
        let request = crate::http::request::HttpRequest::new(
            Method::GET,
            url,
            None, // No special headers needed for CRL download
            None, // No body for GET
            Some(std::time::Duration::from_secs(30)), // 30 second timeout
        );
        
        let response = client.execute(request);
        let body_stream = response.into_body_stream();
        
        let mut crl_bytes = Vec::new();
        const MAX_CRL_SIZE: usize = 50 * 1024 * 1024; // 50MB max CRL size
        
        // Collect body stream using Iterator
        for chunk in body_stream {
            // Check size limit before extending
            if crl_bytes.len() + chunk.data.len() > MAX_CRL_SIZE {
                return Err(TlsError::CrlValidation(
                    format!("CRL response too large (>{}MB)", MAX_CRL_SIZE / (1024 * 1024))
                ));
            }
            crl_bytes.extend_from_slice(&chunk.data);
            
            if chunk.is_final {
                break;
            }
        }
        
        if crl_bytes.is_empty() {
            return Err(TlsError::NetworkError("Empty CRL response".to_string()));
        }

        // Parse CRL
        self.parse_crl_data(&crl_bytes)
    }

    fn parse_crl_data(&self, crl_bytes: &[u8]) -> Result<CrlCacheEntry, TlsError> {
        // Parse PEM if it starts with "-----BEGIN"
        let der_bytes = if crl_bytes.starts_with(b"-----BEGIN") {
            let crl_pem = std::str::from_utf8(crl_bytes)
                .map_err(|_| TlsError::CrlValidation("Invalid UTF-8 in PEM CRL".to_string()))?;

            // Extract DER from PEM
            let mut der_data = Vec::new();
            let mut in_crl = false;
            for line in crl_pem.lines() {
                if line.contains("-----BEGIN") && line.contains("CRL") {
                    in_crl = true;
                    continue;
                }
                if line.contains("-----END") && line.contains("CRL") {
                    break;
                }
                if in_crl {
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(line) {
                        der_data.extend(decoded);
                    }
                }
            }

            if der_data.is_empty() {
                return Err(TlsError::CrlValidation(
                    "No CRL data found in PEM".to_string(),
                ));
            }

            der_data
        } else {
            // Assume DER format
            crl_bytes.to_vec()
        };

        // Parse X.509 CRL using x509-parser
        let (_, crl) = parse_x509_crl(&der_bytes)
            .map_err(|e| TlsError::CrlValidation(format!("CRL parsing failed: {}", e)))?;

        // Extract revoked certificate serial numbers
        let mut revoked_serials = HashSet::new();
        for revoked_cert in crl.iter_revoked_certificates() {
            revoked_serials.insert(revoked_cert.user_certificate.to_bytes_be());
        }

        // Extract next update time
        let next_update = crl.next_update().map(|time| {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(time.timestamp() as u64)
        });

        tracing::info!(
            "Parsed CRL with {} revoked certificates, next update: {:?}",
            revoked_serials.len(),
            next_update
        );

        Ok(CrlCacheEntry {
            revoked_serials,
            cached_at: SystemTime::now(),
            next_update,
        })
    }

    /// Cleanup expired CRL cache entries
    pub fn cleanup_cache(&self) {
        let mut cache = match self.cache.write() {
            Ok(cache) => cache,
            Err(poisoned) => {
                tracing::warn!("CRL cache write lock poisoned during cleanup, recovering");
                poisoned.into_inner()
            }
        };

        cache.retain(|_url, entry| !Self::is_crl_cache_expired(entry));

        tracing::debug!(
            "CRL cache cleanup completed, {} CRLs remaining",
            cache.len()
        );
    }
}

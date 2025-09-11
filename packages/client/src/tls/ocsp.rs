//! OCSP (Online Certificate Status Protocol) validation module

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use der::{Decode, Encode};
use rand::Rng;
// AsyncStream wrappers removed - using direct async methods per cryypt pattern

use ring::digest::{Context as DigestContext, SHA256};

use x509_cert::serial_number::SerialNumber;
// HttpChunk import removed - not used
use x509_ocsp::{CertId, OcspRequest, OcspResponse};

use super::types::ParsedCertificate;
use super::errors::TlsError;

/// OCSP response status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OcspStatus {
    Good,
    Revoked,
    Unknown,
}

/// OCSP response cache entry
#[derive(Debug, Clone)]
pub struct OcspCacheEntry {
    pub status: OcspStatus,
    pub cached_at: SystemTime,
    pub next_update: Option<SystemTime>,
}

/// OCSP response cache for performance optimization
#[derive(Clone)]
pub struct OcspCache {
    cache: Arc<RwLock<HashMap<String, OcspCacheEntry>>>,
    http_client: crate::HttpClient,
    /// Pre-generated random bytes for nonce generation
    nonce_pool: Arc<RwLock<Vec<u8>>>,
    /// Cache hit statistics
    cache_hits: Arc<AtomicUsize>,
    /// Cache miss statistics
    cache_misses: Arc<AtomicUsize>,
}

impl std::fmt::Debug for OcspCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cache_size = match self.cache.read() {
            Ok(cache) => cache.len(),
            Err(_) => 0, // Graceful fallback for poisoned lock
        };
        f.debug_struct("OcspCache")
            .field("cache_size", &cache_size)
            .field("cache_hits", &self.cache_hits.load(std::sync::atomic::Ordering::Relaxed))
            .field("cache_misses", &self.cache_misses.load(std::sync::atomic::Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl OcspCache {
    pub fn new() -> Self {
        let http_client = crate::HttpClient::new();

        // Pre-generate 1KB of random bytes for nonce generation
        let mut nonce_pool = vec![0u8; 1024];
        rand::rng().fill(&mut nonce_pool[..]);

        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(128))),
            http_client,
            nonce_pool: Arc::new(RwLock::new(nonce_pool)),
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
                tracing::warn!("OCSP cache read lock poisoned during size check, recovering");
                poisoned.into_inner().len()
            }
        }
    }

    /// Check OCSP status for a certificate with caching
    pub async fn check_certificate(
        &self,
        cert: &ParsedCertificate,
        issuer_cert: Option<&ParsedCertificate>,
    ) -> Result<OcspStatus, TlsError> {
        let cache_key = Self::make_cache_key(&cert.serial_number);

        // Check cache first
        if let Some(cached) = self.get_cached_status(&cache_key)
            && !Self::is_cache_expired(&cached) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                tracing::debug!(
                    "OCSP cache hit for certificate serial: {:?}",
                    hex::encode(&cert.serial_number)
                );
                return Ok(cached.status);
            }

        // Cache miss - increment counter
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Perform OCSP check
        match self.perform_ocsp_check(cert, issuer_cert) {
            Ok((status, next_update)) => {
                // Cache the result
                self.cache_status(cache_key, status, next_update);
                Ok(status)
            }
            Err(e) => {
                tracing::warn!("OCSP validation failed: {}", e);
                Ok(OcspStatus::Unknown)
            }
        }
    }

    fn make_cache_key(serial_number: &[u8]) -> String {
        hex::encode(serial_number)
    }

    #[inline]
    fn get_cached_status(&self, cache_key: &str) -> Option<OcspCacheEntry> {
        match self.cache.read() {
            Ok(cache) => cache.get(cache_key).cloned(),
            Err(poisoned) => {
                tracing::warn!("OCSP cache read lock poisoned, recovering");
                poisoned.into_inner().get(cache_key).cloned()
            }
        }
    }

    fn is_cache_expired(entry: &OcspCacheEntry) -> bool {
        let now = SystemTime::now();

        // Check if we have next_update time and it's passed
        if let Some(next_update) = entry.next_update {
            return now > next_update;
        }

        // Default cache expiry: 1 hour
        let cache_duration = Duration::from_secs(3600);
        now.duration_since(entry.cached_at)
            .unwrap_or(Duration::ZERO)
            > cache_duration
    }

    #[inline]
    fn cache_status(&self, cache_key: String, status: OcspStatus, next_update: Option<SystemTime>) {
        let entry = OcspCacheEntry {
            status,
            cached_at: SystemTime::now(),
            next_update,
        };

        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(cache_key, entry);
            }
            Err(poisoned) => {
                tracing::warn!("OCSP cache write lock poisoned, recovering");
                poisoned.into_inner().insert(cache_key, entry);
            }
        }
    }

    fn perform_ocsp_check(
        &self,
        cert: &ParsedCertificate,
        issuer_cert: Option<&ParsedCertificate>,
    ) -> Result<(OcspStatus, Option<SystemTime>), TlsError> {
        if cert.ocsp_urls.is_empty() {
            tracing::warn!("No OCSP URLs found in certificate, skipping OCSP validation");
            return Ok((OcspStatus::Unknown, None));
        }

        // We need issuer certificate for OCSP
        let issuer = issuer_cert.ok_or_else(|| {
            TlsError::OcspValidation("Issuer certificate required for OCSP validation".to_string())
        })?;

        // Try each OCSP URL until one succeeds
        for ocsp_url in &cert.ocsp_urls {
            match self.query_ocsp_responder(cert, issuer, ocsp_url) {
                Ok(result) => {
                    tracing::info!(
                        "OCSP validation successful for certificate serial: {:?}, status: {:?}",
                        hex::encode(&cert.serial_number),
                        result.0
                    );
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("OCSP query failed for URL {}: {}", ocsp_url, e);
                }
            }
        }

        // If all OCSP URLs failed, return Unknown (don't fail the validation)
        tracing::warn!(
            "All OCSP URLs failed for certificate serial: {:?}, treating as unknown",
            hex::encode(&cert.serial_number)
        );
        Ok((OcspStatus::Unknown, None))
    }

    fn query_ocsp_responder(
        &self,
        cert: &ParsedCertificate,
        issuer: &ParsedCertificate,
        ocsp_url: &str,
    ) -> Result<(OcspStatus, Option<SystemTime>), TlsError> {
        use http::{Method, HeaderMap, HeaderName, HeaderValue};
        use url::Url;
        use bytes::Bytes;
        
        // Create OCSP request
        let (ocsp_request, nonce) = self.create_ocsp_request(cert, issuer)?;

        // Parse OCSP URL
        let url = Url::parse(ocsp_url)
            .map_err(|e| TlsError::OcspValidation(format!("Invalid OCSP URL: {e}")))?;

        // Prepare headers for OCSP request
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/ocsp-request"),
        );
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("application/ocsp-response"),
        );

        // Create HTTP POST request with OCSP request body
        let request = crate::http::request::HttpRequest::new(
            Method::POST,
            url,
            Some(headers),
            Some(crate::http::request::RequestBody::Bytes(Bytes::from(ocsp_request))),
            Some(std::time::Duration::from_secs(10)), // 10 second timeout for OCSP
        );

        // Execute the request
        let response = self.http_client.execute(request);
        let body_stream = response.into_body_stream();
        
        // Collect response body using Iterator
        let mut response_bytes = Vec::new();
        for chunk in body_stream {
            response_bytes.extend_from_slice(&chunk.data);
            if chunk.is_final {
                break;
            }
        }

        if response_bytes.is_empty() {
            return Err(TlsError::OcspValidation("Empty OCSP response".to_string()));
        }

        // Parse OCSP response
        self.parse_ocsp_response(&response_bytes, &nonce, &cert.serial_number)
    }

    fn create_ocsp_request(
        &self,
        cert: &ParsedCertificate,
        issuer_cert: &ParsedCertificate,
    ) -> Result<(Vec<u8>, Vec<u8>), TlsError> {
        // Generate 16-byte nonce for replay protection
        let nonce = self.generate_nonce();

        // Create CertID using SHA-256
        let mut hasher = DigestContext::new(&SHA256);
        hasher.update(&issuer_cert.subject_der);
        let issuer_name_hash = hasher.finish();

        let mut hasher = DigestContext::new(&SHA256);
        hasher.update(&issuer_cert.public_key_der);
        let issuer_key_hash = hasher.finish();

        // Convert serial number
        let serial = SerialNumber::new(&cert.serial_number)
            .map_err(|e| TlsError::OcspValidation(format!("Invalid serial number: {e}")))?;

        use x509_cert::spki::AlgorithmIdentifierOwned;

        let cert_id = CertId {
            hash_algorithm: AlgorithmIdentifierOwned {
                oid: der::asn1::ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1"), // SHA-256
                parameters: None,
            },
            issuer_name_hash: der::asn1::OctetString::new(issuer_name_hash.as_ref()).map_err(
                |e| TlsError::OcspValidation(format!("Failed to create issuer name hash: {e}")),
            )?,
            issuer_key_hash: der::asn1::OctetString::new(issuer_key_hash.as_ref()).map_err(
                |e| TlsError::OcspValidation(format!("Failed to create issuer key hash: {e}")),
            )?,
            serial_number: serial,
        };

        use x509_ocsp::{Request, TbsRequest};

        let tbs_request = TbsRequest {
            version: x509_ocsp::Version::V1,
            requestor_name: None,
            request_list: vec![Request {
                req_cert: cert_id,
                single_request_extensions: None,
            }],
            request_extensions: None,
        };

        let request = OcspRequest {
            tbs_request,
            optional_signature: None,
        };

        let der_bytes = request.to_der().map_err(|e| {
            TlsError::OcspValidation(format!("Failed to encode OCSP request: {e}"))
        })?;

        Ok((der_bytes, nonce))
    }

    fn parse_ocsp_response(
        &self,
        response_bytes: &[u8],
        expected_nonce: &[u8],
        cert_serial: &[u8],
    ) -> Result<(OcspStatus, Option<SystemTime>), TlsError> {
        let response = OcspResponse::from_der(response_bytes).map_err(|e| {
            TlsError::OcspValidation(format!("Failed to decode OCSP response: {e}"))
        })?;

        // Check response status
        if response.response_status != x509_ocsp::OcspResponseStatus::Successful {
            return Err(TlsError::OcspValidation(format!(
                "OCSP response status: {:?}",
                response.response_status
            )));
        }

        let response_bytes = response.response_bytes.as_ref().ok_or_else(|| {
            TlsError::OcspValidation("No response bytes in OCSP response".to_string())
        })?;

        let basic_response =
            x509_ocsp::BasicOcspResponse::from_der(response_bytes.response.as_bytes()).map_err(
                |e| TlsError::OcspValidation(format!("Failed to parse basic OCSP response: {e}")),
            )?;

        // Verify nonce matches
        if let Some(nonce_ext) = basic_response
            .tbs_response_data
            .response_extensions
            .as_ref()
            .and_then(|exts| {
                exts.iter().find(|ext| {
                    ext.extn_id == der::asn1::ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.48.1.2")
                })
            })
            && nonce_ext.extn_value.as_bytes() != expected_nonce {
                return Err(TlsError::OcspValidation(
                    "OCSP nonce mismatch - possible replay attack".to_string(),
                ));
            }

        // Find response for our certificate
        let single_response = basic_response
            .tbs_response_data
            .responses
            .iter()
            .find(|resp| resp.cert_id.serial_number.as_bytes() == cert_serial)
            .ok_or_else(|| {
                TlsError::OcspValidation("Certificate not found in OCSP response".to_string())
            })?;

        let status = match &single_response.cert_status {
            x509_ocsp::CertStatus::Good(_) => OcspStatus::Good,
            x509_ocsp::CertStatus::Revoked(_) => OcspStatus::Revoked,
            x509_ocsp::CertStatus::Unknown(_) => OcspStatus::Unknown,
        };

        // Extract next update time
        let next_update = single_response.next_update.as_ref().map(|time| {
            let unix_time = time.0.to_unix_duration().as_secs();
            SystemTime::UNIX_EPOCH + Duration::from_secs(unix_time)
        });

        Ok((status, next_update))
    }

    #[inline]
    fn generate_nonce(&self) -> Vec<u8> {
        let mut nonce = vec![0u8; 16];

        // Get random bytes from pre-generated pool
        {
            let mut pool = match self.nonce_pool.write() {
                Ok(pool) => pool,
                Err(poisoned) => {
                    tracing::warn!("OCSP nonce pool write lock poisoned, recovering");
                    poisoned.into_inner()
                }
            };
            if pool.len() >= 16 {
                nonce.copy_from_slice(&pool[..16]);
                pool.drain(..16);
            } else {
                // Refill pool if exhausted
                pool.resize(1024, 0);
                rand::rng().fill(&mut pool[..]);
                nonce.copy_from_slice(&pool[..16]);
                pool.drain(..16);
            }
        }

        nonce
    }

    /// Cleanup expired cache entries
    pub fn cleanup_cache(&self) {
        let mut cache = match self.cache.write() {
            Ok(cache) => cache,
            Err(poisoned) => {
                tracing::warn!("OCSP cache write lock poisoned during cleanup, recovering");
                poisoned.into_inner()
            }
        };

        cache.retain(|_key, entry| !Self::is_cache_expired(entry));

        tracing::debug!(
            "OCSP cache cleanup completed, {} entries remaining",
            cache.len()
        );
    }
}

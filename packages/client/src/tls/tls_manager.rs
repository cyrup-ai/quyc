//! Enterprise TLS Manager
//!
//! Provides comprehensive TLS connection management with OCSP validation,
//! CRL checking, certificate validation, and enterprise security features.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rustls::{ClientConfig, RootCertStore};
// ServerName import removed - not used
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::ocsp::OcspCache;
use super::crl_cache::CrlCache;
use super::certificate::parser::parse_certificate_from_der;
use super::builder::CertificateAuthority;
use super::errors::TlsError;
// ParsedCertificate alias import removed - not used
use crate::config::HttpConfig;

/// Detailed TLS cache statistics for monitoring and troubleshooting
#[derive(Debug, Clone)]
pub struct TlsCacheStats {
    /// OCSP cache hits
    pub ocsp_hits: usize,
    /// OCSP cache misses
    pub ocsp_misses: usize,
    /// Number of entries in OCSP cache
    pub ocsp_cache_size: usize,
    /// CRL cache hits
    pub crl_hits: usize,
    /// CRL cache misses
    pub crl_misses: usize,
    /// Number of entries in CRL cache
    pub crl_cache_size: usize,
}

impl TlsCacheStats {
    /// Calculate total cache hits
    pub fn total_hits(&self) -> usize {
        self.ocsp_hits + self.crl_hits
    }
    
    /// Calculate total cache misses
    pub fn total_misses(&self) -> usize {
        self.ocsp_misses + self.crl_misses
    }
    
    /// Calculate total cache requests
    pub fn total_requests(&self) -> usize {
        self.total_hits() + self.total_misses()
    }
    
    /// Calculate overall cache hit rate as a percentage (0.0 to 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            (self.total_hits() as f64 / total as f64) * 100.0
        }
    }
    
    /// Calculate OCSP cache hit rate as a percentage (0.0 to 100.0)
    pub fn ocsp_hit_rate(&self) -> f64 {
        let total = self.ocsp_hits + self.ocsp_misses;
        if total == 0 {
            0.0
        } else {
            (self.ocsp_hits as f64 / total as f64) * 100.0
        }
    }
    
    /// Calculate CRL cache hit rate as a percentage (0.0 to 100.0)
    pub fn crl_hit_rate(&self) -> f64 {
        let total = self.crl_hits + self.crl_misses;
        if total == 0 {
            0.0
        } else {
            (self.crl_hits as f64 / total as f64) * 100.0
        }
    }
}



/// Enterprise TLS connection manager with comprehensive security validation
#[derive(Clone)]
pub struct TlsManager {
    /// OCSP validation cache for certificate status checking
    ocsp_cache: Arc<OcspCache>,
    /// CRL cache for certificate revocation checking
    crl_cache: Arc<CrlCache>,
    /// Custom certificate authorities for validation
    custom_cas: Arc<RwLock<HashMap<String, CertificateAuthority>>>,
    /// TLS configuration
    config: TlsConfig,
}

/// TLS configuration for enterprise features
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Enable OCSP validation
    pub enable_ocsp: bool,
    /// Enable CRL checking
    pub enable_crl: bool,
    /// Use system certificate store
    pub use_system_certs: bool,
    /// Custom root certificates
    pub custom_root_certs: Vec<String>,
    /// TLS 1.3 early data support
    pub enable_early_data: bool,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Certificate validation timeout
    pub validation_timeout: Duration,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enable_ocsp: true,
            enable_crl: true,
            use_system_certs: true,
            custom_root_certs: Vec::new(),
            enable_early_data: false,
            connect_timeout: Duration::from_secs(10),
            validation_timeout: Duration::from_secs(5),
        }
    }
}

impl TlsConfig {
    /// Create TLS configuration from HttpConfig
    pub fn from_http_config(http_config: &HttpConfig) -> Self {
        Self {
            enable_ocsp: true, // Always enable for enterprise
            enable_crl: true,  // Always enable for enterprise
            use_system_certs: http_config.use_native_certs,
            custom_root_certs: Vec::new(),
            enable_early_data: http_config.tls_early_data,
            connect_timeout: Duration::from_secs(10),
            validation_timeout: Duration::from_secs(5),
        }
    }
    
    /// Create AI-optimized TLS configuration
    pub fn ai_optimized() -> Self {
        Self {
            enable_ocsp: true,
            enable_crl: true,
            use_system_certs: true,
            custom_root_certs: Vec::new(),
            enable_early_data: true, // Enable for AI performance
            connect_timeout: Duration::from_secs(5), // Faster for AI workloads
            validation_timeout: Duration::from_secs(3),
        }
    }
}

impl TlsManager {
    /// Create new TLS manager with default configuration
    pub fn new() -> Self {
        Self::with_config(TlsConfig::default())
    }
    
    /// Create TLS manager with specific configuration
    pub fn with_config(config: TlsConfig) -> Self {
        Self {
            ocsp_cache: Arc::new(OcspCache::new()),
            crl_cache: Arc::new(CrlCache::new()),
            custom_cas: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create TLS manager from HttpConfig
    pub fn from_http_config(http_config: &HttpConfig) -> Self {
        Self::with_config(TlsConfig::from_http_config(http_config))
    }
    
    /// Create new TLS manager with certificate directory (async)
    pub async fn with_cert_dir(cert_dir: std::path::PathBuf) -> Result<Self, TlsError> {
        // Create certificate directory if it doesn't exist
        if !cert_dir.exists() {
            std::fs::create_dir_all(&cert_dir)
                .map_err(|e| TlsError::Internal(format!("Failed to create cert directory: {}", e)))?;
        }
        
        // Initialize TLS manager with custom config
        let mut config = TlsConfig::default();
        
        // Add any certificates found in the directory
        if let Ok(entries) = std::fs::read_dir(&cert_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("pem") {
                    if let Ok(cert_data) = std::fs::read_to_string(&path) {
                        config.custom_root_certs.push(cert_data);
                    }
                }
            }
        }
        
        Ok(Self::with_config(config))
    }
    
    /// Add custom certificate authority
    pub fn add_certificate_authority(&self, name: String, ca: CertificateAuthority) -> Result<(), TlsError> {
        let mut cas = self.custom_cas.write()
            .map_err(|_| TlsError::Internal("Failed to acquire CA lock".to_string()))?;
        
        // Validate CA before adding
        if !ca.is_valid() {
            return Err(TlsError::CertificateExpired(format!("Certificate authority '{}' is expired", name)));
        }
        
        cas.insert(name, ca);
        Ok(())
    }
    
    /// Create enterprise TLS connection with full validation
    pub async fn create_connection(
        &self,
        host: &str,
        port: u16,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>, TlsError> {
        tracing::debug!("Creating enterprise TLS connection to {}:{}", host, port);
        
        // Create TCP connection with timeout
        let tcp_stream = tokio::time::timeout(
            self.config.connect_timeout,
            TcpStream::connect((host, port))
        ).await
            .map_err(|_| TlsError::Internal("Connection timeout".to_string()))?
            .map_err(|e| TlsError::Internal(format!("Failed to connect to {}:{}: {}", host, port, e)))?;

        // Create enterprise TLS client configuration
        let client_config = self.create_client_config_sync()?;
        
        // Create TLS connector
        let connector = TlsConnector::from(Arc::new(client_config));
        
        // Create server name for TLS
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| TlsError::Internal(format!("Invalid hostname '{}': {}", host, e)))?;

        // Perform TLS handshake
        let tls_stream = connector.connect(server_name, tcp_stream).await
            .map_err(|e| TlsError::Internal(format!("TLS handshake failed: {}", e)))?;

        tracing::info!("Enterprise TLS connection established to {}:{}", host, port);
        Ok(tls_stream)
    }
    
    /// Create enterprise client configuration with full certificate validation
    fn create_client_config_sync(&self) -> Result<ClientConfig, TlsError> {
        // Create root certificate store
        let mut root_store = RootCertStore::empty();
        
        // Add system certificates if enabled
        if self.config.use_system_certs {
            let cert_result = rustls_native_certs::load_native_certs();
            for cert in cert_result.certs {
                if let Err(e) = root_store.add(cert) {
                    tracing::warn!("Failed to add system certificate: {}", e);
                }
            }
            
            if !cert_result.errors.is_empty() {
                for err in &cert_result.errors {
                    tracing::warn!("Certificate load error: {}", err);
                }
                // Fall back to webpki roots if there were significant errors
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            }
            
            tracing::debug!("Loaded {} system certificates", root_store.len());
        } else {
            // Use webpki roots as fallback
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }
        
        // Add custom root certificates
        for cert_pem in &self.config.custom_root_certs {
            // Parse PEM certificate data
            if let Ok(cert_der) = pem::parse(cert_pem) {
                let cert = rustls::pki_types::CertificateDer::from(cert_der.contents());
                if let Err(e) = root_store.add(cert) {
                    tracing::warn!("Failed to add custom root certificate: {}", e);
                } else {
                    tracing::debug!("Added custom root certificate from PEM data");
                }
            } else {
                tracing::warn!("Failed to parse custom root certificate PEM data");
            }
        }
        
        // Add custom certificate authorities
        let cas = self.custom_cas.read()
            .map_err(|_| TlsError::Internal("Failed to acquire CA lock".to_string()))?;
        
        for (name, ca) in cas.iter() {
            if ca.is_valid() {
                // Parse CA certificate and add to root store
                if let Ok(cert_der) = pem::parse(&ca.certificate_pem) {
                    let cert = rustls::pki_types::CertificateDer::from(cert_der.contents());
                    if let Err(e) = root_store.add(cert) {
                        tracing::warn!("Failed to add custom CA '{}': {}", name, e);
                    } else {
                        tracing::debug!("Added custom CA: {}", name);
                    }
                }
            } else {
                tracing::warn!("Skipping expired CA: {}", name);
            }
        }
        
        // Create verifier that includes OCSP and CRL validation
        let verifier = Arc::new(EnterpriseServerCertVerifier::new(
            self.ocsp_cache.clone(),
            self.crl_cache.clone(),
            self.config.enable_ocsp,
            self.config.enable_crl,
            self.config.validation_timeout,
        ));
        
        // Build configuration with enterprise verifier
        let mut client_config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();
        
        // Configure early data if enabled
        if self.config.enable_early_data {
            client_config.enable_early_data = true;
        }
        
        Ok(client_config)
    }
    
    /// Get aggregated cache statistics from OCSP and CRL caches
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let (ocsp_hits, ocsp_misses) = self.ocsp_cache.get_stats();
        let (crl_hits, crl_misses) = self.crl_cache.get_stats();
        (ocsp_hits + crl_hits, ocsp_misses + crl_misses)
    }
    
    /// Get detailed cache statistics for monitoring and troubleshooting
    pub fn get_detailed_cache_stats(&self) -> TlsCacheStats {
        let (ocsp_hits, ocsp_misses) = self.ocsp_cache.get_stats();
        let (crl_hits, crl_misses) = self.crl_cache.get_stats();
        
        let ocsp_cache_size = self.ocsp_cache.get_cache_size();
        let crl_cache_size = self.crl_cache.get_cache_size();
        
        TlsCacheStats {
            ocsp_hits,
            ocsp_misses,
            ocsp_cache_size,
            crl_hits,
            crl_misses,
            crl_cache_size,
        }
    }
    
    /// Get OCSP cache statistics only
    pub fn get_ocsp_stats(&self) -> (usize, usize) {
        self.ocsp_cache.get_stats()
    }
    
    /// Get CRL cache statistics only
    pub fn get_crl_stats(&self) -> (usize, usize) {
        self.crl_cache.get_stats()
    }
    
    /// Perform maintenance operations (cleanup caches, etc.)
    pub fn perform_maintenance(&self) {
        self.ocsp_cache.cleanup_cache();
        self.crl_cache.cleanup_cache();
        tracing::debug!("TLS manager maintenance completed");
    }
    
    /// Validate certificate using OCSP (Online Certificate Status Protocol)
    pub async fn validate_certificate_ocsp(
        &self,
        cert_pem: &str,
        issuer_cert_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        let parsed_cert = crate::tls::certificate::parse_certificate_from_pem(cert_pem)?;

        // Parse issuer certificate if provided
        let issuer_cert = if let Some(issuer_pem) = issuer_cert_pem {
            Some(crate::tls::certificate::parse_certificate_from_pem(issuer_pem)?)
        } else {
            None
        };

        match self
            .ocsp_cache
            .check_certificate(&parsed_cert, issuer_cert.as_ref())
            .await
        {
            Ok(crate::tls::ocsp::OcspStatus::Good) => {
                tracing::info!("OCSP validation successful: certificate is valid");
                Ok(())
            },
            Ok(crate::tls::ocsp::OcspStatus::Revoked) => {
                Err(TlsError::CertificateRevoked("Certificate revoked via OCSP".to_string()))
            },
            Ok(crate::tls::ocsp::OcspStatus::Unknown) => {
                tracing::warn!("OCSP validation inconclusive");
                Ok(()) // Allow unknown status but log warning
            },
            Err(e) => {
                tracing::warn!("OCSP validation failed: {}", e);
                Err(TlsError::OcspValidationFailed(format!("OCSP validation error: {}", e)))
            }
        }
    }
    
    /// Validate certificate using CRL (Certificate Revocation List)
    pub async fn validate_certificate_crl(&self, cert_pem: &str) -> Result<(), TlsError> {
        let parsed_cert = crate::tls::certificate::parse_certificate_from_pem(cert_pem)?;

        if parsed_cert.crl_urls.is_empty() {
            tracing::debug!("No CRL URLs found in certificate, skipping CRL validation");
            return Ok(());
        }

        // Check certificate against each CRL URL
        for crl_url in &parsed_cert.crl_urls {
            match self
                .crl_cache
                .check_certificate_status(&parsed_cert.serial_number, crl_url)
                .await
            {
                Ok(crate::tls::crl_cache::CrlStatus::Valid) => {
                    tracing::debug!("CRL validation passed for URL: {}", crl_url);
                },
                Ok(crate::tls::crl_cache::CrlStatus::Revoked) => {
                    return Err(TlsError::CertificateRevoked(format!("Certificate revoked via CRL: {}", crl_url)));
                },
                Ok(crate::tls::crl_cache::CrlStatus::Unknown) => {
                    tracing::warn!("CRL validation inconclusive for URL: {}", crl_url);
                    // Continue checking other CRL URLs
                },
                Err(e) => {
                    tracing::warn!("CRL validation failed for {}: {}", crl_url, e);
                    // Continue checking other CRL URLs
                }
            }
        }

        tracing::info!("CRL validation completed successfully");
        Ok(())
    }
}

/// Enterprise server certificate verifier with OCSP and CRL validation
#[derive(Debug)]
struct EnterpriseServerCertVerifier {
    ocsp_cache: Arc<OcspCache>,
    crl_cache: Arc<CrlCache>,
    enable_ocsp: bool,
    enable_crl: bool,
    validation_timeout: Duration,
}

impl EnterpriseServerCertVerifier {
    fn new(
        ocsp_cache: Arc<OcspCache>,
        crl_cache: Arc<CrlCache>,
        enable_ocsp: bool,
        enable_crl: bool,
        validation_timeout: Duration,
    ) -> Self {
        Self {
            ocsp_cache,
            crl_cache,
            enable_ocsp,
            enable_crl,
            validation_timeout,
        }
    }
}

impl rustls::client::danger::ServerCertVerifier for EnterpriseServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        intermediates: &[rustls::pki_types::CertificateDer<'_>],
        server_name: &rustls::pki_types::ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // First perform standard certificate validation
        let webpki_verifier = rustls::client::WebPkiServerVerifier::builder(
            Arc::new(webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect())
        ).build().map_err(|e| rustls::Error::General(format!("Failed to create webpki verifier: {}", e)))?;
        
        // Perform standard validation
        webpki_verifier.verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)?;
        
        // Parse end entity certificate for additional validation
        let parsed_cert = parse_certificate_from_der(end_entity.as_ref())
            .map_err(|e| rustls::Error::General(format!("Failed to parse certificate: {}", e)))?;
        
        // Perform OCSP validation if enabled (synchronous for rustls compatibility)
        if self.enable_ocsp && !parsed_cert.ocsp_urls.is_empty() {
            let issuer_cert = if !intermediates.is_empty() {
                Some(parse_certificate_from_der(intermediates[0].as_ref())
                    .map_err(|e| rustls::Error::General(format!("Failed to parse issuer certificate: {}", e)))?)
            } else {
                None
            };
            
            // Perform real OCSP validation using blocking calls to our async infrastructure
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(
                    self.ocsp_cache.check_certificate(&parsed_cert, issuer_cert.as_ref())
                )
            }) {
                Ok(crate::tls::ocsp::OcspStatus::Good) => {
                    tracing::debug!("OCSP validation passed for {:?}", server_name);
                },
                Ok(crate::tls::ocsp::OcspStatus::Revoked) => {
                    tracing::error!("Certificate revoked via OCSP for {:?}", server_name);
                    return Err(rustls::Error::General("Certificate revoked via OCSP".to_string()));
                },
                Ok(crate::tls::ocsp::OcspStatus::Unknown) => {
                    tracing::warn!("OCSP validation inconclusive for {:?}", server_name);
                    // Allow unknown status but log warning
                },
                Err(e) => {
                    tracing::warn!("OCSP validation failed for {:?}: {}", server_name, e);
                    // Allow validation errors but log them
                }
            }
        }
        
        // Perform CRL validation if enabled (synchronous for rustls compatibility)
        if self.enable_crl && !parsed_cert.crl_urls.is_empty() {
            // Check certificate against each CRL URL
            for crl_url in &parsed_cert.crl_urls {
                match tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(
                        self.crl_cache.check_certificate_status(&parsed_cert.serial_number, crl_url)
                    )
                }) {
                    Ok(crate::tls::crl_cache::CrlStatus::Valid) => {
                        tracing::debug!("CRL validation passed for {:?} against {}", server_name, crl_url);
                    },
                    Ok(crate::tls::crl_cache::CrlStatus::Revoked) => {
                        // SECURITY: Don't expose internal CRL URLs in logs or error messages
                        tracing::error!("Certificate revoked via CRL for {:?}", server_name);
                        return Err(rustls::Error::General("Certificate revoked via CRL check".to_string()));
                    },
                    Ok(crate::tls::crl_cache::CrlStatus::Unknown) => {
                        tracing::warn!("CRL validation inconclusive for {:?} against {}", server_name, crl_url);
                        // Allow unknown status but log warning
                    },
                    Err(e) => {
                        tracing::warn!("CRL validation failed for {:?} against {}: {}", server_name, crl_url, e);
                        // Allow validation errors but log them
                    }
                }
            }
        }
        
        tracing::info!("Enterprise certificate validation completed for {:?}", server_name);
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }
    
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
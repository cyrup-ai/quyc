//! Certificate Authority domain object and builders

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::tls::errors::TlsError;
use super::responses::{CertificateAuthorityResponse, CaOperation};

/// Format a distinguished name `HashMap` into a string representation
fn format_dn_hashmap(dn: &HashMap<String, String>) -> String {
    let mut parts = Vec::new();
    
    // Order DN components in standard order: CN, O, OU, L, ST, C
    let ordered_keys = ["CN", "O", "OU", "L", "ST", "C"];
    
    for &key in &ordered_keys {
        if let Some(value) = dn.get(key) {
            parts.push(format!("{key}={value}"));
        }
    }
    
    // Add any remaining keys that weren't in the standard order
    for (key, value) in dn {
        if !ordered_keys.contains(&key.as_str()) {
            parts.push(format!("{key}={value}"));
        }
    }
    
    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        parts.join(", ")
    }
}

/// Format serial number bytes as hexadecimal string
fn format_serial_number(serial: &[u8]) -> String {
    if serial.is_empty() {
        "00".to_string()
    } else {
        hex::encode(serial)
    }
}

/// Certificate Authority domain object with serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthority {
    pub name: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub metadata: CaMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaMetadata {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub valid_from: SystemTime,
    pub valid_until: SystemTime,
    pub key_algorithm: String,
    pub key_size: Option<u32>,
    pub created_at: SystemTime,
    pub source: CaSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaSource {
    Filesystem { path: PathBuf },
    Keychain,
    Remote { url: String },
    Generated,
}

impl CertificateAuthority {
    /// Check if the certificate authority is currently valid
    #[must_use] 
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now();
        now >= self.metadata.valid_from && now <= self.metadata.valid_until
    }

    /// Get duration until expiry
    pub fn expires_in(&self) -> Result<Duration, TlsError> {
        let now = SystemTime::now();
        self.metadata.valid_until.duration_since(now).map_err(|_| {
            TlsError::CertificateExpired("Certificate authority has expired".to_string())
        })
    }

    /// Check if this CA can sign certificates for the given domain
    pub fn can_sign_for_domain(&self, domain: &str) -> bool {
        use crate::tls::certificate::parsing::{parse_certificate_from_pem, verify_hostname};
        
        if !self.is_valid() {
            return false;
        }
        
        // Parse CA certificate to check constraints
        let ca_cert = match parse_certificate_from_pem(&self.certificate_pem) {
            Ok(cert) => cert,
            Err(e) => {
                tracing::error!("Failed to parse CA certificate for domain validation: {}", e);
                return false;
            }
        };
        
        // Check if this is a proper CA
        if !ca_cert.is_ca {
            tracing::warn!("Certificate is not marked as CA, cannot sign for domain: {}", domain);
            return false;
        }
        
        // Delegate to existing hostname verification logic
        // If the CA certificate itself can validate this domain, then it can sign for it
        if let Ok(()) = verify_hostname(&ca_cert, domain) {
            tracing::debug!("CA can sign for domain '{}' - matches CA constraints", domain);
            true
        } else {
            tracing::warn!("CA certificate cannot sign for domain '{}' - no matching constraints", domain);
            false
        }
    }
}

/// Builder for certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityBuilder {
    name: String,
}

impl AuthorityBuilder {
    #[must_use] 
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Work with filesystem-based certificate authority
    pub fn path<P: AsRef<Path>>(self, path: P) -> AuthorityFilesystemBuilder {
        AuthorityFilesystemBuilder {
            name: self.name,
            path: path.as_ref().to_path_buf(),
            common_name: None,
            valid_for_years: 10,
            key_size: 2048,
        }
    }

    /// Work with keychain-based certificate authority (macOS/Windows)
    #[must_use] 
    pub fn keychain(self) -> AuthorityKeychainBuilder {
        AuthorityKeychainBuilder { name: self.name }
    }

    /// Work with remote certificate authority
    #[must_use] 
    pub fn url(self, url: &str) -> AuthorityRemoteBuilder {
        AuthorityRemoteBuilder {
            name: self.name,
            url: url.to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// Builder for filesystem certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityFilesystemBuilder {
    name: String,
    path: PathBuf,
    common_name: Option<String>,
    valid_for_years: u32,
    key_size: u32,
}

impl AuthorityFilesystemBuilder {
    /// Set common name for certificate authority creation
    #[must_use] 
    pub fn common_name(self, cn: &str) -> Self {
        Self {
            common_name: Some(cn.to_string()),
            ..self
        }
    }

    /// Set validity period in years for certificate authority creation
    #[must_use] 
    pub fn valid_for_years(self, years: u32) -> Self {
        Self {
            valid_for_years: years,
            ..self
        }
    }

    /// Set key size for certificate authority creation
    #[must_use] 
    pub fn key_size(self, bits: u32) -> Self {
        Self {
            key_size: bits,
            ..self
        }
    }

    /// Create a new certificate authority
    pub async fn create(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;

        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&self.path) {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to create directory: {}", e)],
                files_created: vec![],
            };
        }

        // Generate CA certificate
        let mut params = match CertificateParams::new(vec![]) {
            Ok(params) => params,
            Err(e) => {
                return CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: CaOperation::CreateFailed,
                    issues: vec![format!("Failed to create certificate parameters: {}", e)],
                    files_created: vec![],
                };
            }
        };
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        let mut distinguished_name = DistinguishedName::new();
        let common_name = self.common_name.unwrap_or_else(|| self.name.clone());
        distinguished_name.push(DnType::CommonName, &common_name);
        params.distinguished_name = distinguished_name;

        // Set validity period
        let now = SystemTime::now();
        params.not_before = now.into();
        params.not_after = (now
            + std::time::Duration::from_secs(365 * 24 * 3600 * u64::from(self.valid_for_years)))
        .into();

        // Generate key pair
        let key_pair = KeyPair::generate()
            .map_err(|e| format!("Failed to generate key pair: {e}"));

        let key_pair = match key_pair {
            Ok(kp) => kp,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![e],
                    files_created: vec![],
                };
            }
        };

        let cert = match params.self_signed(&key_pair) {
            Ok(c) => c,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![format!("Failed to generate certificate: {}", e)],
                    files_created: vec![],
                };
            }
        };

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // cert_pem and key_pem are now direct String results
        let (cert_pem, key_pem) = (cert_pem, key_pem);

        // Save files
        let cert_path = self.path.join("ca.crt");
        let key_path = self.path.join("ca.key");
        let mut files_created = vec![];

        if let Err(e) = std::fs::write(&cert_path, &cert_pem) {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to write certificate: {}", e)],
                files_created,
            };
        }
        files_created.push(cert_path);

        if let Err(e) = std::fs::write(&key_path, &key_pem) {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to write private key: {}", e)],
                files_created,
            };
        }
        files_created.push(key_path);

        // Create authority object
        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: key_pem,
            metadata: CaMetadata {
                subject: common_name.clone(),
                issuer: common_name,
                serial_number: "1".to_string(), // CA serial number
                valid_from: now,
                valid_until: now
                    + std::time::Duration::from_secs(365 * 24 * 3600 * u64::from(self.valid_for_years)),
                key_algorithm: "RSA".to_string(),
                key_size: Some(self.key_size),
                created_at: now,
                source: CaSource::Generated,
            },
        };

        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Created,
            issues: vec![],
            files_created,
        }
    }

    /// Load existing certificate authority from filesystem
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;

        use crate::tls::certificate::parse_certificate_from_pem;

        let cert_path = self.path.join("ca.crt");
        let key_path = self.path.join("ca.key");

        // Check if both files exist
        if !cert_path.exists() || !key_path.exists() {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("CA files not found at {:?}", self.path)],
                files_created: vec![],
            };
        }

        // Read certificate and key files
        let cert_pem = match std::fs::read_to_string(&cert_path) {
            Ok(content) => content,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to read certificate: {}", e)],
                    files_created: vec![],
                };
            }
        };

        let key_pem = match std::fs::read_to_string(&key_path) {
            Ok(content) => content,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to read private key: {}", e)],
                    files_created: vec![],
                };
            }
        };

        // Parse certificate to extract metadata
        let parsed_cert = match parse_certificate_from_pem(&cert_pem) {
            Ok(cert) => cert,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to parse certificate: {}", e)],
                    files_created: vec![],
                };
            }
        };

        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: key_pem,
            metadata: CaMetadata {
                subject: format_dn_hashmap(&parsed_cert.subject),
                issuer: format_dn_hashmap(&parsed_cert.issuer),
                serial_number: format_serial_number(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                key_algorithm: parsed_cert.key_algorithm.clone(),
                key_size: parsed_cert.key_size,
                created_at: SystemTime::now(),
                source: CaSource::Filesystem {
                    path: self.path.clone(),
                },
            },
        };

        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Loaded,
            issues: vec![],
            files_created: vec![],
        }
    }
}

/// Builder for keychain certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityKeychainBuilder {
    name: String,
}

impl AuthorityKeychainBuilder {
    /// Load certificate authority from system keychain
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;
        use crate::tls::certificate::parse_certificate_from_pem;
        
        tracing::debug!("Loading CA '{}' from system keychain", self.name);
        
        // Use fluent-ai service pattern for keychain access
        let service_name = "fluent-ai-http3";
        let cert_key_id = format!("ca-cert-{}", self.name);
        let private_key_id = format!("ca-key-{}", self.name);
        
        // Create keychain entry for certificate
        let cert_entry = match keyring::Entry::new(service_name, &cert_key_id) {
            Ok(entry) => entry,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to create keychain entry for certificate: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Create keychain entry for private key
        let key_entry = match keyring::Entry::new(service_name, &private_key_id) {
            Ok(entry) => entry,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to create keychain entry for private key: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Retrieve certificate from keychain
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = cert_entry.get_password();
            let _ = tx.send(result);
        });
        
        let cert_pem = match rx.recv() {
            Ok(Ok(pem)) => pem,
            Ok(Err(keyring::Error::NoEntry)) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Certificate for CA '{}' not found in keychain", self.name)],
                    files_created: vec![],
                };
            }
            Ok(Err(e)) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to retrieve certificate from keychain: {}", e)],
                    files_created: vec![],
                };
            }
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Keychain operation failed: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Retrieve private key from keychain
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = key_entry.get_password();
            let _ = tx.send(result);
        });
        
        let key_pem = match rx.recv() {
            Ok(Ok(pem)) => pem,
            Ok(Err(keyring::Error::NoEntry)) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Private key for CA '{}' not found in keychain", self.name)],
                    files_created: vec![],
                };
            }
            Ok(Err(e)) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to retrieve private key from keychain: {}", e)],
                    files_created: vec![],
                };
            }
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Keychain operation failed: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Parse certificate to extract metadata
        let parsed_cert = match parse_certificate_from_pem(&cert_pem) {
            Ok(cert) => cert,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to parse certificate from keychain: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Validate that this is actually a CA certificate
        if !parsed_cert.is_ca {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Certificate for '{}' is not a Certificate Authority", self.name)],
                files_created: vec![],
            };
        }
        
        // Check certificate validity
        let now = SystemTime::now();
        if now < parsed_cert.not_before || now > parsed_cert.not_after {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Certificate Authority '{}' has expired or is not yet valid", self.name)],
                files_created: vec![],
            };
        }
        
        // Create the CA object
        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: key_pem,
            metadata: CaMetadata {
                subject: format_dn_hashmap(&parsed_cert.subject),
                issuer: format_dn_hashmap(&parsed_cert.issuer),
                serial_number: format_serial_number(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                key_algorithm: parsed_cert.key_algorithm.clone(),
                key_size: parsed_cert.key_size,
                created_at: SystemTime::now(),
                source: CaSource::Keychain,
            },
        };
        
        tracing::info!("Successfully loaded CA '{}' from keychain (valid until: {:?})", 
                      self.name, parsed_cert.not_after);
        
        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Loaded,
            issues: vec![],
            files_created: vec![],
        }
    }
    
    /// Store certificate authority in system keychain
    pub async fn store(&self, authority: &CertificateAuthority) -> super::responses::CertificateAuthorityResponse {
        tracing::debug!("Storing CA '{}' to system keychain", authority.name);
        
        // Use fluent-ai service pattern for keychain access
        let service_name = "fluent-ai-http3";
        let cert_key_id = format!("ca-cert-{}", authority.name);
        let private_key_id = format!("ca-key-{}", authority.name);
        
        // Store certificate in keychain
        let cert_entry = match keyring::Entry::new(service_name, &cert_key_id) {
            Ok(entry) => entry,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::StoreFailed,
                    issues: vec![format!("Failed to create keychain entry for certificate: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        let cert_pem = authority.certificate_pem.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = cert_entry.set_password(&cert_pem);
            let _ = tx.send(result);
        });
        
        if let Err(e) = rx.recv().unwrap_or_else(|_e| Err(keyring::Error::NoEntry)) {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::StoreFailed,
                issues: vec![format!("Failed to store certificate in keychain: {}", e)],
                files_created: vec![],
            };
        }
        
        // Store private key in keychain
        let key_entry = match keyring::Entry::new(service_name, &private_key_id) {
            Ok(entry) => entry,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::StoreFailed,
                    issues: vec![format!("Failed to create keychain entry for private key: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        let key_pem = authority.private_key_pem.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = key_entry.set_password(&key_pem);
            let _ = tx.send(result);
        });
        
        if let Err(e) = rx.recv().unwrap_or_else(|_e| Err(keyring::Error::NoEntry)) {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::StoreFailed,
                issues: vec![format!("Failed to store private key in keychain: {}", e)],
                files_created: vec![],
            };
        }
        
        tracing::info!("Successfully stored CA '{}' in keychain", authority.name);
        
        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority.clone()),
            operation: super::responses::CaOperation::Stored,
            issues: vec![],
            files_created: vec![],
        }
    }
}

/// Builder for remote certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityRemoteBuilder {
    name: String,
    url: String,
    timeout: Duration,
}

impl AuthorityRemoteBuilder {
    /// Set timeout for remote operations
    #[must_use] 
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    /// Load certificate authority from remote URL
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;
        use crate::tls::certificate::parse_certificate_from_pem;
        use http::Method;
        use url::Url;
        
        tracing::debug!("Loading CA '{}' from remote URL: {}", self.name, self.url);
        
        // Parse URL
        let url = match Url::parse(&self.url) {
            Ok(u) => u,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Invalid URL: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Download certificate from remote URL using execute()
        let client = crate::client::HttpClient::default();
        let request = crate::http::request::HttpRequest::new(
            Method::GET,
            url,
            None, // No special headers needed for certificate download
            None, // No body for GET
            Some(self.timeout),
        );
        
        let response = client.execute(request);
        
        let cert_content = match tokio::time::timeout(
            self.timeout,
            async {
                let mut body_bytes = Vec::new();
                let body_stream = response.into_body_stream();
                
                // Collect body stream into bytes
                let mut body_stream_pin = std::pin::Pin::new(&body_stream);
                while let Some(chunk) = ystream::AsyncStream::next(&mut body_stream_pin).await {
                    body_bytes.extend_from_slice(&chunk.data);
                    if chunk.is_final {
                        break;
                    }
                }
                
                if body_bytes.is_empty() {
                    return Err(crate::error::network_error("No body data received"));
                }
                
                // Convert to string for PEM parsing
                let body_string = String::from_utf8(body_bytes)
                    .map_err(|e| crate::error::network_error(format!("Invalid UTF-8 response: {e}")))?;
                
                Ok(body_string)
            }
        ).await {
            Ok(Ok(cert_content)) => cert_content,
            Ok(Err(e)) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to fetch from remote URL: {}", e)],
                    files_created: vec![],
                };
            }
            Err(_) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Request timeout after {:?}", self.timeout)],
                    files_created: vec![],
                };
            }
        };
        
        // Validate that the content looks like a PEM certificate
        if !cert_content.contains("-----BEGIN CERTIFICATE-----") || !cert_content.contains("-----END CERTIFICATE-----") {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec!["Remote content does not appear to be a PEM certificate".to_string()],
                files_created: vec![],
            };
        }
        
        // Parse certificate to extract metadata
        let parsed_cert = match parse_certificate_from_pem(&cert_content) {
            Ok(cert) => cert,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to parse certificate from remote URL: {}", e)],
                    files_created: vec![],
                };
            }
        };
        
        // Validate that this is actually a CA certificate
        if !parsed_cert.is_ca {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Certificate from '{}' is not a Certificate Authority", self.url)],
                files_created: vec![],
            };
        }
        
        // Check certificate validity
        let now = SystemTime::now();
        if now < parsed_cert.not_before || now > parsed_cert.not_after {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Certificate Authority from '{}' has expired or is not yet valid", self.url)],
                files_created: vec![],
            };
        }
        
        // Create the CA object (note: no private key for remote CAs)
        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_content,
            private_key_pem: String::new(), // Remote CAs don't provide private keys
            metadata: CaMetadata {
                subject: format_dn_hashmap(&parsed_cert.subject),
                issuer: format_dn_hashmap(&parsed_cert.issuer),
                serial_number: format_serial_number(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                key_algorithm: parsed_cert.key_algorithm.clone(),
                key_size: parsed_cert.key_size,
                created_at: SystemTime::now(),
                source: CaSource::Remote { url: self.url.clone() },
            },
        };
        
        tracing::info!("Successfully loaded CA '{}' from remote URL (valid until: {:?})", 
                      self.name, parsed_cert.not_after);
        
        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Loaded,
            issues: vec![],
            files_created: vec![],
        }
    }
}

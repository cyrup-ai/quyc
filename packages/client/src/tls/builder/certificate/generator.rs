//! Certificate generator components

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::super::authority::CertificateAuthority;
use super::super::responses::{CertificateGenerationResponse, CertificateInfo, GenerationIssue, IssueSeverity, GeneratedFile, FileType};

/// Certificate generator builder
#[derive(Debug, Clone)]
pub struct CertificateGenerator {
    // Internal state for generation configuration
}

impl CertificateGenerator {
    #[must_use] 
    pub fn new() -> Self {
        Self {}
    }

    /// Generate certificate for single domain
    #[must_use] 
    pub fn domain(self, domain: &str) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: vec![domain.to_string()],
            is_wildcard: false,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }

    /// Generate certificate for multiple domains
    pub fn domains(self, domains: &[&str]) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: domains.iter().map(std::string::ToString::to_string).collect(),
            is_wildcard: false,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }

    /// Generate wildcard certificate for domain
    #[must_use] 
    pub fn wildcard(self, domain: &str) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: vec![format!("*.{}", domain)],
            is_wildcard: true,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }
}

impl Default for CertificateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Certificate generator with domain configured
#[derive(Debug, Clone)]
pub struct CertificateGeneratorWithDomain {
    domains: Vec<String>,
    is_wildcard: bool,
    authority: Option<CertificateAuthority>,
    self_signed: bool,
    valid_for_days: u32,
    save_path: Option<PathBuf>,
}

impl CertificateGeneratorWithDomain {
    /// Sign certificate with certificate authority
    #[must_use] 
    pub fn authority(self, ca: &CertificateAuthority) -> Self {
        Self {
            authority: Some(ca.clone()),
            self_signed: false,
            ..self
        }
    }

    /// Generate self-signed certificate
    #[must_use] 
    pub fn self_signed(self) -> Self {
        Self {
            self_signed: true,
            authority: None,
            ..self
        }
    }

    /// Set validity period in days
    #[must_use] 
    pub fn valid_for_days(self, days: u32) -> Self {
        Self {
            valid_for_days: days,
            ..self
        }
    }

    /// Save generated certificate to path
    #[must_use = "Certificate generator builder methods return a new generator and should be used"]
    pub fn save_to<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            save_path: Some(path.as_ref().to_path_buf()),
            ..self
        }
    }

    /// Execute certificate generation
    pub async fn generate(self) -> CertificateGenerationResponse {
        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};

        let mut params = match CertificateParams::new(self.domains.clone()) {
            Ok(params) => params,
            Err(e) => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: None,
                    private_key_pem: None,
                    issues: vec![GenerationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Failed to create certificate parameters: {e}"),
                        suggestion: Some("Check certificate parameters and domain names".to_string()),
                    }],
                };
            }
        };

        // Set up distinguished name
        let mut distinguished_name = DistinguishedName::new();
        if let Some(first_domain) = self.domains.first() {
            distinguished_name.push(DnType::CommonName, first_domain);
        }
        params.distinguished_name = distinguished_name;

        // Set validity period
        let now = SystemTime::now();
        params.not_before = now.into();
        params.not_after =
            (now + std::time::Duration::from_secs(u64::from(self.valid_for_days) * 24 * 3600)).into();

        // Add SAN entries with proper error handling
        let mut san_entries = Vec::new();
        for domain in &self.domains {
            let san_entry = if domain.starts_with("*.") {
                match domain.clone().try_into() {
                    Ok(name) => SanType::DnsName(name),
                    Err(e) => {
                        return CertificateGenerationResponse {
                            success: false,
                            certificate_info: None,
                            files_created: vec![],
                            certificate_pem: None,
                            private_key_pem: None,
                            issues: vec![GenerationIssue {
                                severity: IssueSeverity::Error,
                                message: format!("Invalid wildcard domain '{domain}': {e}"),
                                suggestion: Some("Check domain format".to_string()),
                            }],
                        };
                    }
                }
            } else {
                match domain.clone().try_into() {
                    Ok(name) => SanType::DnsName(name),
                    Err(e) => {
                        return CertificateGenerationResponse {
                            success: false,
                            certificate_info: None,
                            files_created: vec![],
                            certificate_pem: None,
                            private_key_pem: None,
                            issues: vec![GenerationIssue {
                                severity: IssueSeverity::Error,
                                message: format!("Invalid domain '{domain}': {e}"),
                                suggestion: Some("Check domain format".to_string()),
                            }],
                        };
                    }
                }
            };
            san_entries.push(san_entry);
        }
        params.subject_alt_names = san_entries;

        // Generate key pair
        let key_pair = match KeyPair::generate() {
            Ok(kp) => kp,
            Err(e) => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: None,
                    private_key_pem: None,
                    issues: vec![GenerationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Failed to generate key pair: {e}"),
                        suggestion: Some("Check system entropy and crypto libraries".to_string()),
                    }],
                };
            }
        };

        // Create certificate
        let cert = if self.self_signed {
            // Self-signed certificate
            match params.self_signed(&key_pair) {
                Ok(c) => c,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![GenerationIssue {
                            severity: IssueSeverity::Error,
                            message: format!("Failed to generate self-signed certificate: {e}"),
                            suggestion: Some("Check certificate parameters".to_string()),
                        }],
                    };
                }
            }
        } else if let Some(ca) = &self.authority {
            // CA-signed certificate generation using rcgen Issuer pattern
            tracing::debug!("Creating CA-signed certificate with domains: {:?}", self.domains);
            
            // Parse CA private key to create KeyPair
            let ca_key_pair = match rcgen::KeyPair::from_pem(&ca.private_key_pem) {
                Ok(kp) => kp,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![GenerationIssue {
                            severity: IssueSeverity::Error,
                            message: format!("Failed to parse CA private key: {e}"),
                            suggestion: Some("Check CA private key format and validity".to_string()),
                        }],
                    };
                }
            };
            
            // Create CA issuer from certificate PEM and key pair
            let ca_issuer = match rcgen::Issuer::from_ca_cert_pem(&ca.certificate_pem, ca_key_pair) {
                Ok(issuer) => issuer,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![GenerationIssue {
                            severity: IssueSeverity::Error,
                            message: format!("Failed to create CA issuer: {e}"),
                            suggestion: Some("Check CA certificate format and key compatibility".to_string()),
                        }],
                    };
                }
            };
            
            // Generate certificate signed by CA
            match params.signed_by(&key_pair, &ca_issuer) {
                Ok(signed_cert) => {
                    tracing::info!("Successfully generated CA-signed certificate for domains: {:?}", self.domains);
                    signed_cert
                },
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![GenerationIssue {
                            severity: IssueSeverity::Error,
                            message: format!("Failed to sign certificate with CA: {e}"),
                            suggestion: Some("Check CA certificate authority constraints and validity".to_string()),
                        }],
                    };
                }
            }
        } else {
            return CertificateGenerationResponse {
                success: false,
                certificate_info: None,
                files_created: vec![],
                certificate_pem: None,
                private_key_pem: None,
                issues: vec![GenerationIssue {
                    severity: IssueSeverity::Error,
                    message: "No signing method specified".to_string(),
                    suggestion: Some("Use .self_signed() or .authority(ca)".to_string()),
                }],
            };
        };

        // Serialize certificate and key (rcgen 0.14.3 returns String directly)
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        let mut files_created = vec![];

        // Save files if path specified
        if let Some(save_path) = &self.save_path {
            // Create directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(save_path) {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![GenerationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Failed to create directory: {e}"),
                        suggestion: Some("Check directory permissions".to_string()),
                    }],
                };
            }

            let cert_file = save_path.join("cert.pem");
            let key_file = save_path.join("key.pem");

            // Write certificate file
            if let Err(e) = std::fs::write(&cert_file, &cert_pem) {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![GenerationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Failed to write certificate file: {e}"),
                        suggestion: Some("Check file permissions".to_string()),
                    }],
                };
            }
            files_created.push(GeneratedFile {
                path: cert_file,
                file_type: FileType::Certificate,
                size_bytes: cert_pem.len() as u64,
            });

            // Write key file
            if let Err(e) = std::fs::write(&key_file, &key_pem) {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![GenerationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Failed to write private key file: {e}"),
                        suggestion: Some("Check file permissions".to_string()),
                    }],
                };
            }
            files_created.push(GeneratedFile {
                path: key_file,
                file_type: FileType::PrivateKey,
                size_bytes: key_pem.len() as u64,
            });
        }

        CertificateGenerationResponse {
            success: true,
            certificate_info: Some(CertificateInfo {
                subject: self
                    .domains
                    .first()
                    .unwrap_or(&"Unknown".to_string())
                    .clone(),
                issuer: if self.self_signed {
                    self.domains
                        .first()
                        .unwrap_or(&"Unknown".to_string())
                        .clone()
                } else if let Some(ca) = &self.authority {
                    ca.metadata.subject.clone()
                } else {
                    "Unknown CA".to_string()
                },
                serial_number: "1".to_string(),
                valid_from: now,
                valid_until: now
                    + std::time::Duration::from_secs(u64::from(self.valid_for_days) * 24 * 3600),
                domains: self.domains.clone(),
                is_ca: false,
                key_algorithm: "RSA".to_string(),
                key_size: Some(2048),
            }),
            files_created,
            certificate_pem: Some(cert_pem),
            private_key_pem: Some(key_pem),
            issues: vec![],
        }
    }
}
//! Certificate validator components

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use std::collections::HashMap;

use super::super::authority::CertificateAuthority;
use super::super::responses::{CertificateValidationResponse, CertificateInfo, ValidationSummary, CheckResult, ValidationIssue, IssueSeverity, IssueCategory, ValidationPerformance};
use super::utils::format_dn_hashmap;
use crate::tls::certificate::{parse_certificate_from_pem, validate_basic_constraints, validate_certificate_time, validate_key_usage};
use crate::tls::types::ParsedCertificate;
use crate::tls::types::CertificateUsage;

/// Certificate validator builder
#[derive(Debug, Clone)]
pub struct CertificateValidator {
    // Internal state for validation configuration
}

impl CertificateValidator {
    #[must_use] 
    pub fn new() -> Self {
        Self {}
    }

    /// Load certificate from file
    pub fn from_file<P: AsRef<Path>>(self, path: P) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::File(path.as_ref().to_path_buf()),
            domain: None,
            domains: None,
            authority: None,
        }
    }

    /// Load certificate from PEM string
    #[must_use] 
    pub fn from_string(self, pem: &str) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::String(pem.to_string()),
            domain: None,
            domains: None,
            authority: None,
        }
    }

    /// Load certificate from bytes
    #[must_use] 
    pub fn from_bytes(self, bytes: &[u8]) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::Bytes(bytes.to_vec()),
            domain: None,
            domains: None,
            authority: None,
        }
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Certificate validator with input source configured
#[derive(Debug, Clone)]
pub struct CertificateValidatorWithInput {
    input_source: InputSource,
    domain: Option<String>,
    domains: Option<Vec<String>>,
    authority: Option<CertificateAuthority>,
}

impl CertificateValidatorWithInput {
    /// Validate certificate for specific domain
    #[must_use] 
    pub fn domain(self, domain: &str) -> Self {
        Self {
            domain: Some(domain.to_string()),
            ..self
        }
    }

    /// Validate certificate for multiple domains
    #[must_use = "Certificate validator builder methods return a new validator and should be used"]
    pub fn domains(self, domains: &[&str]) -> Self {
        Self {
            domains: Some(domains.iter().map(std::string::ToString::to_string).collect()),
            ..self
        }
    }

    /// Validate certificate against specific authority
    #[must_use] 
    pub fn authority(self, ca: &CertificateAuthority) -> Self {
        Self {
            authority: Some(ca.clone()),
            ..self
        }
    }

    /// Execute validation with all security checks enabled by default
    pub async fn validate(self) -> CertificateValidationResponse {
        let start_time = Instant::now();
        let mut validation_breakdown = HashMap::new();
        let mut issues = vec![];

        // Get certificate content based on input source
        let cert_content = match self.load_certificate_content(start_time, &validation_breakdown) {
            Ok(content) => content,
            Err(response) => return response,
        };

        // Parse certificate
        let parsed_cert = match self.parse_certificate_content(&cert_content, &mut validation_breakdown, start_time) {
            Ok(cert) => cert,
            Err(response) => return response,
        };

        // Perform basic validations
        let time_check = self.validate_certificate_time(&parsed_cert, &mut issues, &mut validation_breakdown);
        self.validate_basic_constraints(&parsed_cert, &mut issues, &mut validation_breakdown);
        self.validate_key_usage(&parsed_cert, &mut issues, &mut validation_breakdown);

        // Create TlsManager for advanced validations
        let tls_manager = match self.create_tls_manager(&mut issues).await {
            Some(manager) => manager,
            None => {
                // Return early response with basic validation only
                return self.create_basic_validation_response(
                    &parsed_cert, 
                    time_check, 
                    issues, 
                    validation_breakdown, 
                    start_time
                );
            }
        };

        // Perform advanced validations
        let (ocsp_check, crl_check) = self.perform_revocation_checks(&cert_content, &tls_manager, &mut issues, &mut validation_breakdown).await;
        let ca_check = self.validate_certificate_chain(&cert_content, &mut issues, &mut validation_breakdown).await;
        let domain_check = self.validate_domain_match(&parsed_cert, &mut issues);

        // Create final response
        self.create_complete_validation_response(
            &parsed_cert,
            time_check,
            ocsp_check,
            crl_check,
            ca_check,
            domain_check,
            issues,
            validation_breakdown,
            start_time,
            &tls_manager
        )
    }

    /// Load certificate content from the configured input source
    fn load_certificate_content(&self, start_time: Instant, validation_breakdown: &HashMap<String, std::time::Duration>) -> Result<String, CertificateValidationResponse> {
        match &self.input_source {
            InputSource::File(path) => {
                match std::fs::read_to_string(path) {
                    Ok(content) => Ok(content),
                    Err(e) => Err(self.create_error_response(
                        start_time,
                        validation_breakdown.clone(),
                        CheckResult::Failed(format!("Failed to read file: {e}")),
                        format!("Failed to read certificate file: {e}"),
                        "Check file path and permissions",
                    )),
                }
            },
            InputSource::String(content) => Ok(content.clone()),
            InputSource::Bytes(bytes) => match String::from_utf8(bytes.clone()) {
                Ok(content) => Ok(content),
                Err(e) => Err(self.create_error_response(
                    start_time,
                    validation_breakdown.clone(),
                    CheckResult::Failed(format!("Invalid UTF-8: {e}")),
                    format!("Certificate bytes are not valid UTF-8: {e}"),
                    "Ensure certificate is in PEM format",
                )),
            },
        }
    }

    /// Parse certificate content into `ParsedCertificate`
    fn parse_certificate_content(&self, cert_content: &str, validation_breakdown: &mut HashMap<String, std::time::Duration>, start_time: Instant) -> Result<ParsedCertificate, CertificateValidationResponse> {
        let parse_start = Instant::now();
        match parse_certificate_from_pem(cert_content) {
            Ok(cert) => {
                validation_breakdown.insert("parsing".to_string(), parse_start.elapsed());
                Ok(cert)
            }
            Err(e) => {
                validation_breakdown.insert("parsing".to_string(), parse_start.elapsed());
                Err(self.create_error_response(
                    start_time,
                    validation_breakdown.clone(),
                    CheckResult::Failed(format!("Parse error: {e}")),
                    format!("Failed to parse certificate: {e}"),
                    "Ensure certificate is in valid PEM format",
                ))
            }
        }
    }

    /// Validate certificate time validity
    fn validate_certificate_time(&self, parsed_cert: &ParsedCertificate, issues: &mut Vec<ValidationIssue>, validation_breakdown: &mut HashMap<String, std::time::Duration>) -> CheckResult {
        let time_start = Instant::now();
        let time_result = validate_certificate_time(parsed_cert);
        validation_breakdown.insert("time_validity".to_string(), time_start.elapsed());

        match &time_result {
            Ok(()) => CheckResult::Passed,
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Expiry,
                    message: format!("Time validation failed: {e}"),
                    suggestion: Some("Check certificate validity period".to_string()),
                });
                CheckResult::Failed(format!("Time validation: {e}"))
            }
        }
    }

    /// Validate basic constraints
    fn validate_basic_constraints(&self, parsed_cert: &ParsedCertificate, issues: &mut Vec<ValidationIssue>, validation_breakdown: &mut HashMap<String, std::time::Duration>) {
        let constraints_start = Instant::now();
        let constraints_result = validate_basic_constraints(parsed_cert, false);
        validation_breakdown.insert("basic_constraints".to_string(), constraints_start.elapsed());

        if let Err(e) = constraints_result {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::KeyUsage,
                message: format!("Basic constraints issue: {e}"),
                suggestion: Some("Check certificate basic constraints extension".to_string()),
            });
        }
    }

    /// Validate key usage
    fn validate_key_usage(&self, parsed_cert: &ParsedCertificate, issues: &mut Vec<ValidationIssue>, validation_breakdown: &mut HashMap<String, std::time::Duration>) {
        let key_usage_start = Instant::now();
        let key_usage_result = validate_key_usage(parsed_cert, CertificateUsage::ServerAuth);
        validation_breakdown.insert("key_usage".to_string(), key_usage_start.elapsed());

        if let Err(e) = key_usage_result {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::KeyUsage,
                message: format!("Key usage issue: {e}"),
                suggestion: Some("Check certificate key usage extension".to_string()),
            });
        }
    }

    /// Create TLS manager for advanced validation
    async fn create_tls_manager(&self, issues: &mut Vec<ValidationIssue>) -> Option<crate::tls::tls_manager::TlsManager> {
        let temp_dir = std::env::temp_dir().join("tls_validation");
        match crate::tls::tls_manager::TlsManager::with_cert_dir(temp_dir).await {
            Ok(manager) => Some(manager),
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Chain,
                    message: format!("Could not initialize TLS manager for security checks: {e}"),
                    suggestion: Some("OCSP and CRL validation will be skipped".to_string()),
                });
                None
            }
        }
    }

    /// Perform OCSP and CRL revocation checks
    async fn perform_revocation_checks(&self, cert_content: &str, tls_manager: &crate::tls::tls_manager::TlsManager, issues: &mut Vec<ValidationIssue>, validation_breakdown: &mut HashMap<String, std::time::Duration>) -> (CheckResult, CheckResult) {
        // OCSP validation
        let ocsp_start = Instant::now();
        let ocsp_result = tls_manager.validate_certificate_ocsp(cert_content, None).await;
        validation_breakdown.insert("ocsp_validation".to_string(), ocsp_start.elapsed());

        let ocsp_check = match &ocsp_result {
            Ok(()) => CheckResult::Passed,
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Revocation,
                    message: format!("OCSP validation failed: {e}"),
                    suggestion: Some("Certificate may be revoked or OCSP responder unavailable".to_string()),
                });
                CheckResult::Failed(format!("OCSP: {e}"))
            }
        };

        // CRL validation
        let crl_start = Instant::now();
        let crl_result = tls_manager.validate_certificate_crl(cert_content).await;
        validation_breakdown.insert("crl_validation".to_string(), crl_start.elapsed());

        let crl_check = match &crl_result {
            Ok(()) => CheckResult::Passed,
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Revocation,
                    message: format!("CRL validation failed: {e}"),
                    suggestion: Some("Certificate may be revoked or CRL unavailable".to_string()),
                });
                CheckResult::Failed(format!("CRL: {e}"))
            }
        };

        (ocsp_check, crl_check)
    }

    /// Create basic validation response (when TLS manager unavailable)
    fn create_basic_validation_response(&self, parsed_cert: &ParsedCertificate, time_check: CheckResult, mut issues: Vec<ValidationIssue>, validation_breakdown: HashMap<String, std::time::Duration>, start_time: Instant) -> CertificateValidationResponse {
        let domain_check = self.validate_domain_match(parsed_cert, &mut issues);
        let is_valid = matches!(time_check, CheckResult::Passed)
            && domain_check.as_ref().is_none_or(|c| matches!(c, CheckResult::Passed));

        CertificateValidationResponse {
            is_valid,
            certificate_info: self.create_certificate_info(parsed_cert),
            validation_summary: ValidationSummary {
                parsing: CheckResult::Passed,
                time_validity: time_check,
                domain_match: domain_check,
                ca_validation: None,
                ocsp_status: Some(CheckResult::Skipped),
                crl_status: Some(CheckResult::Skipped),
            },
            issues,
            performance: ValidationPerformance {
                total_duration: start_time.elapsed(),
                parallel_tasks_executed: 0,
                cache_hits: 0,
                cache_misses: 0,
                network_requests: 0,
                validation_breakdown,
            },
        }
    }

    /// Create complete validation response
    fn create_complete_validation_response(&self, parsed_cert: &ParsedCertificate, time_check: CheckResult, ocsp_check: CheckResult, crl_check: CheckResult, ca_check: Option<CheckResult>, domain_check: Option<CheckResult>, issues: Vec<ValidationIssue>, validation_breakdown: HashMap<String, std::time::Duration>, start_time: Instant, tls_manager: &crate::tls::tls_manager::TlsManager) -> CertificateValidationResponse {
        let is_valid = matches!(time_check, CheckResult::Passed)
            && matches!(ocsp_check, CheckResult::Passed)
            && matches!(crl_check, CheckResult::Passed)
            && domain_check.as_ref().is_none_or(|c| matches!(c, CheckResult::Passed))
            && ca_check.as_ref().is_none_or(|c| matches!(c, CheckResult::Passed));

        CertificateValidationResponse {
            is_valid,
            certificate_info: self.create_certificate_info(parsed_cert),
            validation_summary: ValidationSummary {
                parsing: CheckResult::Passed,
                time_validity: time_check,
                domain_match: domain_check,
                ca_validation: ca_check,
                ocsp_status: Some(ocsp_check),
                crl_status: Some(crl_check),
            },
            issues,
            performance: ValidationPerformance {
                total_duration: start_time.elapsed(),
                parallel_tasks_executed: 3,
                cache_hits: {
                    let (hits, _) = tls_manager.get_cache_stats();
                    hits
                },
                cache_misses: {
                    let (_, misses) = tls_manager.get_cache_stats();
                    misses
                },
                network_requests: 2,
                validation_breakdown,
            },
        }
    }

    /// Create certificate info from parsed certificate
    fn create_certificate_info(&self, parsed_cert: &ParsedCertificate) -> CertificateInfo {
        CertificateInfo {
            subject: format_dn_hashmap(&parsed_cert.subject),
            issuer: format_dn_hashmap(&parsed_cert.issuer),
            serial_number: hex::encode(&parsed_cert.serial_number),
            valid_from: parsed_cert.not_before,
            valid_until: parsed_cert.not_after,
            domains: parsed_cert.san_dns_names.clone(),
            is_ca: parsed_cert.is_ca,
            key_algorithm: parsed_cert.key_algorithm.clone(),
            key_size: parsed_cert.key_size,
        }
    }

    /// Create error response for validation failures
    fn create_error_response(&self, start_time: Instant, validation_breakdown: HashMap<String, std::time::Duration>, parsing_result: CheckResult, error_message: String, suggestion: &str) -> CertificateValidationResponse {
        CertificateValidationResponse {
            is_valid: false,
            certificate_info: CertificateInfo {
                subject: "Failed to process".to_string(),
                issuer: "Failed to process".to_string(),
                serial_number: "Failed to process".to_string(),
                valid_from: SystemTime::now(),
                valid_until: SystemTime::now(),
                domains: vec![],
                is_ca: false,
                key_algorithm: "Unknown".to_string(),
                key_size: None,
            },
            validation_summary: ValidationSummary {
                parsing: parsing_result,
                time_validity: CheckResult::Skipped,
                domain_match: None,
                ca_validation: None,
                ocsp_status: None,
                crl_status: None,
            },
            issues: vec![ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::Parsing,
                message: error_message,
                suggestion: Some(suggestion.to_string()),
            }],
            performance: ValidationPerformance {
                total_duration: start_time.elapsed(),
                parallel_tasks_executed: 0,
                cache_hits: 0,
                cache_misses: 0,
                network_requests: 0,
                validation_breakdown,
            },
        }
    }

    /// Validate domain matching
    fn validate_domain_match(&self, parsed_cert: &ParsedCertificate, issues: &mut Vec<ValidationIssue>) -> Option<CheckResult> {
        if let Some(domain) = &self.domain {
            if parsed_cert.san_dns_names.contains(domain)
                || (parsed_cert.subject.get("CN") == Some(domain))
            {
                Some(CheckResult::Passed)
            } else {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Domain,
                    message: format!("Certificate not valid for domain: {domain}"),
                    suggestion: Some("Check SAN entries and subject CN".to_string()),
                });
                Some(CheckResult::Failed("Domain mismatch".to_string()))
            }
        } else {
            None
        }
    }

    /// Validate certificate chain if authority provided
    async fn validate_certificate_chain(&self, cert_content: &str, issues: &mut Vec<ValidationIssue>, validation_breakdown: &mut HashMap<String, std::time::Duration>) -> Option<CheckResult> {
        if let Some(authority) = &self.authority {
            let chain_start = Instant::now();
            let chain_result = crate::tls::certificate::validate_certificate_chain(
                cert_content,
                &rustls::pki_types::CertificateDer::from(
                    authority.certificate_pem.as_bytes().to_vec(),
                ),
            )
            .await;
            validation_breakdown.insert("chain_validation".to_string(), chain_start.elapsed());

            match chain_result {
                Ok(()) => Some(CheckResult::Passed),
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: IssueCategory::Chain,
                        message: format!("Certificate chain validation failed: {e}"),
                        suggestion: Some(
                            "Certificate may not be signed by the provided CA".to_string(),
                        ),
                    });
                    Some(CheckResult::Failed(format!("Chain: {e}")))
                }
            }
        } else {
            None
        }
    }
}

/// Input source for certificate validation
#[derive(Debug, Clone)]
enum InputSource {
    File(PathBuf),
    String(String),
    Bytes(Vec<u8>),
}
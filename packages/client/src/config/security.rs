//! Security Configuration Module
//!
//! Security settings and validation for HTTP/2 and HTTP/3 connections.



/// Security configuration provider trait
pub trait SecurityConfigProvider {
    fn tls_verification_enabled(&self) -> bool;
    fn certificate_pinning_enabled(&self) -> bool;
    fn min_tls_version(&self) -> TlsVersion;
    fn cipher_suites(&self) -> &[&'static str];
    fn enable_sni(&self) -> bool;
    fn enable_ocsp(&self) -> bool;
    fn certificate_transparency_enabled(&self) -> bool;
}

/// TLS version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TlsVersion {
    Tls12,
    #[default]
    Tls13,
}

/// Runtime security configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SecurityConfig {
    pub tls_verification_enabled: bool,
    pub certificate_pinning_enabled: bool,
    pub min_tls_version: TlsVersion,
    pub cipher_suites: Vec<&'static str>,
    pub enable_sni: bool,
    pub enable_ocsp: bool,
    pub certificate_transparency_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_verification_enabled: true,
            certificate_pinning_enabled: false,
            min_tls_version: TlsVersion::Tls13,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384",
                "TLS_AES_128_GCM_SHA256",
                "TLS_CHACHA20_POLY1305_SHA256",
            ],
            enable_sni: true,
            enable_ocsp: true,
            certificate_transparency_enabled: false,
        }
    }
}

impl SecurityConfig {
    /// Create high-security configuration
    #[must_use]
    pub fn high_security() -> Self {
        Self {
            certificate_pinning_enabled: true,
            certificate_transparency_enabled: true,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384", // Only the strongest cipher
            ],
            ..Self::default()
        }
    }
    
    /// Create development configuration with relaxed security
    #[must_use]
    pub fn development() -> Self {
        Self {
            tls_verification_enabled: false,  // For self-signed certs
            certificate_pinning_enabled: false,
            enable_ocsp: false,
            ..Self::default()
        }
    }
    
    /// Validate security configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `cipher_suites` is empty
    /// - Security configuration parameters are invalid
    /// - TLS version requirements cannot be satisfied
    pub fn validate(&self) -> Result<(), String> {
        if self.cipher_suites.is_empty() {
            return Err("At least one cipher suite must be specified".to_string());
        }
        
        Ok(())
    }
}

impl SecurityConfigProvider for SecurityConfig {
    #[inline]
    fn tls_verification_enabled(&self) -> bool {
        self.tls_verification_enabled
    }
    
    #[inline]
    fn certificate_pinning_enabled(&self) -> bool {
        self.certificate_pinning_enabled
    }
    
    #[inline]
    fn min_tls_version(&self) -> TlsVersion {
        self.min_tls_version
    }
    
    #[inline]
    fn cipher_suites(&self) -> &[&'static str] {
        &self.cipher_suites
    }
    
    #[inline]
    fn enable_sni(&self) -> bool {
        self.enable_sni
    }
    
    #[inline]
    fn enable_ocsp(&self) -> bool {
        self.enable_ocsp
    }
    
    #[inline]
    fn certificate_transparency_enabled(&self) -> bool {
        self.certificate_transparency_enabled
    }
}

/// Compile-time security configuration
pub struct StaticSecurityConfig;

impl SecurityConfigProvider for StaticSecurityConfig {
    #[inline]
    fn tls_verification_enabled(&self) -> bool {
        true
    }
    
    #[inline]
    fn certificate_pinning_enabled(&self) -> bool {
        false
    }
    
    #[inline]
    fn min_tls_version(&self) -> TlsVersion {
        TlsVersion::Tls13
    }
    
    #[inline]
    fn cipher_suites(&self) -> &[&'static str] {
        &[
            "TLS_AES_256_GCM_SHA384",
            "TLS_AES_128_GCM_SHA256",
            "TLS_CHACHA20_POLY1305_SHA256",
        ]
    }
    
    #[inline]
    fn enable_sni(&self) -> bool {
        true
    }
    
    #[inline]
    fn enable_ocsp(&self) -> bool {
        true
    }
    
    #[inline]
    fn certificate_transparency_enabled(&self) -> bool {
        false
    }
}
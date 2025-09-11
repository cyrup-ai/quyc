//! Main certificate builder entry point

use super::validator::CertificateValidator;
use super::generator::CertificateGenerator;

/// Main certificate builder entry point
#[derive(Debug, Clone)]
pub struct CertificateBuilder;

impl CertificateBuilder {
    #[must_use] 
    pub fn new() -> Self {
        Self
    }

    /// Create a certificate validator
    #[must_use] 
    pub fn validator(self) -> CertificateValidator {
        CertificateValidator::new()
    }

    /// Create a certificate generator
    #[must_use] 
    pub fn generator(self) -> CertificateGenerator {
        CertificateGenerator::new()
    }
}

impl Default for CertificateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
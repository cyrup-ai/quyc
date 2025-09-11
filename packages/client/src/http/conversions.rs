//! Type conversion utilities with zero-allocation optimizations

use std::convert::TryFrom;

use bytes::Bytes;

use crate::jsonpath::safe_parsing::{SafeParsingContext};
use crate::error::constructors::deserialization_error;
// Unicode normalization functions are called via fully qualified paths (unicode_normalization::is_nfc)
use aho_corasick::AhoCorasick;
use memchr::memchr;
use std::sync::OnceLock;

/// Convert value to bytes with zero-allocation optimization
#[inline]
pub fn to_bytes<T: AsRef<[u8]>>(value: T) -> Bytes {
    Bytes::copy_from_slice(value.as_ref())
}

/// Convert bytes to value with type validation
#[inline]
pub fn from_bytes<T: TryFrom<Vec<u8>>>(bytes: Bytes) -> Result<T, T::Error> {
    T::try_from(bytes.to_vec())
}

/// Convert value to string representation
#[inline]
pub fn to_string<T: ToString>(value: T) -> String {
    value.to_string()
}

/// Convert string to bytes
#[inline]
#[must_use] 
pub fn string_to_bytes(s: String) -> Bytes {
    Bytes::from(s)
}

/// Convert bytes to string with UTF-8 validation
#[inline]
pub fn bytes_to_string(bytes: Bytes) -> Result<String, crate::error::HttpError> {
    String::from_utf8(bytes.to_vec()).map_err(|e| deserialization_error(e.to_string()))
}

/// Convert slice to bytes
#[inline]
#[must_use] 
pub fn slice_to_bytes(slice: &[u8]) -> Bytes {
    Bytes::copy_from_slice(slice)
}

/// Convert vector to bytes with zero-copy optimization
#[inline]
#[must_use] 
pub fn vec_to_bytes(vec: Vec<u8>) -> Bytes {
    Bytes::from(vec)
}

/// Convert bytes to vector
#[inline]
pub fn bytes_to_vec(bytes: Bytes) -> Vec<u8> {
    bytes.to_vec()
}

/// Convert string slice to bytes
#[inline]
#[must_use] 
pub fn str_to_bytes(s: &str) -> Bytes {
    Bytes::copy_from_slice(s.as_bytes())
}

/// Convert bytes to string slice (borrowed)
#[inline]
pub fn bytes_to_str(bytes: &Bytes) -> Result<&str, crate::error::HttpError> {
    std::str::from_utf8(bytes)
        .map_err(|e| deserialization_error(format!("UTF-8 conversion failed: {e}")))
}

/// Enhanced UTF-8 validation modes based on security requirements
#[derive(Debug, Clone, Copy)]
pub enum SecurityMode {
    /// Performance-optimized - basic UTF-8 validation only
    Basic,
    /// Security-focused - strict validation with overlong detection  
    Strict,
    /// Paranoid - includes normalization and bidirectional checks
    Paranoid,
}

/// Secure bytes to string conversion with configurable validation
pub fn bytes_to_string_secure(
    bytes: Bytes, 
    mode: SecurityMode
) -> Result<String, crate::error::HttpError> {
    let context = SafeParsingContext::with_limits(10_000, true);
    
    match mode {
        SecurityMode::Basic => {
            if let Err(e) = context.validate_utf8_basic(&bytes) {
                return Err(deserialization_error(format!("UTF-8 validation failed: {e}")));
            }
            // Use high-performance SIMD validation
            match simdutf8::basic::from_utf8(&bytes) {
                Ok(s) => Ok(s.to_string()),
                Err(_) => Err(deserialization_error("SIMD UTF-8 validation failed".to_string()))
            }
        }
        SecurityMode::Strict => {
            if let Err(e) = context.validate_utf8_strict(&bytes) {
                return Err(deserialization_error(format!("UTF-8 validation failed: {e}")));
            }
            // Detect overlong encodings and invalid code points
            validate_strict_utf8(&bytes)?;
            // Multi-pattern security scanning
            scan_for_malicious_patterns(&bytes)?;
            // Safe conversion after comprehensive validation
            match String::from_utf8(bytes.to_vec()) {
                Ok(s) => Ok(s),
                Err(e) => Err(deserialization_error(format!("UTF-8 conversion failed after validation: {e}")))
            }
        }
        SecurityMode::Paranoid => {
            if let Err(e) = context.validate_utf8_paranoid(&bytes) {
                return Err(deserialization_error(format!("UTF-8 validation failed: {e}")));
            }
            // Convert to string for advanced checks (safe after strict validation)
            let text = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(e) => return Err(deserialization_error(format!("UTF-8 conversion failed: {e}")))
            };
            // Comprehensive Unicode normalization check - text must be in NFC form
            if !unicode_normalization::is_nfc(text) {
                return Err(deserialization_error("Text contains non-normalized Unicode sequences (not NFC)".to_string()));
            }
            // Advanced bidirectional attack detection
            detect_bidirectional_attacks(text)?;
            // Multi-pattern security scanning
            scan_for_malicious_patterns(&bytes)?;
            Ok(text.to_string())
        }
    }
}

/// Detect overlong encodings and invalid code points
pub fn validate_strict_utf8(bytes: &[u8]) -> Result<(), crate::error::HttpError> {
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i];
        
        // ASCII fast path
        if ch & 0x80 == 0 {
            i += 1;
            continue;
        }
        
        // Multi-byte sequences
        let (seq_len, min_value) = match ch {
            0xC2..=0xDF => (2, 0x80),    // 2-byte sequence
            0xE0..=0xEF => (3, 0x800),   // 3-byte sequence  
            0xF0..=0xF4 => (4, 0x10000), // 4-byte sequence
            _ => return Err(deserialization_error(format!("Invalid UTF-8 start byte: 0x{ch:02X} at position {i}"))),
        };
        
        // Check sequence length
        if i + seq_len > bytes.len() {
            return Err(deserialization_error(format!("Truncated UTF-8 sequence at position {i}")));
        }
        
        // Validate continuation bytes and extract code point
        let mut code_point = u32::from(ch & match seq_len { 2 => 0x1F, 3 => 0x0F, 4 => 0x07, _ => 0 });
        
        for j in 1..seq_len {
            let cont_byte = bytes[i + j];
            if cont_byte & 0xC0 != 0x80 {
                return Err(deserialization_error(format!("Invalid UTF-8 continuation byte: 0x{cont_byte:02X} at position {}", i + j)));
            }
            code_point = (code_point << 6) | u32::from(cont_byte & 0x3F);
        }
        
        // Check for overlong encodings
        if code_point < min_value {
            return Err(deserialization_error(format!("Overlong UTF-8 encoding detected at position {i}: codepoint U+{code_point:04X} in {seq_len}-byte sequence")));
        }
        
        // Check for invalid code points
        if code_point > 0x0010_FFFF || (0xD800..=0xDFFF).contains(&code_point) {
            return Err(deserialization_error(format!("Invalid UTF-8 code point: U+{code_point:04X} at position {i}")));
        }
        
        i += seq_len;
    }
    
    Ok(())
}

/// Detect bidirectional override attacks
pub fn detect_bidirectional_attacks(text: &str) -> Result<(), crate::error::HttpError> {
    for ch in text.chars() {
        match ch {
            '\u{202A}' | // LEFT-TO-RIGHT EMBEDDING
            '\u{202B}' | // RIGHT-TO-LEFT EMBEDDING  
            '\u{202C}' | // POP DIRECTIONAL FORMATTING
            '\u{202D}' | // LEFT-TO-RIGHT OVERRIDE
            '\u{202E}' | // RIGHT-TO-LEFT OVERRIDE (common in attacks)
            '\u{2066}' | // LEFT-TO-RIGHT ISOLATE
            '\u{2067}' | // RIGHT-TO-LEFT ISOLATE
            '\u{2068}' | // FIRST STRONG ISOLATE
            '\u{2069}'   // POP DIRECTIONAL ISOLATE
            => {
                return Err(deserialization_error(format!("Potentially malicious bidirectional Unicode character detected: U+{:04X}", ch as u32)));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Multi-pattern security scanner for detecting malicious content
static SECURITY_SCANNER: OnceLock<Result<AhoCorasick, String>> = OnceLock::new();

/// Initialize security patterns for comprehensive threat detection
fn get_security_scanner() -> Result<&'static AhoCorasick, crate::error::HttpError> {
    let scanner_result = SECURITY_SCANNER.get_or_init(|| {
        // Use Vec<Vec<u8>> for consistent typing - all patterns as owned byte vectors
        let patterns: Vec<Vec<u8>> = vec![
            // Bidirectional override characters (visual spoofing) - UTF-8 encoded
            vec![0xE2, 0x80, 0xAA], // U+202A LEFT-TO-RIGHT EMBEDDING
            vec![0xE2, 0x80, 0xAB], // U+202B RIGHT-TO-LEFT EMBEDDING
            vec![0xE2, 0x80, 0xAC], // U+202C POP DIRECTIONAL FORMATTING
            vec![0xE2, 0x80, 0xAD], // U+202D LEFT-TO-RIGHT OVERRIDE
            vec![0xE2, 0x80, 0xAE], // U+202E RIGHT-TO-LEFT OVERRIDE
            vec![0xE2, 0x81, 0xA6], // U+2066 LEFT-TO-RIGHT ISOLATE
            vec![0xE2, 0x81, 0xA7], // U+2067 RIGHT-TO-LEFT ISOLATE
            vec![0xE2, 0x81, 0xA8], // U+2068 FIRST STRONG ISOLATE
            vec![0xE2, 0x81, 0xA9], // U+2069 POP DIRECTIONAL ISOLATE
            
            // Overlong encodings (common attack vectors)
            vec![0xC0, 0x80], // Overlong encoding of NULL
            vec![0xC1, 0x81], // Overlong encoding
            vec![0xE0, 0x80, 0x80], // 3-byte overlong
            vec![0xF0, 0x80, 0x80, 0x80], // 4-byte overlong
            
            // Invalid surrogate pairs
            vec![0xED, 0xA0, 0x80], // High surrogate start
            vec![0xED, 0xAF, 0xBF], // High surrogate end
            vec![0xED, 0xB0, 0x80], // Low surrogate start
            vec![0xED, 0xBF, 0xBF], // Low surrogate end
            
            // Byte order marks (BOM attacks)
            vec![0xEF, 0xBB, 0xBF], // UTF-8 BOM
            vec![0xFF, 0xFE], // UTF-16 LE BOM
            vec![0xFE, 0xFF], // UTF-16 BE BOM
            
            // Control characters (potential injection)
            vec![0x00], vec![0x01], vec![0x02], vec![0x03], 
            vec![0x04], vec![0x05], vec![0x06], vec![0x07],
        ];
        
        match AhoCorasick::new(&patterns) {
            Ok(scanner) => Ok(scanner),
            Err(e) => Err(format!("AhoCorasick security scanner creation failed: {e}")),
        }
    });
    
    match scanner_result {
        Ok(scanner) => Ok(scanner),
        Err(e) => {
            tracing::error!(
                target: "quyc::security", 
                error = %e,
                "Critical security scanner failure - requests will be blocked"
            );
            Err(crate::error::constructors::security_error(format!(
                "Critical security scanner failure: {e}. Request blocked for safety."
            )))
        }
    }
}

/// Scan for malicious patterns using multi-pattern detection
pub fn scan_for_malicious_patterns(data: &[u8]) -> Result<(), crate::error::HttpError> {
    let scanner = get_security_scanner()?;
    
    if let Some(mat) = scanner.find(data) {
        let pattern_id = mat.pattern().as_usize();
        let position = mat.start();
        
        let threat_type = match pattern_id {
            0..=8 => "Bidirectional override attack",
            9..=12 => "Overlong UTF-8 encoding",
            13..=16 => "Invalid surrogate pair",
            17..=19 => "Byte order mark attack",
            20..=27 => "Control character injection",
            _ => "Unknown malicious pattern",
        };
        
        tracing::warn!(
            "Security threat detected: {} at position {} (pattern {})",
            threat_type, position, pattern_id
        );
        
        return Err(deserialization_error(format!(
            "Security threat detected: {threat_type} at position {position}"
        )));
    }
    
    Ok(())
}

/// Fast scan for common attack bytes using SIMD acceleration
#[must_use] 
pub fn fast_scan_malicious_bytes(data: &[u8]) -> Option<usize> {
    // Check for bidirectional override characters (most common attack)
    // UTF-8 encoding: U+202E = 0xE2 0x80 0xAE
    let mut pos = 0;
    while pos < data.len() {
        if let Some(offset) = memchr(0xE2, &data[pos..]) {
            let abs_pos = pos + offset;
            if abs_pos + 2 < data.len() 
                && data[abs_pos + 1] == 0x80 
                && data[abs_pos + 2] == 0xAE {
                return Some(abs_pos);
            }
            pos = abs_pos + 1;
        } else {
            break;
        }
    }
    
    None
}

/// Generic conversion with error handling
#[inline]
pub fn convert<T, U>(value: T) -> Result<U, crate::error::HttpError>
where
    T: TryInto<U>,
    T::Error: std::fmt::Display,
{
    value
        .try_into()
        .map_err(|e| deserialization_error(format!("Type conversion failed: {e}")))
}

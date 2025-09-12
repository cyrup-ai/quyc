//! DNS hostname validation
//!
//! This module provides security-conscious hostname validation for DNS operations.

/// Validate hostname format for security
///
/// # Errors
/// 
/// Returns `String` error if:
/// - Hostname is empty or contains only whitespace
/// - Hostname exceeds maximum length (253 characters per DNS standards)
/// - Hostname contains invalid characters (non-ASCII, spaces, or forbidden symbols)
/// - Hostname format violates DNS naming conventions or security policies
pub fn validate_hostname(hostname: &str) -> Result<(), String> {
    if hostname.is_empty() {
        return Err("Empty hostname".to_string());
    }
    if hostname.len() > 253 {
        return Err("Hostname too long (max 253 characters)".to_string());
    }

    // Check for invalid characters
    if !hostname
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err("Invalid characters in hostname".to_string());
    }

    // Check label constraints
    for label in hostname.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err("Invalid label length".to_string());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("Invalid label format".to_string());
        }
    }

    Ok(())
}



//! Certificate utility functions

use std::collections::HashMap;

/// Format a distinguished name HashMap into a string representation
pub fn format_dn_hashmap(dn: &HashMap<String, String>) -> String {
    let mut parts = Vec::new();
    
    // Order DN components in standard order: CN, O, OU, L, ST, C
    let ordered_keys = ["CN", "O", "OU", "L", "ST", "C"];
    
    for &key in &ordered_keys {
        if let Some(value) = dn.get(key) {
            parts.push(format!("{}={}", key, value));
        }
    }
    
    // Add any remaining keys that weren't in the standard order
    for (key, value) in dn.iter() {
        if !ordered_keys.contains(&key.as_str()) {
            parts.push(format!("{}={}", key, value));
        }
    }
    
    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        parts.join(", ")
    }
}
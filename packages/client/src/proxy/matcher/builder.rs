//! Builder pattern for proxy matcher configuration
//!
//! Provides fluent API for configuring proxy matching patterns
//! with support for inclusion/exclusion rules and protocol-specific settings.

use super::implementation::Matcher;

/// Builder for configuring proxy matcher patterns
#[derive(Debug)]
pub struct MatcherBuilder {
    pub(crate) all_patterns: Vec<String>,
    pub(crate) no_patterns: Vec<String>,
}

impl MatcherBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            all_patterns: Vec::new(),
            no_patterns: Vec::new(),
        }
    }

    /// Add pattern to match all requests
    pub fn all(mut self, pattern: String) -> Self {
        self.all_patterns.push(pattern);
        self
    }

    /// Add pattern to exclude from proxy
    pub fn no(mut self, pattern: &str) -> Self {
        if !pattern.is_empty() {
            self.no_patterns.push(pattern.to_string());
        }
        self
    }

    /// Add HTTP proxy pattern
    pub fn http(mut self, url: String) -> Self {
        self.all_patterns.push(url);
        self
    }

    /// Add HTTPS proxy pattern
    pub fn https(mut self, url: String) -> Self {
        self.all_patterns.push(url);
        self
    }

    /// Build the configured matcher
    pub fn build(self) -> Matcher {
        // Combine patterns with exclusions taking precedence
        let mut final_patterns = self.all_patterns;

        // Remove any patterns that match exclusion rules
        final_patterns.retain(|pattern| {
            !self
                .no_patterns
                .iter()
                .any(|no_pattern| pattern.contains(no_pattern) || no_pattern == "*")
        });

        Matcher::new(final_patterns)
    }
}

impl Default for MatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

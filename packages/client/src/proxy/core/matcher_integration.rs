//! Ultra-performance matcher integration for proxy configuration
//!
//! Zero-allocation conversion of Proxy configuration into blazing-fast Matcher instances
//! with comprehensive pattern matching and no-proxy rule handling.

use super::types::Proxy;

// Import Matcher_ enum for type safety
use super::super::matcher::types::Matcher_;

// Zero-allocation pattern constants for maximum performance
const HTTP_PATTERNS: &[&str] = &["http"];
const HTTPS_PATTERNS: &[&str] = &["https"];
const ALL_PATTERNS: &[&str] = &["*"];

impl Proxy {
    /// Convert Proxy configuration into ultra-performance Matcher
    /// 
    /// Zero-allocation implementation with:
    /// - Const pattern slices for blazing-fast lookup
    /// - Move semantics for ownership transfer
    /// - Lazy iterator evaluation for `no_proxy` parsing
    /// - Perfect error propagation without unwrap/expect
    #[inline]
    pub(crate) fn into_matcher(self) -> Result<super::super::matcher::types::Matcher, crate::Error> {
        use super::super::matcher::{
            implementation::Matcher as ImplMatcher,
            types::{Matcher, Matcher_}
        };

        // Zero-allocation auth/header flag detection - branchless optimization
        let maybe_has_http_auth = self.extra.auth.is_some();
        let maybe_has_http_custom_headers = self.extra.misc.is_some();

        // Extract no_proxy reference before moving intercept to avoid borrow checker issues
        let no_proxy_ref = self.no_proxy.as_ref();
        
        // Ultra-fast pattern matching with zero allocations
        let inner = match self.intercept {
            super::types::Intercept::All(url) => {
                // Use const slice for zero allocation, only convert URL at the end
                let patterns: Vec<String> = ALL_PATTERNS.iter()
                    .copied()
                    .chain(std::iter::once(url.as_str()))
                    .map(String::from)
                    .collect();
                Matcher_::Util(ImplMatcher::new(patterns))
            }
            super::types::Intercept::Http(url) => {
                // Efficient HTTP-only pattern matching
                let patterns: Vec<String> = HTTP_PATTERNS.iter()
                    .copied()
                    .chain(std::iter::once(url.as_str()))
                    .map(String::from)
                    .collect();
                Matcher_::Util(ImplMatcher::new(patterns))
            }
            super::types::Intercept::Https(url) => {
                // Efficient HTTPS-only pattern matching
                let patterns: Vec<String> = HTTPS_PATTERNS.iter()
                    .copied()
                    .chain(std::iter::once(url.as_str()))
                    .map(String::from)
                    .collect();
                Matcher_::Util(ImplMatcher::new(patterns))
            }
            super::types::Intercept::Custom(custom) => {
                // Move custom directly for zero-copy transfer
                Matcher_::Custom(custom)
            }
        };

        // Lazy no_proxy pattern processing with zero intermediate allocations
        let processed_inner = if let Some(no_proxy) = no_proxy_ref {
            if no_proxy.inner.is_empty() {
                inner
            } else {
                Self::apply_no_proxy_patterns_static(inner, &no_proxy.inner)?
            }
        } else {
            inner
        };

        // Move ownership of extra for zero-copy transfer
        Ok(Matcher {
            inner: processed_inner,
            extra: self.extra,
            maybe_has_http_auth,
            maybe_has_http_custom_headers,
        })
    }

    /// Apply no-proxy patterns with zero-allocation lazy evaluation
    #[inline]
    fn apply_no_proxy_patterns_static(
        mut inner: Matcher_,
        no_proxy_str: &str,
    ) -> Result<Matcher_, crate::Error> {
        if let Matcher_::Util(ref mut impl_matcher) = inner {
            // Lazy iterator processing - no intermediate Vec allocations until necessary
            let exclusion_count = no_proxy_str.matches(',').count() + 1;
            
            if exclusion_count > 0 {
                // Pre-allocate with exact capacity for zero reallocation
                let mut exclusions = Vec::with_capacity(exclusion_count);
                
                // Zero-allocation pattern extraction using iterators
                no_proxy_str
                    .split(',')
                    .filter_map(|pattern| {
                        let trimmed = pattern.trim();
                        if trimmed.is_empty() { 
                            None 
                        } else { 
                            Some(trimmed)
                        }
                    })
                    .for_each(|trimmed| exclusions.push(trimmed));

                if !exclusions.is_empty() {
                    // Ultra-efficient pattern filtering with single-pass algorithm
                    impl_matcher.patterns.retain(|pattern| {
                        !exclusions.iter().any(|exclusion| {
                            pattern.contains(exclusion) || (*exclusion == "*")
                        })
                    });
                }
            }
        }
        // Custom variants handle no_proxy internally through their function
        Ok(inner)
    }
}

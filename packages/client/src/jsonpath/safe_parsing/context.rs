//! Safe parsing context with resource limits and error recovery
//!
//! Provides controlled parsing environment that protects against various
//! attack vectors while maintaining functionality for legitimate use cases.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::jsonpath::error::{JsonPathResult, buffer_error, invalid_expression_error};

/// Maximum allowed nesting depth for `JSONPath` expressions
///
/// Prevents stack overflow and excessive memory usage from deeply nested
/// filter expressions or recursive descent operations.
pub const MAX_NESTING_DEPTH: usize = 100;

/// Maximum allowed expression complexity score
///
/// Prevents evaluation of expressions that would consume excessive resources.
pub const MAX_COMPLEXITY_SCORE: u32 = 10_000;

/// Maximum allowed buffer size for expression parsing
///
/// Prevents memory exhaustion from extremely large `JSONPath` expressions.
pub const MAX_BUFFER_SIZE: usize = 1_048_576; // 1MB

/// Maximum parsing time allowed for a single expression
///
/// Prevents denial of service through expressions that take too long to parse.
pub const MAX_PARSE_TIME: Duration = Duration::from_secs(5);

/// Global memory usage tracking for parsing operations
static GLOBAL_MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);

/// Safe parsing context with resource limits and error recovery
///
/// Provides controlled parsing environment that protects against various
/// attack vectors while maintaining functionality for legitimate use cases.
pub struct SafeParsingContext {
    /// Current nesting depth
    nesting_depth: usize,
    /// Memory allocated for this parsing context
    allocated_memory: usize,
    /// Start time for timeout tracking
    start_time: Instant,
    /// Whether strict UTF-8 validation is enabled
    strict_utf8: bool,
    /// Maximum allowed complexity for expressions
    max_complexity: u32,
}

impl Default for SafeParsingContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeParsingContext {
    /// Create new safe parsing context with default limits
    #[inline]
    #[must_use] 
    pub fn new() -> Self {
        Self {
            nesting_depth: 0,
            allocated_memory: 0,
            start_time: Instant::now(),
            strict_utf8: true,
            max_complexity: MAX_COMPLEXITY_SCORE,
        }
    }

    /// Create parsing context with custom limits
    #[inline]
    #[must_use] 
    pub fn with_limits(max_complexity: u32, strict_utf8: bool) -> Self {
        Self {
            nesting_depth: 0,
            allocated_memory: 0,
            start_time: Instant::now(),
            strict_utf8,
            max_complexity,
        }
    }

    /// Enter a new nesting level (increment depth)
    ///
    /// Returns error if maximum nesting depth would be exceeded.
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Maximum nesting depth limit would be exceeded
    /// - Stack overflow protection is triggered
    #[inline]
    pub fn enter_nesting(&mut self) -> JsonPathResult<()> {
        if self.nesting_depth >= MAX_NESTING_DEPTH {
            return Err(invalid_expression_error(
                "",
                format!("maximum nesting depth {MAX_NESTING_DEPTH} exceeded"),
                None,
            ));
        }

        self.nesting_depth += 1;
        Ok(())
    }

    /// Exit current nesting level (decrement depth)
    #[inline]
    pub fn exit_nesting(&mut self) {
        if self.nesting_depth > 0 {
            self.nesting_depth -= 1;
        }
    }

    /// Allocate memory with tracking and limits
    ///
    /// Tracks memory usage both locally and globally to prevent
    /// memory exhaustion attacks.
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Requested allocation would exceed local memory limit
    /// - Global memory usage limit would be exceeded
    /// - Memory tracking state is inconsistent
    #[inline]
    pub fn allocate_memory(&mut self, size: usize) -> JsonPathResult<()> {
        // Check local limits
        if self.allocated_memory + size > MAX_BUFFER_SIZE {
            return Err(buffer_error(
                "memory allocation",
                size,
                MAX_BUFFER_SIZE - self.allocated_memory,
            ));
        }

        // Check global limits (simple DoS protection)
        let global_usage = GLOBAL_MEMORY_USAGE.load(Ordering::Relaxed);
        if global_usage + size > MAX_BUFFER_SIZE * 10 {
            return Err(buffer_error(
                "global memory allocation",
                size,
                MAX_BUFFER_SIZE * 10 - global_usage,
            ));
        }

        // Update tracking
        self.allocated_memory += size;
        GLOBAL_MEMORY_USAGE.fetch_add(size, Ordering::Relaxed);

        Ok(())
    }

    /// Check if parsing time limit has been exceeded
    /// Check if parsing has exceeded maximum allowed time
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Parsing time has exceeded the maximum allowed duration
    /// - Timeout protection is triggered to prevent DoS attacks
    #[inline]
    pub fn check_timeout(&self) -> JsonPathResult<()> {
        if self.start_time.elapsed() > MAX_PARSE_TIME {
            return Err(invalid_expression_error(
                "",
                format!("parsing timeout after {MAX_PARSE_TIME:?}"),
                None,
            ));
        }
        Ok(())
    }

    /// Validate expression complexity
    /// Validate that expression complexity does not exceed limits
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Expression complexity score exceeds maximum allowed limit
    /// - Complexity protection is triggered to prevent resource exhaustion
    #[inline]
    pub fn validate_complexity(&self, complexity_score: u32) -> JsonPathResult<()> {
        if complexity_score > self.max_complexity {
            return Err(invalid_expression_error(
                "",
                format!(
                    "expression complexity {} exceeds limit {}",
                    complexity_score, self.max_complexity
                ),
                None,
            ));
        }
        Ok(())
    }

    /// Get current nesting depth
    #[inline]
    #[must_use] 
    pub fn nesting_depth(&self) -> usize {
        self.nesting_depth
    }

    /// Get allocated memory size
    #[inline]
    #[must_use] 
    pub fn allocated_memory(&self) -> usize {
        self.allocated_memory
    }

    /// Validate UTF-8 chunk with basic checks
    /// Validate UTF-8 chunk with basic checks
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Chunk contains invalid UTF-8 sequences and strict validation is enabled
    /// - UTF-8 validation fails at the byte level
    pub fn validate_utf8_basic(&self, chunk: &[u8]) -> JsonPathResult<()> {
        if !self.strict_utf8 {
            return Ok(());
        }
        
        // Use standard library UTF-8 validation
        std::str::from_utf8(chunk)
            .map_err(|e| self.utf8_error("Basic UTF-8 validation failed", e.valid_up_to(), None))?;
            
        Ok(())
    }
    
    /// Validate UTF-8 chunk with strict security checks  
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Chunk contains invalid UTF-8 sequences
    /// - Strict UTF-8 validation detects security-relevant encoding issues
    /// - Character validation fails security checks
    pub fn validate_utf8_strict(&self, chunk: &[u8]) -> JsonPathResult<()> {
        if !self.strict_utf8 {
            return Ok(());
        }
        
        // First do basic validation
        self.validate_utf8_basic(chunk)?;
        
        // Then check for security issues using the validation from http::conversions
        crate::http::conversions::validate_strict_utf8(chunk)
            .map_err(|e| invalid_expression_error("", e.to_string(), None))
    }
    
    /// Validate UTF-8 chunk with paranoid security checks
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Strict UTF-8 validation is enabled and the chunk fails strict validation
    /// - Text contains non-normalized Unicode sequences (not NFC form)  
    /// - Text contains suspicious Unicode patterns or control characters
    pub fn validate_utf8_paranoid(&self, chunk: &[u8]) -> JsonPathResult<()> {
        if !self.strict_utf8 {
            return Ok(());
        }
        
        // Do strict validation first
        self.validate_utf8_strict(chunk)?;
        
        // Convert to string for advanced checks (safe after strict validation)
        let text = std::str::from_utf8(chunk)
            .map_err(|e| invalid_expression_error("", format!("UTF-8 conversion failed: {e}"), None))?;
        
        // Unicode normalization validation - text must be in NFC form
        if !unicode_normalization::is_nfc(text) {
            return Err(invalid_expression_error(
                "", 
                "Text contains non-normalized Unicode sequences (not NFC)", 
                None
            ));
        }
        
        // Bidirectional attack detection  
        crate::http::conversions::detect_bidirectional_attacks(text)
            .map_err(|e| invalid_expression_error("", e.to_string(), None))?;
        
        // Multi-pattern security scanning
        crate::http::conversions::scan_for_malicious_patterns(chunk)
            .map_err(|e| invalid_expression_error("", e.to_string(), None))?;
            
        Ok(())
    }
    
    /// Create UTF-8 validation error with context
    fn utf8_error(&self, message: &str, position: usize, error_len: Option<usize>) -> crate::jsonpath::error::JsonPathError {
        let error_detail = if let Some(len) = error_len {
            format!("{message} at byte position {position} (error length: {len} bytes)")
        } else {
            format!("{message} at byte position {position}")
        };
        
        invalid_expression_error("", &error_detail, Some(position))
    }
}

impl Drop for SafeParsingContext {
    fn drop(&mut self) {
        // Release allocated memory from global tracking
        GLOBAL_MEMORY_USAGE.fetch_sub(self.allocated_memory, Ordering::Relaxed);
    }
}

/// Get current global memory usage for monitoring
#[inline]
pub fn global_memory_usage() -> usize {
    GLOBAL_MEMORY_USAGE.load(Ordering::Relaxed)
}

/// Reset global memory usage tracking (for testing)
#[inline]
pub fn reset_global_memory_tracking() {
    GLOBAL_MEMORY_USAGE.store(0, Ordering::Relaxed);
}

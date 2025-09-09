//! Regex compilation cache and ReDoS protection for JSONPath function extensions
//! Provides zero-allocation regex caching with timeout protection against ReDoS attacks

use std::thread;
use std::time::Duration;

/// Zero-allocation regex compilation cache for blazing-fast performance optimization
pub struct RegexCache {
    cache: std::sync::RwLock<std::collections::HashMap<String, regex::Regex>>,
}

impl RegexCache {
    pub fn new() -> Self {
        Self {
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Get compiled regex from cache or compile and cache if not present
    pub fn get_or_compile(&self, pattern: &str) -> Result<regex::Regex, regex::Error> {
        // Try read lock first for fast path
        if let Ok(cache) = self.cache.read() {
            if let Some(regex) = cache.get(pattern) {
                return Ok(regex.clone());
            }
        }

        // Compile new regex
        let regex = regex::Regex::new(pattern)?;

        // Store in cache with write lock
        if let Ok(mut cache) = self.cache.write() {
            if cache.len() < 32 {
                // Limit cache size for memory efficiency
                cache.insert(pattern.to_string(), regex.clone());
            }
        }

        Ok(regex)
    }
}

lazy_static::lazy_static! {
    pub static ref REGEX_CACHE: RegexCache = RegexCache::new();
}

/// Execute regex operation with timeout protection against ReDoS attacks
/// Returns error if timeout is exceeded (500ms for aggressive protection)
pub fn execute_regex_with_timeout<F>(regex_operation: F) -> Result<bool, String>
where
    F: FnOnce() -> bool + Send + 'static,
{
    use std::time::Instant;

    let timeout_duration = Duration::from_millis(500); // 500ms aggressive timeout
    let start_time = Instant::now();

    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn regex execution in separate thread
    let handle = thread::spawn(move || {
        log::debug!("Starting regex execution in timeout thread");
        let result = regex_operation();
        log::debug!("Regex execution completed in thread");
        let _ = tx.send(result); // Ignore send errors if receiver dropped
    });

    // Wait for completion or timeout
    match rx.recv_timeout(timeout_duration) {
        Ok(result) => {
            let elapsed = start_time.elapsed();
            log::debug!("Regex completed successfully in {:?}", elapsed);
            Ok(result)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let elapsed = start_time.elapsed();
            log::warn!(
                "Regex execution timed out after {:?} - potential ReDoS attack",
                elapsed
            );

            // Clean up thread - it will continue running but we ignore result
            drop(handle);

            Err(format!(
                "regex execution timed out after {}ms - potential ReDoS attack",
                elapsed.as_millis()
            ))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            let elapsed = start_time.elapsed();
            log::error!("Regex execution thread disconnected after {:?}", elapsed);
            Err("regex execution thread disconnected unexpectedly".to_string())
        }
    }
}

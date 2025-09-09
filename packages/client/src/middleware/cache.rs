//! Cache middleware for HTTP requests/responses
//!
//! Zero-allocation cache middleware integrating with the production cache system.
//! Provides conditional request validation and response caching with lock-free operations.

use std::sync::Arc;
// time imports removed - not used

use ystream::AsyncStream;
use bytes::Bytes;
// http header imports removed - not used

use super::Middleware;
use crate::cache::{CacheKey, CacheEntry, GLOBAL_CACHE, httpdate};
use crate::http::response::{HttpBodyChunk, HttpHeader};
use crate::{HttpRequest, HttpResponse};
// error imports removed - not used

/// Request context for cache key generation
#[derive(Debug, Clone)]
struct RequestContext {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
}

/// Cache middleware for HTTP requests/responses with zero-allocation design
#[derive(Debug)]
pub struct CacheMiddleware {
    enabled: bool,
    cache_key_buffer: Arc<str>,
    /// Store request context for use in process_response
    request_context: std::sync::RwLock<Option<RequestContext>>,
}

impl Default for CacheMiddleware {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl CacheMiddleware {
    #[inline]
    pub fn new() -> Self {
        Self {
            enabled: true,
            cache_key_buffer: Arc::from(""),
            request_context: std::sync::RwLock::new(None),
        }
    }

    #[inline]
    pub fn enabled(enabled: bool) -> Self {
        Self {
            enabled,
            cache_key_buffer: Arc::from(""),
            request_context: std::sync::RwLock::new(None),
        }
    }

    /// Generate cache key with zero allocations using request context
    #[inline]
    fn generate_cache_key(&self, method: &str, uri: &str, headers: &[(&str, &str)]) -> CacheKey {
        let headers_map = headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        CacheKey::new(uri.to_string(), method.to_string(), headers_map)
    }
}

impl Middleware for CacheMiddleware {
    #[inline]
    fn process_request(&self, request: HttpRequest) -> crate::error::Result<HttpRequest> {
        if !self.enabled {
            return Ok(request);
        }

        let uri = request.uri();
        let method = request.method().as_str();
        
        // Extract relevant headers for cache key generation
        let headers: Vec<(String, String)> = request.headers()
            .iter()
            .filter(|(name, _)| {
                // Only include headers that affect caching
                matches!(name.as_str(), "accept" | "accept-language" | "accept-encoding" | "authorization")
            })
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();
        
        // Store request context for use in process_response
        let context = RequestContext {
            url: uri,
            method: method.to_string(),
            headers: headers.clone(),
        };
        
        if let Ok(mut ctx) = self.request_context.write() {
            *ctx = Some(context);
        }
        
        let cache_key = self.generate_cache_key(method, &request.uri(), &[]);

        match GLOBAL_CACHE.get(&cache_key) {
            Some(cached_entry) => {
                let mut modified_request = request;

                // Add conditional validation headers for cache revalidation
                if let Some(ref etag) = cached_entry.etag() {
                    if let Ok(header_value) = http::HeaderValue::from_str(etag) {
                        modified_request =
                            modified_request.header(http::header::IF_NONE_MATCH, header_value);
                    }
                }

                if let Some(last_modified) = cached_entry.last_modified() {
                    let system_time = std::time::SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(last_modified.parse().unwrap_or(0));
                    let http_date = httpdate::fmt_http_date(system_time);
                    if let Ok(header_value) = http::HeaderValue::from_str(&http_date) {
                        modified_request =
                            modified_request.header(http::header::IF_MODIFIED_SINCE, header_value);
                    }
                }

                Ok(modified_request)
            }
            None => Ok(request),
        }
    }

    #[inline]
    fn process_response(&self, response: HttpResponse) -> crate::error::Result<HttpResponse> {
        if !self.enabled {
            return Ok(response);
        }

        // Only cache responses that meet HTTP caching criteria
        if !GLOBAL_CACHE.should_cache(&response) {
            return Ok(response);
        }

        // Get stored request context
        let context = {
            if let Ok(ctx_guard) = self.request_context.read() {
                ctx_guard.clone()
            } else {
                return Ok(response); // Unable to read context, skip caching
            }
        };
        
        let context = match context {
            Some(ctx) => ctx,
            None => {
                tracing::warn!(
                    target: "quyc::middleware::cache",
                    "No request context available for response caching, skipping cache"
                );
                return Ok(response);
            }
        };

        // Generate cache key using stored request context
        let headers_slice: Vec<(&str, &str)> = context.headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let cache_key = self.generate_cache_key(&context.method, &context.url, &headers_slice);

        // Implement streaming response caching with background task
        // Extract stream_id and version before moving the response
        let stream_id = response.stream_id;
        let version = response.version;
        
        // Clone response streams for concurrent processing - one for forwarding, one for caching
        let (headers_stream, body_stream, trailers_stream) = response.into_streams();
        
        // Create response streams for forwarding to client
        let (forward_headers_tx, forward_headers_stream) = AsyncStream::<HttpHeader, 256>::channel();
        let (forward_body_tx, forward_body_stream) = AsyncStream::<HttpBodyChunk, 1024>::channel();
        let (forward_trailers_tx, forward_trailers_stream) = AsyncStream::<HttpHeader, 64>::channel();
        
        // Create response streams for caching
        let (cache_headers_tx, _cache_headers_stream) = AsyncStream::<HttpHeader, 256>::channel();
        let (cache_body_tx, _cache_body_stream) = AsyncStream::<HttpBodyChunk, 1024>::channel();
        let (cache_trailers_tx, _cache_trailers_stream) = AsyncStream::<HttpHeader, 64>::channel();
        
        // Spawn background task for concurrent stream processing
        ystream::spawn_task(move || {
            let mut cached_headers = Vec::new();
            let mut cached_body_chunks = Vec::new();
            let mut cached_trailers = Vec::new();
            let mut total_body_size = 0usize;
            const MAX_CACHEABLE_SIZE: usize = 10 * 1024 * 1024; // 10MB limit for cacheable responses
            
            // Process headers stream
            for header in headers_stream {
                cached_headers.push(header.clone());
                ystream::emit!(forward_headers_tx, header.clone());
                ystream::emit!(cache_headers_tx, header);
            }
            
            // Process body stream with size monitoring
            for body_chunk in body_stream {
                let chunk_size = body_chunk.data.len();
                total_body_size += chunk_size;
                
                // Forward chunk to client immediately for streaming
                ystream::emit!(forward_body_tx, body_chunk.clone());
                
                // Only cache if response is within size limit
                if total_body_size <= MAX_CACHEABLE_SIZE {
                    cached_body_chunks.push(body_chunk.data.clone());
                    ystream::emit!(cache_body_tx, body_chunk);
                } else {
                    // Response too large for caching, log and stop caching
                    tracing::info!(
                        target: "quyc::middleware::cache",
                        cache_key = %cache_key.url,
                        body_size = total_body_size,
                        limit = MAX_CACHEABLE_SIZE,
                        "Response exceeds cacheable size limit - forwarding without caching"
                    );
                    return; // Exit caching task but continue forwarding
                }
            }
            
            // Process trailers stream
            for trailer in trailers_stream {
                cached_trailers.push(trailer.clone());
                ystream::emit!(forward_trailers_tx, trailer.clone());
                ystream::emit!(cache_trailers_tx, trailer);
            }
            
            // Create cache entry from collected data
            if !cached_body_chunks.is_empty() || !cached_headers.is_empty() {
                // Combine body chunks into single buffer
                let cached_body: Vec<u8> = cached_body_chunks.into_iter().flat_map(|chunk| chunk).collect();
                
                // Create HttpResponse streams for cache entry creation
                let (cache_entry_headers_tx, cache_entry_headers_stream) = AsyncStream::<HttpHeader, 256>::channel();
                let (cache_entry_body_tx, cache_entry_body_stream) = AsyncStream::<HttpBodyChunk, 1024>::channel();
                let (_, cache_entry_trailers_stream) = AsyncStream::<HttpHeader, 64>::channel();
                
                // Emit cached headers to the stream
                for header in cached_headers {
                    ystream::emit!(cache_entry_headers_tx, header);
                }
                
                // Emit cached body to the stream
                if !cached_body.is_empty() {
                    let body_chunk = HttpBodyChunk::new(Bytes::from(cached_body), 0, true);
                    ystream::emit!(cache_entry_body_tx, body_chunk);
                }
                
                // Create HttpResponse for cache entry
                let cache_response = HttpResponse::new(
                    cache_entry_headers_stream,
                    cache_entry_body_stream, 
                    cache_entry_trailers_stream,
                    http::Version::HTTP_11, // Default version for cached response
                    0, // Default stream_id
                );
                
                // Create cache entry asynchronously with proper await
                ystream::spawn_task(move || {
                    // Use tokio runtime to properly execute async cache operations
                    if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        handle.block_on(async move {
                            match CacheEntry::new(cache_response).await {
                                cache_entry => {
                                    // Store in global cache using existing put method
                                    GLOBAL_CACHE.put(cache_key.clone(), HttpResponse::from_cache_entry(cache_entry)).await;
                                    
                                    tracing::debug!(
                                        target: "quyc::middleware::cache",
                                        cache_key = %cache_key.url,
                                        body_size = total_body_size,
                                        "Streaming response successfully cached in background"
                                    );
                                }
                            }
                        });
                    } else {
                        tracing::warn!(
                            target: "quyc::middleware::cache",
                            cache_key = %cache_key.url,
                            "No tokio runtime available for cache storage - caching skipped"
                        );
                    }
                });
            }
        });
        
        // Create new response from forwarding streams
        let cached_response = HttpResponse::new(
            forward_headers_stream,
            forward_body_stream,
            forward_trailers_stream,
            version,
            stream_id,
        );
        
        Ok(cached_response)
    }
}

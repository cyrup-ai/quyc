//! Cache integration with HTTP client and global cache instance
//!
//! Provides global cache instance and cache-aware streaming functions
//! for seamless integration with the HTTP client.

use std::collections::HashMap;

use ystream::AsyncStream;

use super::{cache_key::CacheKey, response_cache::ResponseCache};
use crate::prelude::*;

/// Cache size limit (50MB)
const MAX_CACHE_SIZE: usize = 50 * 1024 * 1024;

/// Global cache instance for use across the HTTP client
pub static GLOBAL_CACHE: std::sync::LazyLock<ResponseCache> = std::sync::LazyLock::new(ResponseCache::default);

/// Cache-aware HTTP stream that checks cache before making requests using `AsyncStream`
pub fn cached_stream<F>(cache_key: CacheKey, operation: F) -> AsyncStream<HttpResponse, 1024>
where
    F: Fn() -> AsyncStream<HttpResponse, 1024> + Send + Sync + 'static,
{
    AsyncStream::with_channel(move |sender| {
        // Check cache first
        if let Some(cached_response) = GLOBAL_CACHE.get(&cache_key) {
            ystream::emit!(sender, cached_response);
            return;
        }

        // Cache miss - execute operation stream
        let operation_stream = operation();
        for response in operation_stream {
            if response.is_error() {
                // Forward error chunks as-is
                ystream::emit!(sender, response);
            } else {
                // Implement proper cache integration for streaming responses
                // Use streaming approach: tee the response streams for concurrent caching and forwarding
                let response_stream_id = response.stream_id;
                let response_version = response.version;
                let (headers_stream, body_stream, trailers_stream) = response.into_streams();
                
                // Create forwarding streams for the client
                let (forward_headers_tx, forward_headers_stream) = AsyncStream::<crate::http::response::HttpHeader, 256>::channel();
                let (forward_body_tx, forward_body_stream) = AsyncStream::<crate::http::response::HttpBodyChunk, 1024>::channel();
                let (forward_trailers_tx, forward_trailers_stream) = AsyncStream::<crate::http::response::HttpHeader, 64>::channel();
                
                // Create cache storage - spawn background task for concurrent processing
                let cache_key_clone = cache_key.clone();
                ystream::spawn_task(move || {
                    let mut cached_headers = Vec::new();
                    let mut cached_body_data = Vec::new();
                    let mut cached_trailers = Vec::new();
                    let mut total_size = 0usize;
                    
                    // Process headers - forward and cache simultaneously
                    for header in headers_stream {
                        cached_headers.push(header.clone());
                        ystream::emit!(forward_headers_tx, header);
                    }
                    
                    // Process body - forward and cache with size limits
                    for body_chunk in body_stream {
                        let chunk_size = body_chunk.data.len();
                        total_size += chunk_size;
                        
                        // Always forward to client for streaming
                        ystream::emit!(forward_body_tx, body_chunk.clone());
                        
                        // Cache only if within reasonable size limits
                        if total_size <= MAX_CACHE_SIZE {
                            cached_body_data.extend_from_slice(&body_chunk.data);
                        }
                    }
                    
                    // Process trailers - forward and cache
                    for trailer in trailers_stream {
                        cached_trailers.push(trailer.clone());
                        ystream::emit!(forward_trailers_tx, trailer);
                    }
                    
                    // Store in cache if size is reasonable
                    if total_size > 0 && total_size <= MAX_CACHE_SIZE {
                        // Create cache streams for storage
                        let (cache_headers_tx, cache_headers_stream) = AsyncStream::channel();
                        let (cache_body_tx, cache_body_stream) = AsyncStream::channel();
                        let (cache_trailers_tx, cache_trailers_stream) = AsyncStream::channel();
                        
                        // Emit cached data to storage streams
                        for header in cached_headers {
                            ystream::emit!(cache_headers_tx, header);
                        }
                        
                        if !cached_body_data.is_empty() {
                            let cached_body_chunk = crate::http::response::HttpBodyChunk::new(
                                bytes::Bytes::from(cached_body_data), 0, true
                            );
                            ystream::emit!(cache_body_tx, cached_body_chunk);
                        }
                        
                        for trailer in cached_trailers {
                            ystream::emit!(cache_trailers_tx, trailer);
                        }
                        
                        // Create cached response for storage
                        let cached_response = crate::HttpResponse::new(
                            cache_headers_stream,
                            cache_body_stream,
                            cache_trailers_stream,
                            http::Version::HTTP_11, // Normalized version for cache
                            0, // Stream ID not relevant for cached response
                        );
                        
                        // Store the response asynchronously using tokio runtime
                        ystream::spawn_task(move || {
                            // Use tokio runtime to properly execute async cache operations
                            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                handle.block_on(async move {
                                    let cache_entry = super::CacheEntry::new(cached_response).await;
                                    {
                                        // Store in global cache - create HttpResponse from cache entry
                                        let stored_response = crate::HttpResponse::from_cache_entry(cache_entry);
                                        GLOBAL_CACHE.put(cache_key_clone.clone(), stored_response).await;
                                        
                                        tracing::debug!(
                                            target: "quyc::cache::integration",
                                            cache_key = %cache_key_clone.url,
                                            total_size = total_size,
                                            "Streaming response successfully cached via integration layer"
                                        );
                                    }
                                });
                            } else {
                                tracing::warn!(
                                    target: "quyc::cache::integration",
                                    cache_key = %cache_key_clone.url,
                                    "No tokio runtime available for cache integration storage - caching skipped"
                                );
                            }
                        });
                    }
                });
                
                // Create forwarded response from streaming channels
                let forwarded_response = crate::HttpResponse::new(
                    forward_headers_stream,
                    forward_body_stream,
                    forward_trailers_stream,
                    response_version,
                    response_stream_id,
                );
                
                ystream::emit!(sender, forwarded_response);
            }
        }
    })
}

/// Helper to create conditional request headers for cache validation
#[must_use] 
pub fn conditional_headers_for_key(cache_key: &CacheKey) -> HashMap<String, String> {
    GLOBAL_CACHE
        .get_validation_headers(cache_key)
        .unwrap_or_default()
}

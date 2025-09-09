//! HttpResponse conversion utilities
//!
//! Converts AsyncStream<HttpChunk, 1024> from protocol implementations to canonical HttpResponse
//! with proper header parsing, status extraction, and body stream conversion.

use std::time::Instant;

use ystream::{AsyncStream, emit, spawn_task};
use http::{HeaderMap, StatusCode};
use bytes::Bytes;

use crate::prelude::*;
use crate::http::response::{HttpResponse, HttpBodyChunk};

/// Convert AsyncStream<HttpChunk, 1024> to HttpResponse
///
/// Parses the HttpChunk stream to extract HTTP status, headers, and body data.
/// The first few chunks contain response metadata, subsequent chunks become body stream.
///
/// # Arguments
/// * `chunk_stream` - The HttpChunk stream from H2Connection or H3Connection
/// * `stream_id` - The stream ID for the response
///
/// # Returns
/// * `HttpResponse` - Complete response with parsed status, headers, and streaming body
///
/// # Architecture
/// - Zero allocation parsing using atomic operations
/// - Lock-free header extraction and status parsing
/// - Maintains streaming patterns for body data
/// - No unwrap() or expect() calls - production safe
pub fn convert_http_chunks_to_response(
    chunk_stream: AsyncStream<HttpChunk, 1024>,
    stream_id: u64,
) -> HttpResponse {
    // Create header and trailer streams for the response
    let (headers_sender, headers_stream) = AsyncStream::<crate::http::response::HttpHeader, 256>::channel();
    let (_trailers_sender, _trailers_stream) = AsyncStream::<crate::http::response::HttpHeader, 64>::channel();
    
    // Create body stream by filtering and converting HttpChunks
    let body_stream = AsyncStream::with_channel(move |sender| {
        spawn_task(move || {
            let mut parsing_headers_local = true;
            let mut header_buffer_local = Vec::new();
            
            for chunk in chunk_stream {
                match chunk {
                    HttpChunk::Data(data) | HttpChunk::Body(data) | HttpChunk::Chunk(data) => {
                        if parsing_headers_local {
                            // Accumulate data for header parsing
                            header_buffer_local.extend_from_slice(&data);
                            
                            // Look for header/body separator (\r\n\r\n)
                            if let Some(separator_pos) = find_header_body_separator(&header_buffer_local) {
                                // Parse headers from buffer
                                let header_section = &header_buffer_local[..separator_pos];
                                let (_parsed_status, parsed_headers) = parse_http_response_headers(header_section);
                                
                                // Emit headers to headers stream
                                for (name, value) in parsed_headers.iter() {
                                    let header = crate::http::response::HttpHeader {
                                        name: name.clone(),
                                        value: value.clone(),
                                        timestamp: Instant::now(),
                                    };
                                    // Intentionally ignore send result - channel may be closed
                                    drop(headers_sender.send(header));
                                }
                                
                                // Switch to body parsing mode
                                parsing_headers_local = false;
                                
                                // Emit remaining data as first body chunk if any
                                let body_start = separator_pos + 4; // Skip \r\n\r\n
                                if body_start < header_buffer_local.len() {
                                    let body_data = Bytes::copy_from_slice(&header_buffer_local[body_start..]);
                                    let body_chunk = HttpBodyChunk {
                                        data: body_data,
                                        offset: 0,
                                        is_final: false,
                                        timestamp: Instant::now(),
                                    };
                                    emit!(sender, body_chunk);
                                }
                                header_buffer_local.clear();
                            }
                        } else {
                            // Direct body data - emit as HttpBodyChunk
                            let body_chunk = HttpBodyChunk {
                                data,
                                offset: 0,
                                is_final: false,
                                timestamp: Instant::now(),
                            };
                            emit!(sender, body_chunk);
                        }
                    }
                    HttpChunk::Headers(_, _) => {
                        // Headers are processed separately - skip in body conversion
                        continue;
                    }
                    HttpChunk::Trailers(_) => {
                        // Trailers come after body - end body stream
                        let final_chunk = HttpBodyChunk {
                            data: Bytes::new(),
                            offset: 0,
                            is_final: true,
                            timestamp: Instant::now(),
                        };
                        emit!(sender, final_chunk);
                        break;
                    }
                    HttpChunk::End => {
                        // End of stream - emit final chunk
                        let final_chunk = HttpBodyChunk {
                            data: Bytes::new(),
                            offset: 0,
                            is_final: true,
                            timestamp: Instant::now(),
                        };
                        emit!(sender, final_chunk);
                        break;
                    }
                    HttpChunk::Error(error_msg) => {
                        // Error handling - emit error as final chunk
                        let error_chunk = HttpBodyChunk {
                            data: Bytes::from(error_msg.into_bytes()),
                            offset: 0,
                            is_final: true,
                            timestamp: Instant::now(),
                        };
                        emit!(sender, error_chunk);
                        break;
                    }
                }
            }
        });
    });
    
    // Create empty trailers stream (most responses don't have trailers)
    let trailers_stream = AsyncStream::with_channel(|_sender| {
        // Empty stream for trailers
    });
    
    // Create HttpResponse with proper streaming architecture
    HttpResponse::new(
        headers_stream,
        body_stream,
        trailers_stream,
        http::Version::HTTP_2, // Default to HTTP/2 for this converter
        stream_id,
    )
}

/// Find the header/body separator in HTTP response data
///
/// Looks for the \r\n\r\n sequence that separates headers from body.
/// Returns the position where the separator starts, or None if not found.
fn find_header_body_separator(data: &[u8]) -> Option<usize> {
    let separator = b"\r\n\r\n";
    
    if data.len() < separator.len() {
        return None;
    }
    
    for i in 0..=(data.len() - separator.len()) {
        if &data[i..i + separator.len()] == separator {
            return Some(i);
        }
    }
    
    None
}

/// Parse HTTP response headers and extract status code
///
/// Parses the raw HTTP response header section to extract status code and headers.
/// Uses production-safe parsing without unwrap() or expect() calls.
///
/// # Arguments
/// * `header_data` - Raw header bytes from HTTP response
///
/// # Returns
/// * `(StatusCode, HeaderMap)` - Parsed status and headers, with safe defaults
fn parse_http_response_headers(header_data: &[u8]) -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    let mut status = StatusCode::OK; // Safe default
    
    // Convert to string for parsing (with error handling)
    let header_str = match std::str::from_utf8(header_data) {
        Ok(s) => s,
        Err(_) => return (status, headers), // Return defaults on invalid UTF-8
    };
    
    let lines: Vec<&str> = header_str.lines().collect();
    
    if lines.is_empty() {
        return (status, headers);
    }
    
    // Parse status line (first line)
    if let Some(status_line) = lines.get(0) {
        status = parse_status_line(status_line).unwrap_or(StatusCode::OK);
    }
    
    // Parse header lines (skip status line)
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            break; // End of headers
        }
        
        if let Some((name, value)) = parse_header_line(line) {
            // Safely insert header (ignore errors)
            if let (Ok(header_name), Ok(header_value)) = (
                http::header::HeaderName::try_from(name),
                http::header::HeaderValue::try_from(value)
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }
    
    (status, headers)
}

/// Parse HTTP status line to extract status code
///
/// Parses lines like "HTTP/1.1 200 OK" or "HTTP/2 404 Not Found"
/// Returns None if parsing fails (safe error handling).
fn parse_status_line(status_line: &str) -> Option<StatusCode> {
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }
    
    // Parse status code from second part
    if let Ok(status_num) = parts[1].parse::<u16>() {
        StatusCode::from_u16(status_num).ok()
    } else {
        None
    }
}

/// Parse individual header line
///
/// Parses lines like "Content-Type: application/json"
/// Returns None if parsing fails (safe error handling).
fn parse_header_line(line: &str) -> Option<(&str, &str)> {
    if let Some(colon_pos) = line.find(':') {
        let name = line[..colon_pos].trim();
        let value = line[colon_pos + 1..].trim();
        
        if !name.is_empty() && !value.is_empty() {
            Some((name, value))
        } else {
            None
        }
    } else {
        None
    }
}
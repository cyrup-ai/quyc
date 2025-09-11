# HTTP Compression Implementation Plan - Correct Architecture

## Pre-planner Orientation

The user wants HTTP compression in their HTTP/3 client. The objective is transparent compression/decompression that doesn't modify the user's request/response objects.

The current implementation was architecturally wrong - it tried to modify request bodies in the client layer instead of handling compression transparently at the protocol transmission layer.

The correct architecture should be:
- User calls client.execute(request) with original data
- Protocol layer compresses during wire transmission
- Protocol layer decompresses during reception
- User receives original-format responses
- Compression is completely transparent
- Full metrics and configuration work

## Milestone Analysis

**Completed Milestones:**
- ✅ Dependencies added (flate2, brotli)
- ✅ Core compression module with algorithms
- ✅ Configuration fields and builder methods
- ✅ Metrics infrastructure in ClientStats

**Current Milestone:**
Fix the fundamental architecture - move compression to protocol layer

**Success Criteria:**
HTTP compression that is completely transparent:
- User sends HttpRequest with JSON body
- Protocol compresses JSON→bytes during transmission
- Remote receives compressed data
- Response is decompressed bytes→data during reception
- User receives normal HttpResponse
- Original request/response objects never modified
- Full metrics recorded
- All configurations work

**Demonstration Goals:**
- Send 1MB JSON request, verify wire compression
- Receive compressed response, verify transparent decompression
- Metrics show compression ratios, times, byte counts
- Configuration controls work (algorithms, levels)
- Code compiles without errors
- No unwrap/expect in src code

## Research Results

### HTTP/2 and HTTP/3 Protocol Analysis

**HTTP/2 (h2 crate)**:
- Uses HPACK for header compression (built into protocol)
- API: `h2::hpack::Encoder` and `h2::hpack::Decoder`
- Headers are automatically compressed by the protocol layer
- Body compression is separate, application-layer concern

**HTTP/3 (h3 crate)**:
- Uses QPACK for header compression (evolution of HPACK for QUIC)
- API: `h3::qpack::encode_stateless()` and `h3::qpack::decode_stateless()`
- Headers are automatically compressed by the protocol layer
- Body compression is separate, application-layer concern

**Quiche HTTP/3 Body APIs - The Perfect Compression Hooks**:
```rust
// For REQUEST compression - compress BEFORE calling this
pub fn send_body(
    &mut self, 
    conn: &mut Connection, 
    stream_id: u64, 
    body: &[u8],  // <-- This is where we apply compression
    fin: bool
) -> Result<usize>

// For RESPONSE decompression - decompress AFTER calling this  
pub fn recv_body(
    &mut self,
    conn: &mut Connection,
    stream_id: u64,
    out: &mut [u8] // <-- This is where we apply decompression
) -> Result<usize>
```

**Compression Libraries**:
- **flate2**: Provides gzip/deflate with multiple backends (rust_backend, zlib-rs)
- **brotli**: Pure Rust brotli implementation
- Both support streaming compression for large payloads

### Current Broken Implementation Analysis

**Critical Architectural Errors in `/packages/client/src/client/core.rs:250-450`**:

1. **Request Mutation**: Tries to modify immutable `HttpRequest` object
   ```rust
   // BROKEN: These APIs don't exist or work this way
   modified_request = modified_request.body_bytes(compressed_data);
   modified_request.headers_mut().insert(header);
   ```

2. **Wrong Layer**: Compression happens in client layer instead of protocol transmission layer

3. **User Visible**: User's original request object gets modified

4. **Type Errors**: Using non-existent APIs like `.body_bytes()` and `.headers_mut()`

**Production Quality Violations**:
- `unwrap()` usage in `/packages/client/src/http/compression.rs:167,172`
- `assert!()` panics in `/packages/client/src/client/configuration.rs:89,102,115`
- Missing metrics integration in protocol layer

## Correct Architecture Strategy

### The Protocol-Layer Approach

**Key Insight**: Compression should be completely transparent to the user. The user's `HttpRequest` and `HttpResponse` objects should never be modified. Instead, compression happens during wire transmission/reception.

**Request Flow (Transparent Compression)**:
1. User calls `client.execute(request)` with original JSON body
2. Request passes unchanged to protocol layer
3. Protocol layer calls `H3RequestProcessor.prepare_request_body()` 
4. Body gets serialized to bytes AND compressed here
5. Compressed bytes are passed to `h3_conn.send_body(compressed_bytes)`
6. Content-Encoding header added to wire headers (not user's request)
7. User's original request object remains untouched

**Response Flow (Transparent Decompression)**:
1. Protocol layer calls `h3_conn.recv_body()` to get compressed bytes
2. Protocol layer detects compression from Content-Encoding header
3. Protocol layer decompresses bytes automatically
4. Decompressed data flows to user as normal `HttpResponse`
5. User receives original-format data, never knows compression occurred

### Implementation Hooks

**Perfect Request Compression Hook**: 
- Location: `H3RequestProcessor.prepare_request_body()` in `/packages/client/src/protocols/h3/strategy/processing.rs:124`
- Action: After serializing body to bytes, compress if worthwhile
- Headers: Add Content-Encoding to H3 headers (not user's request)

**Perfect Response Decompression Hook**:
- Location: `H3RequestProcessor.process_response_data()` in `/packages/client/src/protocols/h3/strategy/processing.rs:506`
- Action: After receiving bytes from `h3_conn.recv_body()`, decompress if Content-Encoding indicates compression

## TODO.md

### Phase 1: Remove Broken Request Modification

1. **Remove request body modification from HttpClient.execute()**
   - File: `packages/client/src/client/core.rs`
   - Lines: 250-450 (the entire compression section)
   - Remove all `modified_request = modified_request.body_bytes(...)` attempts
   - Remove `modified_request.headers_mut().insert(...)` attempts  
   - Remove header manipulation in client layer
   - Restore simple `strategy.execute(request)` call
   - Keep only Accept-Encoding header addition for response compression
   - The broken code tries to use non-existent APIs like `.body_bytes()` and `.headers_mut()`
   - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

2. **Act as an Objective QA Rust developer and verify the HttpClient.execute() method correctly passes original requests to protocol layer without modification, and that the code compiles without errors related to request body mutation.**

3. **Fix HttpConfig default constructor compilation error**
   - File: `packages/client/src/config/client.rs`
   - Line: 37
   - Add missing fields: `gzip_level: None, brotli_level: None, deflate_level: None`
   - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

4. **Act as an Objective QA Rust developer and verify HttpConfig::default() includes all required fields and compiles successfully.**

### Phase 2: Fix Production Quality Violations

5. **Remove unwrap() from BufferGuard methods**
   - File: `packages/client/src/http/compression.rs`
   - Lines: 167, 172
   - Replace `unwrap()` with proper error handling
   - Return `Result<&mut Vec<u8>, HttpError>` from as_mut()
   - Return `Result<&Vec<u8>, HttpError>` from as_ref()
   - Update all call sites to handle Results
   - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

6. **Act as an Objective QA Rust developer and verify BufferGuard methods use proper error handling without unwrap() or expect() in src code.**

7. **Replace assert!() with proper validation in builder methods**
   - File: `packages/client/src/client/configuration.rs`
   - Lines: 89, 102, 115
   - Replace `assert!()` with `if` checks returning `Result<Self, HttpError>`
   - Update method signatures to return Results
   - Update call sites to handle validation errors
   - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

8. **Act as an Objective QA Rust developer and verify builder methods use proper error handling without panics and return appropriate Result types.**

### Phase 3: Implement Correct Protocol-Layer Compression

9. **Add ClientStats parameter to H3RequestProcessor methods**
   - File: `packages/client/src/protocols/h3/strategy/processing.rs`
   - Method: `process_request()` around line 42
   - Add `stats: Arc<ClientStats>` parameter
   - Store stats in struct for use in prepare_request_body
   - Update all call sites in protocol strategy
   - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

10. **Act as an Objective QA Rust developer and verify ClientStats are properly passed to H3RequestProcessor and stored for metrics recording.**

11. **Implement request compression in H3RequestProcessor.prepare_request_body()**
    - File: `packages/client/src/protocols/h3/strategy/processing.rs`
    - Method: `prepare_request_body()` around line 124
    - After serializing body to bytes, check if compression should be applied
    - Use `should_compress_content_type()` with detected content type  
    - Apply compression using `compress_bytes_with_metrics()` with stored stats
    - Only compress if result is smaller (worthwhile)
    - Return compressed bytes to be passed to `h3_conn.send_body(compressed_bytes)`
    - Store compression algorithm in H3RequestProcessor for header management
    - **Critical**: Do NOT modify user's request object - compression is transparent
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

12. **Act as an Objective QA Rust developer and verify request compression is applied at protocol layer with proper content-type detection, metrics recording, and header management.**

13. **Fix H3Config to HttpConfig type conversion**
    - File: `packages/client/src/protocols/strategy/mod.rs` (or wherever H3Config is defined)
    - Add `to_http_config()` method to H3Config
    - Extract relevant fields for compression configuration
    - OR modify `needs_decompression()` to accept H3Config directly
    - Fix compilation errors in adapter.rs:264 and processing.rs:477
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

14. **Act as an Objective QA Rust developer and verify H3Config properly converts to HttpConfig for compression functions and all type mismatches are resolved.**

15. **Pass ClientStats to decompression functions in protocol layer**
    - Files: 
      - `packages/client/src/protocols/h3/adapter.rs` line 277
      - `packages/client/src/protocols/h3/strategy/processing.rs` line 506
    - Change `None` to actual ClientStats reference
    - Modify functions to accept and thread ClientStats through
    - Ensure decompression metrics are properly recorded
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

16. **Act as an Objective QA Rust developer and verify decompression operations record metrics properly and ClientStats are threaded through the protocol layer correctly.**

### Phase 4: Fix Missing JSON Compression Metrics

17. **Fix JSON compression metrics recording**
    - File: `packages/client/src/client/core.rs`
    - Line: 425 (approximately, in JSON body compression)
    - Change `compress_bytes()` to `compress_bytes_with_metrics()` with stats
    - Ensure consistency with other compression calls
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

18. **Act as an Objective QA Rust developer and verify all compression operations consistently use metrics recording functions.**

### Phase 5: Integration Testing and Verification

19. **Verify compilation succeeds**
    - Run `cargo check --workspace` 
    - Fix any remaining compilation errors
    - Ensure no unwrap/expect in src code
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

20. **Act as an Objective QA Rust developer and verify the codebase compiles successfully without errors and adheres to the no-unwrap/no-expect requirements.**

21. **Create integration test for transparent compression**
    - File: `tests/compression_integration.rs`
    - Test: Send JSON request, verify compression happens transparently
    - Test: Receive compressed response, verify decompression is transparent
    - Test: Verify original request/response objects are unchanged
    - Test: Verify metrics are recorded correctly
    - Use expect() in tests (allowed), not unwrap()
    - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

22. **Act as an Objective QA Rust developer and verify integration tests properly validate transparent compression behavior and metrics recording without mocking operations.**

## Architecture Notes

**Milestone 1: Clean Architecture**
- HttpClient.execute() becomes a pure router - no request modification
- Protocol layer handles all compression transparently 
- Clear separation of concerns

**Milestone 2: Production Quality**
- All unwrap/expect removed from src code
- Proper error handling with Result types
- No panic-causing validation

**Milestone 3: Correct Integration**
- Compression happens during wire serialization
- Decompression happens during wire deserialization
- Full metrics flow to protocol layer
- Configuration properly controls behavior

**Final State:**
- User API unchanged - requests/responses never modified
- Compression completely transparent  
- Full observability through metrics
- Production-grade error handling
- Zero-allocation performance maintained

## Wire-Level Implementation Details

### Request Compression in H3RequestProcessor

**Current Flow** (packages/client/src/protocols/h3/strategy/processing.rs:124):
```rust
pub(crate) fn prepare_request_body(&self, body_data: RequestBody, config: &H3Config, body_tx: &AsyncStreamSender<HttpBodyChunk>) -> Result<Vec<u8>, HttpError> {
    match body_data {
        RequestBody::Bytes(bytes) => Ok(bytes.to_vec()), // <-- Add compression here
        RequestBody::Text(text) => Ok(text.into_bytes()), // <-- Add compression here  
        RequestBody::Json(json) => { /* serialize then compress */ }
        // ... other types
    }
}
```

**Enhanced Flow** (with transparent compression):
```rust
pub(crate) fn prepare_request_body(&self, body_data: RequestBody, config: &H3Config, body_tx: &AsyncStreamSender<HttpBodyChunk>) -> Result<Vec<u8>, HttpError> {
    // 1. Serialize body to bytes (existing logic)
    let body_bytes = match body_data { /* existing serialization */ };
    
    // 2. Apply compression if configured and worthwhile
    if config.request_compression && should_compress_content_type(detected_content_type, config) {
        let compressed = compress_bytes_with_metrics(&body_bytes, algorithm, level, stats)?;
        if compressed.len() < body_bytes.len() {
            self.request_compression_algorithm = Some(algorithm); // Store for headers
            return Ok(compressed);
        }
    }
    
    // 3. Return original bytes if compression not beneficial
    Ok(body_bytes)
}
```

### Response Decompression in H3RequestProcessor  

**Current Flow** (packages/client/src/protocols/h3/strategy/processing.rs:506):
```rust
fn process_response_data(&self, /* ... */) {
    match h3_conn.recv_body(quic_conn, stream_id, &mut body_buf) {
        Ok(len) => {
            let raw_data = &body_buf[..len];
            // Apply decompression if needed - ALREADY IMPLEMENTED!
            let processed_data = if let Some(algorithm) = self.compression_algorithm {
                decompress_bytes_with_metrics(raw_data, algorithm, None)?  // <-- Fix metrics
            } else {
                Bytes::from(raw_data.to_vec())
            };
        }
    }
}
```

### H3 Header Management

**Request Headers** (Content-Encoding addition):
- Location: `H3RequestProcessor.process_request()` around line 62
- Add Content-Encoding header to H3 headers if compression was applied
- Use stored `self.request_compression_algorithm` to determine encoding name

**Response Headers** (Content-Encoding detection):  
- Location: `H3RequestProcessor.process_response_headers()` around line 476
- Already implemented: detects compression from Content-Encoding header
- Sets `self.compression_algorithm` for use in `process_response_data()`

### Quiche Integration Points

**Send Path**: `h3_conn.send_body(quic_conn, stream_id, compressed_bytes, fin)`
- Compressed bytes from `prepare_request_body()` are passed directly
- No modification to user's request object

**Receive Path**: `h3_conn.recv_body(quic_conn, stream_id, &mut buffer)` 
- Compressed bytes received from wire
- Decompression applied before returning to user

## Key Principles

1. **Transparency**: User never knows compression is happening
2. **Protocol-Level**: Compression occurs during wire transmission, not object modification
3. **Metrics**: Full observability of compression operations
4. **Production Quality**: No unwrap/expect, proper error handling
5. **Configuration**: User controls algorithms, levels, and behavior
6. **Zero-Allocation**: Maintain performance characteristics
7. **Surgical Changes**: Minimal modifications to achieve goals
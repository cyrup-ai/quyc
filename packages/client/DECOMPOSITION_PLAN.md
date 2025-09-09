# File Decomposition Plan

## Overview
27 files over 300 lines need decomposition into logical submodules for better maintainability.

## Priority Order (by size, largest first):

### PRIORITY 1: Massive Files (800+ lines)
1. **src/tls/builder/certificate.rs (1057 lines)** → Multiple builders
   - CertificateBuilder → certificate_builder.rs
   - CertificateValidator → certificate_validator.rs  
   - CertificateGenerator → certificate_generator.rs
   - Utility functions → certificate_utils.rs

2. **src/protocols/h3/strategy.rs (989 lines)** → Strategy components
   - H3Strategy core → strategy_core.rs
   - Connection handling → strategy_connection.rs
   - Stream management → strategy_streams.rs

3. **src/http/response.rs (900 lines)** → Response components
   - HttpResponse core → response_core.rs
   - HttpBodyChunk → body_chunk.rs
   - Helper structs (HttpStatus, HttpHeader) → response_types.rs
   - Utility functions → response_utils.rs

4. **src/tls/builder/authority.rs (876 lines)** → Authority components
   - Core authority logic → authority_core.rs
   - Validation → authority_validation.rs
   - Certificate chain → authority_chain.rs

5. **src/tls/certificate/parser.rs (827 lines)** → Parser components
   - Core parsing → parser_core.rs
   - ASN.1 parsing → asn1_parser.rs
   - Extensions → certificate_extensions.rs

6. **src/http/request.rs (804 lines)** → Request components
   - HttpRequest core → request_core.rs
   - Builder pattern → request_builder.rs
   - Header management → request_headers.rs

### PRIORITY 2: Large Files (600-799 lines)
7. **src/protocols/quiche/streaming.rs (701 lines)** → Streaming components
8. **src/wasm/request/types.rs (626 lines)** → WASM request types
9. **src/protocols/wire.rs (623 lines)** → Wire protocol components
10. **src/tls/tls_manager.rs (609 lines)** → TLS management components
11. **src/wasm/response.rs (592 lines)** → WASM response components

### PRIORITY 3: Medium Files (400-599 lines)
12-18. Files from 541 to 399 lines

### PRIORITY 4: Above Threshold (300-399 lines)
19-27. Files from 377 to 306 lines

## Decomposition Strategy:

1. **Extract by Logical Responsibility**: Separate structs, implementations, and utilities
2. **Maintain Module Hierarchy**: Keep existing module structure, add submodules
3. **Preserve Public API**: Ensure no breaking changes to public interfaces
4. **Update mod.rs Files**: Add proper re-exports for decomposed components
5. **Test Compilation**: Verify each decomposition maintains compilation

## Implementation Steps:
1. Start with largest files first
2. Create submodule directories where needed
3. Extract components to separate files
4. Update parent mod.rs with re-exports
5. Run cargo check after each decomposition
6. Move to next file only after successful compilation
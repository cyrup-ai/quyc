//! JSONPath module tests
//!
//! Tests mirroring the source module structure in src/json_path/

pub mod ast;
pub mod buffer;
pub mod compiler;
pub mod deserializer;
pub mod error;
pub mod expression;
pub mod filter;
pub mod filter_parser;
pub mod functions;
pub mod parser;
pub mod selector_parser;
pub mod state_machine;
pub mod tokenizer;
pub mod tokens;

// Moved from root level
pub mod buffer_tests;
pub mod deserializer_tests;
pub mod error_tests;
pub mod parser_tests;
pub mod state_machine_tests;
pub mod streaming_tests;

// RFC 9535 Compliance Tests
pub mod rfc9535;

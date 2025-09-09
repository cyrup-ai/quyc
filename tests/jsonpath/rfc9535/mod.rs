//! RFC 9535 JSONPath Standard Compliance Tests
//!
//! This module contains comprehensive tests for RFC 9535 compliance,
//! ensuring the JSONPath implementation meets all standard requirements.

// Core compliance tests
pub mod abnf_compliance;
pub mod abnf_grammar_tests;
pub mod core_requirements_tests;
pub mod core_syntax;
pub mod iana_compliance;
pub mod iregexp_compliance;

// Feature tests
pub mod advanced_features_tests;
pub mod array_slice_algorithms;
pub mod array_slice_selectors;
pub mod bracket_escape_sequences;
pub mod current_node_identifier_tests;
pub mod filter_precedence;
pub mod filter_precedence_comprehensive_tests;
pub mod filter_selectors;
pub mod segment_traversal;
pub mod segments;
pub mod selectors;
pub mod shorthand_syntax_validation;
pub mod singular_queries;

// Function system tests
pub mod function_composition_tests;
pub mod function_count;
pub mod function_extensions;
pub mod function_length;
pub mod function_system_tests;
pub mod function_type_system;
pub mod function_well_typedness;

// Comparison and compatibility tests
pub mod comparison_edge_cases;
pub mod json_pointer_compatibility_tests;
pub mod xpath_equivalence;

// Error handling and security
pub mod dos_protection_tests;
pub mod error_handling;
pub mod security_compliance;
pub mod security_error_tests;

// Data types and semantics
pub mod null_semantics;
pub mod string_literals;
pub mod unicode_compliance;

// Path normalization and validation
pub mod normalized_paths;
pub mod wellformedness_validity_comprehensive;

// Performance and streaming
pub mod performance_compliance;
pub mod streaming_behavior;

// Integration and boundary tests
pub mod ijson_boundary_tests;
pub mod integration;

// Examples and documentation
pub mod examples;
pub mod official_examples;

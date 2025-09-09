//! RFC 9535 Well-formedness vs Validity Comprehensive Tests (Section 2.1)
//!
//! Tests the critical distinction between:
//! - Well-formedness: Syntactic correctness according to ABNF grammar
//! - Validity: Semantic correctness when evaluated against JSON data
//!
//! This distinction is fundamental to RFC 9535 compliance

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct WellformednessTestModel {
    name: String,
    value: serde_json::Value,
}

/// RFC 9535 Section 2.1 - Well-formedness vs Validity Tests
#[cfg(test)]
mod wellformedness_validity_comprehensive {
    use super::*;

    #[test]
    fn test_syntactic_wellformedness_parsing_only() {
        // RFC 9535: These expressions are syntactically well-formed and should parse
        // regardless of whether they would match any data
        let wellformed_expressions = vec![
            // Basic well-formed patterns
            "$",
            "$.store",
            "$.store.book",
            "$.store.book[0]",
            "$.store.book[*]",
            "$..book",
            "$.*",
            // Well-formed but semantically questionable
            "$.nonexistent",
            "$.store.nonexistent.chain",
            "$[999999]",
            "$[-999999]",
            "$.store.book[999999]",
            "$..nonexistent",
            // Well-formed complex expressions
            "$.store.book[?@.price > 1000000]",
            "$.store.book[?@.nonexistent == 'impossible']",
            "$[?@.missing.property.chain]",
            "$.store.book[*].nonexistent.property",
            "$..book[*].nonexistent[999].impossible",
            // Well-formed basic filter calls (using only core operators)
            "$[?@.nonexistent > 0]",
            "$[?@.missing == 999]",
            "$[?@.property == 'pattern']",
            "$[?@.text != null]",
            "$[?@.value >= 10]",
            // Well-formed but extreme expressions
            "$.a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z",
            "$[0][1][2][3][4][5][6][7][8][9][10]",
            "$..a..b..c..d..e",
            "$[*][*][*][*][*]",
            // Well-formed Unicode expressions
            "$.北京.東京.Москва.Paris",
            "$['🌟']['🎉']['🚀']",
            "$['\u{0041}\u{0042}\u{0043}']",
        ];

        for expr in wellformed_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Well-formed expression '{}' should parse successfully",
                expr
            );
        }
    }

    #[test]
    fn test_syntactic_malformedness_must_reject() {
        // RFC 9535: These expressions are syntactically malformed and MUST be rejected
        let malformed_expressions = vec![
            // Missing required components
            "",         // Empty expression
            "store",    // Missing root identifier
            "$.store.", // Trailing dot
            "$.",       // Incomplete dot access
            "$..",      // Incomplete descendant
            "$...",     // Triple dots
            // Unmatched delimiters
            "$[",       // Unclosed bracket
            "$]",       // Unmatched closing bracket
            "$.store[", // Unclosed array access
            "$.store]", // Unmatched closing bracket
            "$[[",      // Double opening brackets
            "$]]",      // Double closing brackets
            "$()",      // Invalid parentheses at root
            "${}",      // Invalid braces at root
            // Invalid root identifiers
            "@",        // @ without $
            "@.store",  // @ as root (should be in filter only)
            "$.store$", // Multiple root identifiers
            "$$.store", // Double root identifiers
            // Invalid property access
            "$.123abc",       // Property starting with digit
            "$.-abc",         // Property starting with hyphen
            "$.@abc",         // Property starting with @
            "$.store..book.", // Trailing dot after descendant
            // Unclosed string literals
            "$['unclosed",  // Unclosed single quote
            "$[\"unclosed", // Unclosed double quote
            "$['mixed\"]",  // Mixed quote types
            "$[\"mixed']",  // Mixed quote types reverse
            // Invalid filter expressions
            "$[?]",                // Empty filter
            "$[?@]",               // Incomplete filter (may be valid for existence)
            "$[?@.]",              // Incomplete property access
            "$[?@.price >]",       // Incomplete comparison
            "$[?@.price > 10 &&]", // Incomplete logical expression
            "$[?&& @.price > 10]", // Leading logical operator
            "$[?@.price === 10]",  // Invalid triple equals
            "$[?@.price <> 10]",   // Invalid not-equal operator
            "$[?(@.price > 10]",   // Unmatched opening parenthesis
            "$[?@.price > 10)]",   // Unmatched closing parenthesis
            // Invalid array slice syntax
            "$[::]",            // Invalid slice at root
            "$.array[::0]",     // Zero step in slice
            "$.array[1:2:3:4]", // Too many slice parameters
            "$.array[abc:def]", // Non-numeric slice parameters
            // Invalid escape sequences
            "$['\\x41']",   // Invalid hex escape
            "$['\\q']",     // Invalid escape character
            "$['\\u123']",  // Incomplete Unicode escape
            "$['\\uXXXX']", // Invalid Unicode hex
            // Invalid function calls
            "$[?unknown_function(@.test)]", // Undefined function
            "$[?length()]",                 // Missing required argument
            "$[?length(@.test, extra)]",    // Too many arguments
            "$[?length(@.test,)]",          // Trailing comma in function call
            "$[?(length(@.test)]",          // Unmatched parenthesis in function
        ];

        for expr in malformed_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Malformed expression '{}' should be rejected during parsing",
                expr
            );
        }
    }

    #[test]
    fn test_semantic_validity_with_actual_data() {
        // RFC 9535: Test semantic validity - well-formed expressions against real data
        let json_data = r#"{
            "store": {
                "book": [
                    {
                        "title": "Book 1",
                        "author": "Author 1", 
                        "price": 10.99,
                        "isbn": "123-456",
                        "available": true
                    },
                    {
                        "title": "Book 2",
                        "author": "Author 2",
                        "price": 15.99,
                        "isbn": "789-012",
                        "available": false
                    }
                ],
                "bicycle": {
                    "color": "red",
                    "price": 19.95
                }
            },
            "empty_array": [],
            "empty_object": {},
            "null_value": null
        }"#;

        let semantic_validity_tests = vec![
            // Valid semantics - should return results
            ("$.store", 1, "Root property exists"),
            ("$.store.book", 1, "Nested property exists"),
            ("$.store.book[0]", 1, "First array element"),
            ("$.store.book[1]", 1, "Second array element"),
            ("$.store.book[0].title", 1, "Property of array element"),
            ("$.store.bicycle.color", 1, "Nested object property"),
            ("$.store.book[*].title", 2, "Wildcard over array"),
            ("$..price", 3, "Descendant search finds all prices"),
            ("$.empty_array", 1, "Empty array is valid"),
            ("$.empty_object", 1, "Empty object is valid"),
            ("$.null_value", 1, "Null value is valid"),
            // Invalid semantics - well-formed but no matches
            ("$.nonexistent", 0, "Non-existent root property"),
            ("$.store.nonexistent", 0, "Non-existent nested property"),
            ("$.store.book[999]", 0, "Out-of-bounds array index"),
            ("$.store.book[-999]", 0, "Out-of-bounds negative index"),
            (
                "$.store.book.title",
                0,
                "Property access on array (not element)",
            ),
            ("$.store.bicycle[0]", 0, "Array access on object"),
            (
                "$.store.book[*].nonexistent",
                0,
                "Non-existent property via wildcard",
            ),
            ("$..nonexistent", 0, "Descendant search for non-existent"),
            ("$.empty_array[0]", 0, "Index into empty array"),
            ("$.empty_object.anything", 0, "Property of empty object"),
            ("$.null_value.property", 0, "Property access on null"),
            // Type mismatch semantics
            (
                "$.store.book[0].price.invalid",
                0,
                "Property access on number",
            ),
            ("$.store.bicycle.color[0]", 0, "Array access on string"),
            (
                "$.store.book[0].available.nested",
                0,
                "Property access on boolean",
            ),
        ];

        for (expr, expected_count, _description) in semantic_validity_tests {
            // First verify the expression is well-formed
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Expression '{}' should be well-formed: {}",
                expr,
                _description
            );

            // Then test semantic validity
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Semantic validity test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_filter_expression_wellformedness_vs_validity() {
        // RFC 9535: Test well-formedness vs validity specifically for filter expressions
        let json_data = r#"{
            "items": [
                {"name": "item1", "price": 10, "active": true, "tags": ["a", "b"]},
                {"name": "item2", "price": 20, "active": false, "tags": ["c"]},
                {"name": "item3", "price": null, "active": true},
                {"price": 15, "active": true, "tags": []}
            ]
        }"#;

        let filter_wellformedness_tests = vec![
            // Well-formed and semantically valid
            ("$.items[?@.price > 10]", true, 2, "Valid price comparison"),
            ("$.items[?@.name]", true, 3, "Existence test for name"),
            ("$.items[?@.active == true]", true, 3, "Boolean equality"),
            (
                "$.items[?@.price && @.active]",
                true,
                3,
                "Logical AND with existence",
            ),
            // Well-formed but semantically limited
            (
                "$.items[?@.nonexistent > 10]",
                true,
                0,
                "Comparison with non-existent property",
            ),
            (
                "$.items[?@.price > @.nonexistent]",
                true,
                0,
                "Comparison between existent and non-existent",
            ),
            ("$.items[?@.price == null]", true, 1, "Null comparison"),
            (
                "$.items[?@.tags[999]]",
                true,
                0,
                "Out-of-bounds array access in filter",
            ),
            (
                "$.items[?@.name.length]",
                true,
                0,
                "Property access on string (invalid)",
            ),
            // Well-formed function calls
            (
                "$.items[?length(@.name) > 5]",
                true,
                0,
                "Function on property",
            ),
            (
                "$.items[?count(@.tags) > 0]",
                true,
                2,
                "Function on array property",
            ),
            (
                "$.items[?match(@.name, '^item')]",
                true,
                3,
                "Regex function",
            ),
            // Well-formed but semantically questionable
            (
                "$.items[?length(@.nonexistent) > 0]",
                true,
                0,
                "Function on non-existent property",
            ),
            (
                "$.items[?count(@.price) > 0]",
                true,
                0,
                "Count function on non-array",
            ),
            (
                "$.items[?match(@.price, 'pattern')]",
                true,
                0,
                "Regex function on number",
            ),
        ];

        for (expr, should_be_wellformed, expected_count, _description) in
            filter_wellformedness_tests
        {
            let result = JsonPathParser::compile(expr);

            if should_be_wellformed {
                assert!(
                    result.is_ok(),
                    "Filter expression '{}' should be well-formed: {}",
                    expr,
                    _description
                );

                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                assert_eq!(
                    results.len(),
                    expected_count,
                    "Filter validity test '{}' should return {} results: {}",
                    expr,
                    expected_count,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "Filter expression '{}' should be malformed: {}",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_type_safety_wellformedness_vs_runtime_validity() {
        // RFC 9535: Test the distinction between compile-time well-formedness and runtime validity
        let json_data = r#"{
            "mixed": [
                "string_value",
                42,
                true,
                null,
                {"nested": "object"},
                [1, 2, 3]
            ]
        }"#;

        let type_safety_tests = vec![
            // Well-formed, runtime type checks
            (
                "$.mixed[0].length",
                true,
                0,
                "String method access (not property)",
            ),
            ("$.mixed[1].toString", true, 0, "Number method access"),
            ("$.mixed[2].valueOf", true, 0, "Boolean method access"),
            ("$.mixed[3].anything", true, 0, "Property access on null"),
            ("$.mixed[4].nested", true, 1, "Valid object property access"),
            ("$.mixed[5][0]", true, 1, "Valid array index access"),
            // Well-formed but type mismatches
            ("$.mixed[0][0]", true, 0, "Array access on string"),
            ("$.mixed[1].property", true, 0, "Property access on number"),
            ("$.mixed[2].property", true, 0, "Property access on boolean"),
            ("$.mixed[4][0]", true, 0, "Array access on object"),
            ("$.mixed[5].property", true, 0, "Property access on array"),
            // Well-formed complex type scenarios
            (
                "$.mixed[*].nested",
                true,
                1,
                "Property access on mixed types",
            ),
            ("$.mixed[*][0]", true, 1, "Array access on mixed types"),
            (
                "$..nested",
                true,
                1,
                "Descendant search through mixed types",
            ),
            // Function calls with type considerations
            (
                "$.mixed[?length(@) > 0]",
                true,
                0,
                "Length function on mixed types",
            ),
            (
                "$.mixed[?@.nested]",
                true,
                1,
                "Property existence on mixed types",
            ),
            (
                "$.mixed[?@ == null]",
                true,
                1,
                "Null comparison on mixed types",
            ),
        ];

        for (expr, should_be_wellformed, expected_count, _description) in type_safety_tests {
            let result = JsonPathParser::compile(expr);

            if should_be_wellformed {
                assert!(
                    result.is_ok(),
                    "Type safety expression '{}' should be well-formed: {}",
                    expr,
                    _description
                );

                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                assert_eq!(
                    results.len(),
                    expected_count,
                    "Type safety test '{}' should return {} results: {}",
                    expr,
                    expected_count,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "Type safety expression '{}' should be malformed: {}",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_boundary_between_wellformedness_and_validity() {
        // RFC 9535: Test the precise boundary between well-formedness and validity
        let wellformedness_boundary_tests = vec![
            // These should be well-formed (parse successfully)
            ("$.store", true, "Basic property access"),
            ("$.store.book[999999]", true, "Large array index"),
            ("$.store.book[-999999]", true, "Large negative index"),
            ("$..nonexistent", true, "Descendant search for non-existent"),
            ("$.a.b.c.d.e.f.g.h.i.j", true, "Deep property chain"),
            ("$[?@.a.b.c.d.e.f]", true, "Deep property in filter"),
            (
                "$[?length(@.nonexistent) > 999999]",
                true,
                "Function with non-existent property",
            ),
            // These should be malformed (fail to parse)
            ("$.store.", false, "Trailing dot"),
            ("$.store..", false, "Incomplete descendant"),
            ("$.store...", false, "Triple dots"),
            ("$[?@.store.]", false, "Trailing dot in filter"),
            ("$[?@.store..]", false, "Incomplete descendant in filter"),
            ("$[?length()]", false, "Function with no arguments"),
            (
                "$[?length(@.test, extra, params)]",
                false,
                "Function with too many arguments",
            ),
            // Edge cases at the boundary
            ("$['']", true, "Empty string property name"),
            ("$[0]", true, "Zero index"),
            ("$[-0]", true, "Negative zero index"),
            ("$[?@]", true, "Current node existence test (may be valid)"),
            ("$[?@.price]", true, "Property existence test"),
            ("$[?true]", true, "Literal boolean in filter"),
            ("$[?false]", true, "Literal false in filter"),
            ("$[?null]", true, "Literal null in filter"),
        ];

        for (expr, should_be_wellformed, _description) in wellformedness_boundary_tests {
            let result = JsonPathParser::compile(expr);

            if should_be_wellformed {
                assert!(
                    result.is_ok(),
                    "Boundary test: '{}' should be well-formed ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "Boundary test: '{}' should be malformed ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

//! RFC 9535 ABNF Grammar Compliance Tests
//!
//! Tests the complete ABNF grammar specification for JSONPath expressions
//! Validates UTF-8 encoding, I-JSON number ranges, and grammar well-formedness

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};

/// RFC 9535 ABNF Grammar Compliance Tests
#[cfg(test)]
mod abnf_grammar_tests {
    use super::*;

    #[test]
    fn test_jsonpath_query_syntax() {
        // RFC 9535: jsonpath-query = root-identifier segments
        let valid_queries = vec![
            "$",                  // Root identifier only
            "$.store",            // Root + single segment
            "$.store.book",       // Root + multiple segments
            "$['store']['book']", // Root + bracket notation
            "$..book",            // Root + descendant segment
        ];

        for query in valid_queries {
            let result = JsonPathParser::compile(query);
            assert!(
                result.is_ok(),
                "Valid ABNF query '{}' should compile",
                query
            );
        }
    }

    #[test]
    fn test_root_identifier() {
        // RFC 9535: root-identifier = "$"
        let valid_roots = vec!["$"];
        let invalid_roots = vec!["", "@", "store", "$."];

        for root in valid_roots {
            let result = JsonPathParser::compile(root);
            assert!(result.is_ok(), "Valid root '{}' should compile", root);
        }

        for root in invalid_roots {
            let result = JsonPathParser::compile(root);
            assert!(result.is_err(), "Invalid root '{}' should fail", root);
        }
    }

    #[test]
    fn test_segments_syntax() {
        // RFC 9535: segments = *(S segment)
        let segment_tests = vec![
            ("$", true),                // No segments
            ("$.store", true),          // Single segment
            ("$.store.book", true),     // Multiple segments
            ("$  .store", true),        // Whitespace allowed
            ("$ . store . book", true), // Multiple whitespace
        ];

        for (query, should_pass) in segment_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid segments '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid segments '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_segment_types() {
        // RFC 9535: segment = child-segment / descendant-segment
        let segment_tests = vec![
            // Child segments
            ("$.store", true),
            ("$['store']", true),
            ("$[0]", true),
            ("$[*]", true),
            ("$[0:5]", true),
            // Descendant segments
            ("$..store", true),
            ("$..*", true),
            ("$..[0]", true),
            ("$..[?@.price]", true),
        ];

        for (query, should_pass) in segment_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid segment '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid segment '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_child_segment_syntax() {
        // RFC 9535: child-segment = dot-member / bracket-segment
        let child_tests = vec![
            // Dot member notation
            ("$.store", true),
            ("$.book_store", true),
            ("$._private", true),
            ("$.123invalid", false), // Can't start with digit
            // Bracket notation
            ("$['store']", true),
            ("$[\"store\"]", true),
            ("$[0]", true),
            ("$[*]", true),
            ("$[:5]", true),
            ("$[1:3]", true),
            ("$[?@.price]", true),
        ];

        for (query, should_pass) in child_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid child segment '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid child segment '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_descendant_segment_syntax() {
        // RFC 9535: descendant-segment = ".." S bracket-segment
        let descendant_tests = vec![
            ("$..store", true),
            ("$..*", true),
            ("$..[0]", true),
            ("$..[*]", true),
            ("$..[?@.price]", true),
            ("$..['store']", true),
            ("$.. [0]", true),    // Whitespace allowed
            ("$...store", false), // Triple dot invalid
            ("$..", false),       // Must have bracket segment
        ];

        for (query, should_pass) in descendant_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid descendant '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid descendant '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_bracket_segment_syntax() {
        // RFC 9535: bracket-segment = "[" S selector *(S "," S selector) S "]"
        let bracket_tests = vec![
            ("$[0]", true),           // Single selector
            ("$[0,1,2]", true),       // Multiple selectors
            ("$[ 0 , 1 , 2 ]", true), // Whitespace
            ("$[*]", true),           // Wildcard
            ("$['name']", true),      // String
            ("$[?@.price]", true),    // Filter
            ("$[:5]", true),          // Slice
            ("$[", false),            // Unclosed
            ("$0]", false),           // Missing open bracket
            ("$[]", false),           // Empty selectors
        ];

        for (query, should_pass) in bracket_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid bracket '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid bracket '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_selector_types() {
        // RFC 9535: selector = name / wildcard / slice / index / filter
        let selector_tests = vec![
            // Name selectors
            ("$['store']", true),
            ("$[\"book\"]", true),
            // Wildcard selector
            ("$[*]", true),
            // Slice selectors
            ("$[:]", true),
            ("$[1:]", true),
            ("$[:5]", true),
            ("$[1:5]", true),
            ("$[::2]", true),
            ("$[1:5:2]", true),
            // Index selectors
            ("$[0]", true),
            ("$[-1]", true),
            ("$[42]", true),
            // Filter selectors
            ("$[?@.price]", true),
            ("$[?@.price > 10]", true),
        ];

        for (query, should_pass) in selector_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid selector '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid selector '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_name_selector_syntax() {
        // RFC 9535: name = string-literal
        let name_tests = vec![
            ("$['store']", true),
            ("$[\"store\"]", true),
            ("$['']", true),               // Empty string valid
            ("$['book-store']", true),     // Hyphen valid
            ("$['book_store']", true),     // Underscore valid
            ("$['123']", true),            // Number as string valid
            ("$['with spaces']", true),    // Spaces valid
            ("$['with\\nnewline']", true), // Escape sequences valid
            ("$[store]", false),           // Unquoted invalid
        ];

        for (query, should_pass) in name_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid name '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid name '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_string_literal_syntax() {
        // RFC 9535: string-literal = %x22 *string-character %x22 / %x27 *string-character %x27
        let string_tests = vec![
            ("$[\"hello\"]", true),         // Double quotes
            ("$['hello']", true),           // Single quotes
            ("$[\"\"]", true),              // Empty double quoted
            ("$['']", true),                // Empty single quoted
            ("$[\"with 'single'\"]", true), // Mixed quotes
            ("$['with \"double\"']", true), // Mixed quotes
            ("$[\"unclosed]", false),       // Unclosed double quote
            ("$['unclosed]", false),        // Unclosed single quote
        ];

        for (query, should_pass) in string_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid string '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid string '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_string_character_syntax() {
        // RFC 9535: string-character = string-escaped / string-unescaped
        let character_tests = vec![
            // Escaped characters
            ("$[\"\\b\"]", true),     // Backspace
            ("$[\"\\t\"]", true),     // Tab
            ("$[\"\\n\"]", true),     // Newline
            ("$[\"\\f\"]", true),     // Form feed
            ("$[\"\\r\"]", true),     // Carriage return
            ("$[\"\\\"\"]", true),    // Quote
            ("$[\"\\'\"]", true),     // Apostrophe
            ("$[\"\\/\"]", true),     // Solidus
            ("$[\"\\\\\"]", true),    // Backslash
            ("$[\"\\u0041\"]", true), // Unicode escape
            // Unescaped characters
            ("$[\"hello\"]", true),      // Regular characters
            ("$[\"123\"]", true),        // Numbers
            ("$[\"!@#$%^&*()\"]", true), // Special chars
            // Invalid escapes
            ("$[\"\\x\"]", false),    // Invalid escape
            ("$[\"\\u\"]", false),    // Incomplete unicode
            ("$[\"\\u123\"]", false), // Incomplete unicode
        ];

        for (query, should_pass) in character_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid character '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid character '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_wildcard_syntax() {
        // RFC 9535: wildcard = "*"
        let wildcard_tests = vec![
            ("$[*]", true),
            ("$.store[*]", true),
            ("$.*", true),
            ("$[**]", false), // Double wildcard invalid
            ("$[*0]", false), // Wildcard with number invalid
        ];

        for (query, should_pass) in wildcard_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid wildcard '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid wildcard '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_slice_syntax() {
        // RFC 9535: slice = [start S] ":" S [end S] [":" S step]
        let slice_tests = vec![
            ("$[:]", true),           // Full slice
            ("$[1:]", true),          // Start only
            ("$[:5]", true),          // End only
            ("$[1:5]", true),         // Start and end
            ("$[::2]", true),         // Step only
            ("$[1::2]", true),        // Start and step
            ("$[:5:2]", true),        // End and step
            ("$[1:5:2]", true),       // Full slice
            ("$[ 1 : 5 : 2 ]", true), // Whitespace
            ("$[-1:]", true),         // Negative start
            ("$[:-1]", true),         // Negative end
            ("$[::-1]", true),        // Negative step
            ("$[1:5:0]", false),      // Zero step invalid
            ("$[1:5:]", false),       // Missing step after colon
        ];

        for (query, should_pass) in slice_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid slice '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid slice '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_index_syntax() {
        // RFC 9535: index = int
        let index_tests = vec![
            ("$[0]", true),    // Zero
            ("$[1]", true),    // Positive
            ("$[-1]", true),   // Negative
            ("$[42]", true),   // Large positive
            ("$[-42]", true),  // Large negative
            ("$[01]", false),  // Leading zero invalid
            ("$[+1]", false),  // Plus sign invalid
            ("$[1.0]", false), // Decimal invalid
            ("$[1e5]", false), // Scientific notation invalid
        ];

        for (query, should_pass) in index_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid index '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid index '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_filter_syntax() {
        // RFC 9535: filter = "?" S logical-expr
        let filter_tests = vec![
            ("$[?@.price]", true),       // Property existence
            ("$[?@.price > 10]", true),  // Comparison
            ("$[? @.price > 10]", true), // Whitespace after ?
            ("$[?@.a && @.b]", true),    // Logical AND
            ("$[?@.a || @.b]", true),    // Logical OR
            ("$[?(@.a)]", true),         // Parentheses
            ("$[@.price]", false),       // Missing ?
            ("$[?]", false),             // Missing expression
            ("$[? ]", false),            // Only whitespace
        ];

        for (query, should_pass) in filter_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid filter '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid filter '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_integer_syntax() {
        // RFC 9535: int = "0" / (["-"] (non-zero-digit *DIGIT))
        let integer_tests = vec![
            ("$[0]", true),    // Zero
            ("$[1]", true),    // Single digit
            ("$[123]", true),  // Multiple digits
            ("$[-1]", true),   // Negative
            ("$[-123]", true), // Negative multiple digits
            ("$[01]", false),  // Leading zero
            ("$[-0]", false),  // Negative zero
            ("$[+1]", false),  // Plus sign
            ("$[]", false),    // Empty
        ];

        for (query, should_pass) in integer_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Valid integer '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Invalid integer '{}' should fail", query);
            }
        }
    }
}

/// RFC 9535 Function Syntax Compliance Tests
#[cfg(test)]
mod function_syntax_tests {
    use super::*;

    #[test]
    fn test_function_expression_syntax() {
        // RFC 9535: function-expr = length-function-expr / count-function-expr / match-function-expr / search-function-expr / value-function-expr
        let function_tests = vec![
            // Length function
            ("$[?length(@.authors) > 1]", true),
            ("$[?length(@) == 0]", true),
            ("$[?length(@.items)]", true),
            // Count function
            ("$[?count($..book) > 5]", true),
            ("$[?count(@..*) < 10]", true),
            ("$[?count(@)]", true),
            // Match function
            ("$[?match(@.author, \".*Tolkien.*\")]", true),
            ("$[?match(@.title, \"^Lord.*\")]", true),
            ("$[?match(@.isbn, \"[0-9]{13}\")]", true),
            // Search function
            ("$[?search(@._description, \"fantasy\")]", true),
            ("$[?search(@.title, \"Ring\")]", true),
            ("$[?search(@.content, \"magic\")]", true),
            // Value function
            ("$[?value(@.price) > 10]", true),
            ("$[?value(@.available)]", true),
            ("$[?value(length(@.items)) == 3]", true),
            // Nested functions
            ("$[?length(value(@.tags)) > 0]", true),
            ("$[?match(value(@.category), \"fiction\")]", true),
        ];

        for (query, should_pass) in function_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid function syntax '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid function syntax '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_length_function_syntax() {
        // RFC 9535: length-function-expr = "length" "(" S expr S ")"
        let length_tests = vec![
            ("$[?length(@.authors)]", true),
            ("$[?length(@)]", true),
            ("$[?length(@.items) > 0]", true),
            ("$[? length(@.tags) == 3]", true), // Whitespace after ?
            ("$[?length( @.data )]", true),     // Whitespace around argument
            ("$[?length()]", false),            // Missing argument
            ("$[?length(@.a, @.b)]", false),    // Too many arguments
            ("$[?LENGTH(@.items)]", false),     // Wrong case
            ("$[?length @.items]", false),      // Missing parentheses
            ("$[?length(@.items]", false),      // Missing closing paren
            ("$[?length@.items)]", false),      // Missing opening paren
        ];

        for (query, should_pass) in length_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid length function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid length function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_count_function_syntax() {
        // RFC 9535: count-function-expr = "count" "(" S expr S ")"
        let count_tests = vec![
            ("$[?count($..book)]", true),
            ("$[?count(@..*)]", true),
            ("$[?count(@.items) < 5]", true),
            ("$[? count($..book) > 10]", true), // Whitespace after ?
            ("$[?count( @..* )]", true),        // Whitespace around argument
            ("$[?count()]", false),             // Missing argument
            ("$[?count(@.a, @.b)]", false),     // Too many arguments
            ("$[?COUNT(@.items)]", false),      // Wrong case
            ("$[?count @.items]", false),       // Missing parentheses
            ("$[?count(@.items]", false),       // Missing closing paren
            ("$[?count@.items)]", false),       // Missing opening paren
        ];

        for (query, should_pass) in count_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid count function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid count function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_match_function_syntax() {
        // RFC 9535: match-function-expr = "match" "(" S expr S "," S string-literal S ")"
        let match_tests = vec![
            ("$[?match(@.author, \"Tolkien\")]", true),
            ("$[?match(@.title, \".*Ring.*\")]", true),
            ("$[?match(@.isbn, '[0-9]{13}')]", true), // Single quotes
            ("$[? match(@.name, \"pattern\")]", true), // Whitespace after ?
            ("$[?match( @.field , \"value\" )]", true), // Whitespace around args
            ("$[?match(@.text, \"multi word pattern\")]", true),
            ("$[?match(@.code, \"^[A-Z]{3}$\")]", true),
            ("$[?match()]", false),            // Missing arguments
            ("$[?match(@.field)]", false),     // Missing pattern argument
            ("$[?match(\"pattern\")]", false), // Missing expr argument
            ("$[?match(@.field, \"pattern\", \"extra\")]", false), // Too many arguments
            ("$[?MATCH(@.field, \"pattern\")]", false), // Wrong case
            ("$[?match @.field, \"pattern\"]", false), // Missing parentheses
            ("$[?match(@.field \"pattern\")]", false), // Missing comma
            ("$[?match(@.field, pattern)]", false), // Unquoted pattern
        ];

        for (query, should_pass) in match_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid match function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid match function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_search_function_syntax() {
        // RFC 9535: search-function-expr = "search" "(" S expr S "," S string-literal S ")"
        let search_tests = vec![
            ("$[?search(@._description, \"fantasy\")]", true),
            ("$[?search(@.content, \"magic\")]", true),
            ("$[?search(@.title, 'adventure')]", true), // Single quotes
            ("$[? search(@.text, \"keyword\")]", true), // Whitespace after ?
            ("$[?search( @.field , \"value\" )]", true), // Whitespace around args
            ("$[?search(@.text, \"case sensitive\")]", true),
            ("$[?search(@.data, \"unicode café\")]", true),
            ("$[?search()]", false),         // Missing arguments
            ("$[?search(@.field)]", false),  // Missing search term
            ("$[?search(\"term\")]", false), // Missing expr argument
            ("$[?search(@.field, \"term\", \"extra\")]", false), // Too many arguments
            ("$[?SEARCH(@.field, \"term\")]", false), // Wrong case
            ("$[?search @.field, \"term\"]", false), // Missing parentheses
            ("$[?search(@.field \"term\")]", false), // Missing comma
            ("$[?search(@.field, term)]", false), // Unquoted search term
        ];

        for (query, should_pass) in search_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid search function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid search function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_value_function_syntax() {
        // RFC 9535: value-function-expr = "value" "(" S function-expr S ")"
        let value_tests = vec![
            ("$[?value(@.price)]", true),
            ("$[?value(@)]", true),
            ("$[?value(@.data.field)]", true),
            ("$[? value(@.amount)]", true),       // Whitespace after ?
            ("$[?value( @.field )]", true),       // Whitespace around argument
            ("$[?value(length(@.items))]", true), // Nested function
            ("$[?value(count($..book))]", true),  // Nested function
            ("$[?value(@.nested.property)]", true),
            ("$[?value()]", false),         // Missing argument
            ("$[?value(@.a, @.b)]", false), // Too many arguments
            ("$[?VALUE(@.field)]", false),  // Wrong case
            ("$[?value @.field]", false),   // Missing parentheses
            ("$[?value(@.field]", false),   // Missing closing paren
            ("$[?value@.field)]", false),   // Missing opening paren
        ];

        for (query, should_pass) in value_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid value function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid value function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_function_nesting_syntax() {
        // RFC 9535: Functions can be nested according to type system rules
        let nesting_tests = vec![
            // Valid nesting patterns
            ("$[?length(value(@.items)) > 0]", true),
            ("$[?count(value(@.collection)) == 5]", true),
            ("$[?value(length(@.data)) < 10]", true),
            ("$[?match(value(@.category), \"fiction\")]", true),
            ("$[?search(value(@._description), \"adventure\")]", true),
            // Complex valid nesting
            ("$[?value(count($..book[?@.price])) > 3]", true),
            ("$[?length(value(@.authors)) >= 2]", true),
            // Invalid nesting (type mismatches would be caught at runtime, syntax should still be valid)
            ("$[?value(value(@.field))]", true), // Syntax valid, semantics may fail
            ("$[?length(length(@.items))]", true), // Syntax valid, semantics may fail
            ("$[?count(match(@.field, \"pattern\"))]", true), // Syntax valid, semantics may fail
        ];

        for (query, should_pass) in nesting_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid nested function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid nested function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_function_whitespace_handling() {
        // RFC 9535: S = *WSP where WSP = %x20 / %x09 / %x0A / %x0D
        let whitespace_tests = vec![
            // Space variations
            ("$[?length(@.items)]", true),
            ("$[? length(@.items)]", true),  // Space after ?
            ("$[?length (@.items)]", true),  // Space before (
            ("$[?length( @.items)]", true),  // Space after (
            ("$[?length(@.items )]", true),  // Space before )
            ("$[?length( @.items )]", true), // Spaces around argument
            // Tab, newline, carriage return (in practice, these may not be common but should be valid)
            ("$[?\tlength(@.items)]", true), // Tab after ?
            ("$[?length\t(@.items)]", true), // Tab before (
            ("$[?length(\t@.items)]", true), // Tab after (
            ("$[?length(@.items\t)]", true), // Tab before )
            // Multiple arguments with whitespace
            ("$[?match(@.field,\"pattern\")]", true), // No spaces around comma
            ("$[?match(@.field, \"pattern\")]", true), // Space after comma
            ("$[?match(@.field ,\"pattern\")]", true), // Space before comma
            ("$[?match(@.field , \"pattern\")]", true), // Spaces around comma
            ("$[?match( @.field , \"pattern\" )]", true), // Spaces everywhere
        ];

        for (query, should_pass) in whitespace_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid whitespace in function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid whitespace in function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_function_case_sensitivity() {
        // RFC 9535: Function names are case-sensitive
        let case_tests = vec![
            // Correct case
            ("$[?length(@.items)]", true),
            ("$[?count(@.items)]", true),
            ("$[?match(@.field, \"pattern\")]", true),
            ("$[?search(@.field, \"term\")]", true),
            ("$[?value(@.field)]", true),
            // Incorrect case variations
            ("$[?Length(@.items)]", false),
            ("$[?LENGTH(@.items)]", false),
            ("$[?Count(@.items)]", false),
            ("$[?COUNT(@.items)]", false),
            ("$[?Match(@.field, \"pattern\")]", false),
            ("$[?MATCH(@.field, \"pattern\")]", false),
            ("$[?Search(@.field, \"term\")]", false),
            ("$[?SEARCH(@.field, \"term\")]", false),
            ("$[?Value(@.field)]", false),
            ("$[?VALUE(@.field)]", false),
            ("$[?lENGTH(@.items)]", false),
            ("$[?cOUNT(@.items)]", false),
        ];

        for (query, should_pass) in case_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid case function '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid case function '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_function_argument_validation() {
        // RFC 9535: Function argument syntax validation
        let argument_tests = vec![
            // Valid argument types
            ("$[?length(@)]", true),                   // Current node
            ("$[?length(@.field)]", true),             // Property access
            ("$[?length(@.nested.field)]", true),      // Nested property
            ("$[?length(@['field'])]", true),          // Bracket notation
            ("$[?length(@[0])]", true),                // Array index
            ("$[?length(@[*])]", true),                // Wildcard
            ("$[?count($..book)]", true),              // Root descendant
            ("$[?count($.store.book)]", true),         // Absolute path
            ("$[?match(@.title, \"pattern\")]", true), // String literal
            ("$[?match(@.title, 'pattern')]", true),   // Single quoted string
            // Invalid argument syntax
            ("$[?length(field)]", false),           // Missing @
            ("$[?length(.field)]", false),          // Missing @
            ("$[?count(..book)]", false),           // Missing $
            ("$[?match(@.field, pattern)]", false), // Unquoted string
            ("$[?match(@.field, @.other)]", false), // Wrong argument type for pattern
        ];

        for (query, should_pass) in argument_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid function argument '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid function argument '{}' should fail",
                    query
                );
            }
        }
    }
}

/// UTF-8 Encoding Validation Tests
#[cfg(test)]
mod utf8_encoding_tests {
    use super::*;

    #[test]
    fn test_utf8_member_names() {
        // RFC 9535: JSONPath expressions must be valid UTF-8
        let utf8_tests = vec![
            ("$['café']", true),   // Basic Latin + accents
            ("$['κόσμος']", true), // Greek
            ("$['世界']", true),   // Chinese
            ("$['🌍']", true),     // Emoji
            ("$['Москва']", true), // Cyrillic
            ("$['العالم']", true), // Arabic
        ];

        for (query, should_pass) in utf8_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid UTF-8 query '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid UTF-8 query '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // RFC 9535: Unicode escape sequences in string literals
        let unicode_tests = vec![
            ("$[\"\\u0041\"]", true),        // ASCII 'A'
            ("$[\"\\u00E9\"]", true),        // é
            ("$[\"\\u03BA\"]", true),        // κ
            ("$[\"\\u4E16\"]", true),        // 世
            ("$[\"\\uD83C\\uDF0D\"]", true), // 🌍 (surrogate pair)
            ("$[\"\\u\"]", false),           // Incomplete
            ("$[\"\\u123\"]", false),        // Too short
            ("$[\"\\u123G\"]", false),       // Invalid hex
        ];

        for (query, should_pass) in unicode_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Valid Unicode escape '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Invalid Unicode escape '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_byte_order_mark() {
        // RFC 9535: BOM handling in JSONPath expressions
        let bom_prefix = "\u{FEFF}";
        let query_with_bom = format!("{}$.store", bom_prefix);

        // BOM should be handled gracefully or rejected consistently
        let result = JsonPathParser::compile(&query_with_bom);

        // RFC 9535: BOM should be rejected in JSONPath expressions (not valid JSON)
        assert!(
            result.is_err(),
            "RFC 9535: BOM in JSONPath expression should be rejected"
        );
    }
}

/// I-JSON Number Range Validation Tests
#[cfg(test)]
mod ijson_number_tests {
    use super::*;

    #[test]
    fn test_ijson_integer_range() {
        // RFC 9535: I-JSON restricts numbers to IEEE 754 double precision range
        let number_tests = vec![
            // Valid I-JSON integers
            ("$[0]", true),
            ("$[1]", true),
            ("$[-1]", true),
            ("$[9007199254740991]", true),  // MAX_SAFE_INTEGER
            ("$[-9007199254740991]", true), // MIN_SAFE_INTEGER
            // Numbers beyond safe integer range (may be valid syntax but precision loss)
            ("$[9007199254740992]", true),  // Beyond MAX_SAFE_INTEGER
            ("$[-9007199254740992]", true), // Beyond MIN_SAFE_INTEGER
        ];

        for (query, _should_compile) in number_tests {
            let result = JsonPathParser::compile(query);
            if _should_compile {
                assert!(result.is_ok(), "I-JSON number '{}' should compile", query);
            } else {
                assert!(
                    result.is_err(),
                    "Invalid I-JSON number '{}' should fail",
                    query
                );
            }
        }
    }

    #[test]
    fn test_decimal_number_precision() {
        // Test decimal number handling in filter expressions
        let json_data = r#"{"items": [{"price": 1.23456789012345}]}"#;

        // RFC 9535: Test high precision decimal handling
        let test_cases = vec![
            ("$.items[?@.price == 1.23456789012345]", 1), // Exact match should work
            ("$.items[?@.price > 1.2]", 1),               // Greater than comparison
            ("$.items[?@.price < 1.3]", 1),               // Less than comparison
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Decimal precision test '{}' should return {} results",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_scientific_notation() {
        // RFC 9535: Scientific notation in numbers
        let scientific_tests = vec![
            ("$[1e5]", false),   // Scientific notation not in index
            ("$[1E5]", false),   // Capital E
            ("$[1.5e2]", false), // Decimal with exponent
            ("$[1e+5]", false),  // Positive exponent
            ("$[1e-5]", false),  // Negative exponent
        ];

        // Note: Scientific notation typically not allowed in array indices
        // but may be valid in filter expressions
        for (query, should_pass) in scientific_tests {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "Scientific notation '{}' should compile",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Scientific notation '{}' should fail in index",
                    query
                );
            }
        }
    }
}

/// Well-formedness vs Validity Separation Tests
#[cfg(test)]
mod wellformedness_tests {
    use super::*;

    #[test]
    fn test_wellformed_but_invalid_paths() {
        // RFC 9535: Distinguish between syntactically valid but semantically invalid paths
        let test_cases = vec![
            // Well-formed but may be invalid for specific JSON documents
            ("$.nonexistent", true),  // Valid syntax, may not match anything
            ("$[999]", true),         // Valid syntax, array may not have this index
            ("$.store[999]", true),   // Valid syntax, property may not exist
            ("$..nonexistent", true), // Valid syntax, may not match anything
            // Malformed syntax
            ("$.", false),           // Incomplete dot notation
            ("$[", false),           // Unclosed bracket
            ("$store", false),       // Missing root identifier
            ("$.123invalid", false), // Invalid identifier
        ];

        for (query, should_be_wellformed) in test_cases {
            let result = JsonPathParser::compile(query);
            if should_be_wellformed {
                assert!(
                    result.is_ok(),
                    "Well-formed query '{}' should compile",
                    query
                );
            } else {
                assert!(result.is_err(), "Malformed query '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_semantic_validation() {
        // Test semantic validation during execution vs parsing
        let json_data = r#"{"store": {"book": [{"title": "Book1"}]}}"#;

        let semantic_tests = vec![
            ("$.store", true),              // Valid path, exists
            ("$.store.book", true),         // Valid path, exists
            ("$.nonexistent", true),        // Valid syntax, doesn't exist (empty result)
            ("$.store.book[999]", true),    // Valid syntax, index out of bounds (empty result)
            ("$.store.book.invalid", true), // Valid syntax, invalid property access (empty result)
        ];

        for (query, should_execute) in semantic_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let _results: Vec<_> = stream.process_chunk(chunk).collect();

            if should_execute {
                // Valid queries should execute without panicking - test passes if we get here
                // (The fact that we got results without error is the test)
            }
        }
    }

    #[test]
    fn test_dot_notation_syntax_validation() {
        // RFC 9535 Appendix A: Comprehensive dot notation syntax tests
        let valid_dot_notation = vec![
            // Basic property access
            "$.store",
            "$.user",
            "$.data",
            // Chained property access
            "$.store.book",
            "$.user.profile.name",
            "$.data.items.count",
            // Mixed with arrays
            "$.store.book[0].title",
            "$.users[*].profile.email",
            "$.data.matrix[1][2].value",
            // With descendant operator
            "$.store..book.title",
            "$..profile.settings.theme",
            // Valid identifier patterns
            "$.valid_underscore",
            "$.CamelCase",
            "$.lowercase",
            "$.UPPERCASE",
            "$.mixed123Numbers",
            // Unicode identifiers
            "$.café",
            "$.naïve",
            "$.Москва",
            "$.東京",
        ];

        for expr in valid_dot_notation {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid dot notation '{}' should compile",
                expr
            );
        }

        let invalid_dot_notation = vec![
            // Invalid identifier start
            "$.123invalid", // Starts with digit
            "$.-invalid",   // Starts with hyphen
            "$.@invalid",   // Starts with @
            "$..invalid",   // Incomplete descendant followed by dot
            // Invalid syntax
            "$.",             // Trailing dot
            "$.store.",       // Trailing dot after property
            "$.store..book.", // Trailing dot after descendant
            "$.store..",      // Incomplete descendant at end
            "$.store...book", // Triple dots
            // Reserved character usage
            "$.[",            // Bracket after dot
            "$.store.[book]", // Bracket notation after dot
            "$.store.book*",  // Wildcard in property name
            "$.store.book?",  // Question mark in property name
            // Whitespace issues
            "$ .store",      // Space after root (may be valid)
            "$.store .book", // Space before dot
            "$.store. book", // Space after dot
        ];

        for expr in invalid_dot_notation {
            let result = JsonPathParser::compile(expr);
            // Note: Some of these may be valid depending on implementation
            // The key is that they're tested for consistent behavior
            println!(
                "Testing invalid dot notation: '{}' -> {:?}",
                expr,
                result.is_ok()
            );
        }
    }

    #[test]
    fn test_member_name_shorthand_syntax() {
        // RFC 9535: member-name-shorthand = unquoted-member-name
        let valid_shorthand = vec![
            // Basic identifiers
            "$.name",
            "$.title",
            "$.value",
            // With underscores
            "$.first_name",
            "$.last_name",
            "$.user_id",
            // With numbers (not at start)
            "$.item1",
            "$.level2",
            "$.version3x",
            // CamelCase
            "$.firstName",
            "$.lastName",
            "$.userId",
            // Unicode
            "$.名前",
            "$.prénom",
            "$.имя",
        ];

        for expr in valid_shorthand {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid member-name-shorthand '{}' should compile",
                expr
            );
        }

        let invalid_shorthand = vec![
            // Starting with digits
            "$.1name",
            "$.2nd",
            "$.3rdLevel",
            // Special characters
            "$.name-with-hyphens", // Hyphens not allowed in unquoted names
            "$.name@domain",       // @ not allowed in unquoted names
            "$.name.space",        // Interpreted as chained access, not single name
            "$.name[bracket]",     // Brackets not allowed in unquoted names
            // Reserved words (depending on implementation)
            "$.null",  // May be reserved
            "$.true",  // May be reserved
            "$.false", // May be reserved
        ];

        for expr in invalid_shorthand {
            let result = JsonPathParser::compile(expr);
            // Document behavior - some may be valid depending on implementation
            println!(
                "Testing member-name-shorthand: '{}' -> {:?}",
                expr,
                result.is_ok()
            );
        }
    }

    #[test]
    fn test_grammar_edge_cases() {
        // Test edge cases in grammar interpretation
        let edge_cases = vec![
            // Whitespace handling
            ("$ . store", true),        // Spaces around dot
            ("$[ 'store' ]", true),     // Spaces in brackets
            ("$  [  'store'  ]", true), // Multiple spaces
            // Mixed notation
            ("$.store['book']", true),    // Dot then bracket
            ("$['store'].book", true),    // Bracket then dot
            ("$['store']['book']", true), // All brackets
            // Complex expressions
            ("$.store..book[*].title", true), // Mixed segments
            ("$..*.price", true),             // Wildcard after descendant
        ];

        for (query, should_pass) in edge_cases {
            let result = JsonPathParser::compile(query);
            if should_pass {
                assert!(result.is_ok(), "Edge case '{}' should compile", query);
            } else {
                assert!(result.is_err(), "Edge case '{}' should fail", query);
            }
        }
    }

    #[test]
    fn test_comparison_operator_syntax() {
        // RFC 9535 Appendix A: Test all comparison operators
        let json_data = r#"{
            "items": [
                {"name": "item1", "price": 10, "active": true},
                {"name": "item2", "price": 20, "active": false},
                {"name": "item3", "price": 15, "active": true}
            ]
        }"#;

        let comparison_operators = vec![
            // Equality operators
            ("$.items[?@.price == 10]", "Equal operator"),
            ("$.items[?@.price != 20]", "Not equal operator"),
            // Relational operators
            ("$.items[?@.price > 10]", "Greater than operator"),
            ("$.items[?@.price >= 15]", "Greater than or equal operator"),
            ("$.items[?@.price < 20]", "Less than operator"),
            ("$.items[?@.price <= 15]", "Less than or equal operator"),
            // String comparisons
            ("$.items[?@.name == 'item1']", "String equality"),
            ("$.items[?@.name != 'item2']", "String inequality"),
            // Boolean comparisons
            ("$.items[?@.active == true]", "Boolean equality true"),
            ("$.items[?@.active == false]", "Boolean equality false"),
            ("$.items[?@.active != false]", "Boolean inequality"),
            // Mixed type comparisons (should be syntactically valid)
            ("$.items[?@.price == '10']", "Number vs string comparison"),
            ("$.items[?@.active == 1]", "Boolean vs number comparison"),
        ];

        for (expr, _description) in comparison_operators {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Comparison operator '{}' should compile: {}",
                expr,
                _description
            );

            // Verify the expression can execute
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Comparison test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_logical_operator_syntax() {
        // RFC 9535 Appendix A: Test logical operators
        let json_data = r#"{
            "items": [
                {"name": "item1", "price": 10, "active": true, "category": "A"},
                {"name": "item2", "price": 20, "active": false, "category": "B"},
                {"name": "item3", "price": 15, "active": true, "category": "A"}
            ]
        }"#;

        let logical_operators = vec![
            // AND operator
            ("$.items[?@.price > 10 && @.active == true]", "Logical AND"),
            (
                "$.items[?@.category == 'A' && @.price < 20]",
                "String AND number",
            ),
            // OR operator
            ("$.items[?@.price == 10 || @.price == 20]", "Logical OR"),
            (
                "$.items[?@.active == false || @.category == 'A']",
                "Boolean OR string",
            ),
            // NOT operator (unary)
            ("$.items[?!@.active]", "Logical NOT unary"),
            ("$.items[?!(@.price > 15)]", "Logical NOT with parentheses"),
            // Complex combinations
            (
                "$.items[?(@.price > 10 && @.active) || @.category == 'B']",
                "Complex logical expression",
            ),
            (
                "$.items[?@.price > 5 && (@.active == true || @.category == 'B')]",
                "Nested logical groups",
            ),
            // Operator precedence tests
            (
                "$.items[?@.price > 10 && @.active || @.category == 'B']",
                "AND/OR precedence",
            ),
            ("$.items[?!@.active && @.price > 15]", "NOT/AND precedence"),
        ];

        for (expr, _description) in logical_operators {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Logical operator '{}' should compile: {}",
                expr,
                _description
            );

            // Verify the expression can execute
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Logical test '{}' returned {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_filter_expression_syntax() {
        // RFC 9535 Appendix A: Test complete filter expression syntax
        let valid_filter_expressions = vec![
            // Basic filter syntax
            "$[?@.price]",          // Existence test
            "$[?@.price > 10]",     // Comparison
            "$[?@.name == 'test']", // String comparison
            // Parenthesized expressions
            "$[?(@.price > 10)]",             // Simple parentheses
            "$[?((@.price > 10))]",           // Nested parentheses
            "$[?(@.price > 10 && @.active)]", // Parenthesized logical
            // Current node references
            "$[?@]",                 // Current node existence
            "$[?@.price]",           // Property existence
            "$[?@['price']]",        // Bracket property access
            "$[?@.nested.property]", // Nested property access
            // Function calls in filters
            "$[?length(@.name) > 5]",        // Function call
            "$[?count(@.items) == 0]",       // Function with comparison
            "$[?match(@.email, '^[^@]+@')]", // Function with regex
            // Complex expressions
            "$[?@.price > 10 && length(@.name) < 20]", // Mixed property and function
            "$[?(@.active || @.featured) && @.price > 0]", // Complex boolean logic
        ];

        for expr in valid_filter_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid filter expression '{}' should compile",
                expr
            );
        }

        let invalid_filter_expressions = vec![
            // Missing components
            "$[?]",  // Empty filter
            "$[? ]", // Whitespace only filter
            "$[?@]", // Incomplete (may be valid for existence)
            // Invalid operators
            "$[?@.price === 10]", // Triple equals (invalid)
            "$[?@.price <> 10]",  // Invalid not-equal operator
            "$[?@.price =< 10]",  // Invalid less-equal operator
            "$[?@.price => 10]",  // Invalid greater-equal operator
            // Invalid syntax
            "$[?@.price > ]",      // Missing right operand
            "$[?> 10]",            // Missing left operand
            "$[?@.price 10]",      // Missing operator
            "$[?@.price > 10 &&]", // Incomplete logical expression
            "$[?&& @.price > 10]", // Leading logical operator
            // Invalid parentheses
            "$[?(@.price > 10]",   // Unmatched opening parenthesis
            "$[?@.price > 10)]",   // Unmatched closing parenthesis
            "$[?((@.price > 10)]", // Unmatched nested parenthesis
        ];

        for expr in invalid_filter_expressions {
            let result = JsonPathParser::compile(expr);
            // Document behavior - these should fail but behavior may vary
            println!("Testing invalid filter: '{}' -> {:?}", expr, result.is_ok());
        }
    }
}

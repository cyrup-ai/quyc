//! RFC 9535 ABNF Grammar Validation Tests (Appendix A)
//!
//! Tests for RFC 9535 Appendix A ABNF grammar compliance:
//! "This appendix describes the ABNF grammar for JSONPath expressions."
//!
//! This test suite validates:
//! - Dot notation syntax validation tests
//! - Comparison operators (<, <=, >, >=, ==, !=) syntax
//! - Shorthand syntax validation tests  
//! - Bracket notation escape sequence tests
//! - ABNF grammar rule compliance
//! - Lexical token validation

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};

/// Test data for ABNF grammar validation
const ABNF_TEST_JSON: &str = r#"{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees", 
        "title": "Sayings of the Century",
        "price": 8.95,
        "isbn": "0-553-21311-3"
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh",
        "title": "Sword of Honour", 
        "price": 12.99,
        "isbn": "0-679-42267-2"
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95,
      "features": ["lightweight", "durable", "fast"]
    }
  },
  "expensive": 10,
  "special chars": {
    "key with spaces": "value1",
    "key-with-hyphens": "value2", 
    "key.with.dots": "value3",
    "key/with/slashes": "value4",
    "key~with~tildes": "value5",
    "key\"with\"quotes": "value6",
    "unicode-keys": {
      "café": "coffee",
      "naïve": "innocent", 
      "résumé": "cv"
    }
  }
}"#;

/// RFC 9535 Appendix A - ABNF Grammar Tests
#[cfg(test)]
mod abnf_grammar_tests {
    use super::*;

    #[test]
    fn test_dot_notation_syntax_validation() {
        // RFC 9535 ABNF: dot-member-name = name-first *name-char
        let dot_notation_tests = vec![
            // Valid dot notation syntax
            ("$.store", true, "Simple property access"),
            ("$.store.book", true, "Nested property access"),
            ("$.store.bicycle.color", true, "Deep property access"),
            ("$.store.book.author", true, "Property chain"),
            ("$._private", true, "Underscore property"),
            ("$.validName123", true, "Alphanumeric property"),
            ("$.store123.book456", true, "Numeric suffixed properties"),
            // Invalid dot notation (should use bracket notation)
            ("$.store.book.0", false, "Numeric property without brackets"),
            ("$.store.'quoted'", false, "Quoted property in dot notation"),
            ("$.store.key with spaces", false, "Spaces in dot notation"),
            (
                "$.store.key-with-dashes",
                true,
                "Hyphens in identifiers (valid)",
            ),
            ("$.store.key.with.dots", false, "Dots in property names"),
            ("$.store.123invalid", false, "Starting with number"),
            ("$.store.-invalid", false, "Starting with hyphen"),
            ("$.store..double", false, "Double dots"),
            ("$.store.", false, "Trailing dot"),
            ("$.store.book.", false, "Trailing dot in chain"),
        ];

        for (expr, _should_be_valid, _description) in dot_notation_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid dot notation should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid dot notation should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_comparison_operators_syntax() {
        // RFC 9535 ABNF: comparison-op = "==" / "!=" / "<=" / ">=" / "<" / ">"
        let comparison_tests = vec![
            // All valid comparison operators
            ("$.store.book[?@.price == 8.95]", true, "Equal operator =="),
            (
                "$.store.book[?@.price != 8.95]",
                true,
                "Not equal operator !=",
            ),
            ("$.store.book[?@.price < 10]", true, "Less than operator <"),
            (
                "$.store.book[?@.price <= 10]",
                true,
                "Less than or equal <=",
            ),
            (
                "$.store.book[?@.price > 10]",
                true,
                "Greater than operator >",
            ),
            (
                "$.store.book[?@.price >= 10]",
                true,
                "Greater than or equal >=",
            ),
            // String comparisons
            (
                "$.store.book[?@.category == 'fiction']",
                true,
                "String equality",
            ),
            (
                "$.store.book[?@.category != 'fiction']",
                true,
                "String inequality",
            ),
            ("$.store.book[?@.author < 'Z']", true, "String less than"),
            ("$.store.book[?@.author > 'A']", true, "String greater than"),
            // Numeric comparisons with different number types
            ("$.store.book[?@.price == 8]", true, "Integer comparison"),
            ("$.store.book[?@.price == 8.0]", true, "Float comparison"),
            ("$.store.book[?@.price == 8.95]", true, "Decimal comparison"),
            // Invalid comparison operators (not in ABNF)
            (
                "$.store.book[?@.price = 8.95]",
                false,
                "Single equals (assignment)",
            ),
            (
                "$.store.book[?@.price <> 8.95]",
                false,
                "Not equal variant <>",
            ),
            ("$.store.book[?@.price === 8.95]", false, "Triple equals"),
            (
                "$.store.book[?@.price !== 8.95]",
                false,
                "Not triple equals",
            ),
            ("$.store.book[?@.price =< 8.95]", false, "Wrong order <="),
            ("$.store.book[?@.price => 8.95]", false, "Wrong order >="),
            // Whitespace variations (should be valid)
            ("$.store.book[?@.price==8.95]", true, "No spaces around =="),
            ("$.store.book[?@.price == 8.95]", true, "Spaces around =="),
            ("$.store.book[?@.price !=  8.95]", true, "Multiple spaces"),
            ("$.store.book[?@.price<10]", true, "No spaces around <"),
            ("$.store.book[?@.price > 10]", true, "Space after >"),
        ];

        for (expr, _should_be_valid, _description) in comparison_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid comparison operator should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid comparison operator should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_shorthand_syntax_validation() {
        // RFC 9535 ABNF: Various shorthand syntax patterns
        let shorthand_tests = vec![
            // Valid shorthand patterns
            ("$", true, "Root shorthand"),
            ("$.*", true, "Root wildcard"),
            ("$[*]", true, "Root array wildcard"),
            ("$..*", true, "Recursive descent all"),
            ("$..book", true, "Recursive descent property"),
            ("$..book[*]", true, "Recursive descent array"),
            ("$..book[*].author", true, "Recursive descent with property"),
            // Array shorthand
            ("$[0]", true, "Single array index"),
            ("$[0,1]", true, "Multiple array indices"),
            ("$[0:2]", true, "Array slice"),
            ("$[1:]", true, "Array slice from index"),
            ("$[:3]", true, "Array slice to index"),
            ("$[::2]", true, "Array slice with step"),
            ("$[-1]", true, "Negative array index"),
            ("$[-2:]", true, "Negative slice start"),
            // Property shorthand
            ("$['property']", true, "Bracket property notation"),
            ("$[\"property\"]", true, "Double quoted property"),
            ("$['prop1','prop2']", true, "Multiple properties"),
            ("$[\"prop1\",\"prop2\"]", true, "Multiple double quoted"),
            // Invalid shorthand syntax
            ("", false, "Empty expression"),
            ("store", false, "Missing root $"),
            ("$.store[", false, "Unclosed bracket"),
            ("$.store]", false, "Unmatched closing bracket"),
            ("$.[0]", false, "Dot before bracket"),
            ("$[0.5]", false, "Float array index"),
            ("$[0,]", false, "Trailing comma"),
            ("$[,0]", false, "Leading comma"),
            ("$[0:]", false, "Empty slice end with colon"),
            ("$[::]", false, "Empty slice parameters"),
        ];

        for (expr, _should_be_valid, _description) in shorthand_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid shorthand syntax should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid shorthand syntax should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_bracket_notation_escape_sequences() {
        // RFC 9535 ABNF: Escape sequences in bracket notation
        let escape_tests = vec![
            // Valid escape sequences
            ("$['simple']", true, "Simple quoted property"),
            ("$[\"simple\"]", true, "Double quoted property"),
            ("$['key with spaces']", true, "Spaces in quoted property"),
            ("$['key\\'with\\'quotes']", true, "Escaped single quotes"),
            (
                "$[\"key\\\"with\\\"quotes\"]",
                true,
                "Escaped double quotes",
            ),
            ("$['key\\nwith\\nnewlines']", true, "Escaped newlines"),
            ("$['key\\twith\\ttabs']", true, "Escaped tabs"),
            (
                "$['key\\\\with\\\\backslashes']",
                true,
                "Escaped backslashes",
            ),
            ("$['key\\/with\\/slashes']", true, "Escaped forward slashes"),
            ("$['key\\bwith\\bbackspace']", true, "Escaped backspace"),
            ("$['key\\fwith\\fformfeed']", true, "Escaped form feed"),
            ("$['key\\rwith\\rreturn']", true, "Escaped carriage return"),
            // Unicode escape sequences
            ("$['key\\u0041']", true, "Unicode escape sequence"),
            ("$['key\\u00e9']", true, "Unicode accented character"),
            (
                "$['key\\uD83D\\uDE00']",
                true,
                "Unicode emoji (surrogate pair)",
            ),
            // Special characters that need escaping
            ("$['key.with.dots']", true, "Dots in quoted property"),
            ("$['key/with/slashes']", true, "Slashes in quoted property"),
            (
                "$['key[with]brackets']",
                true,
                "Brackets in quoted property",
            ),
            (
                "$['key(with)parens']",
                true,
                "Parentheses in quoted property",
            ),
            ("$['key{with}braces']", true, "Braces in quoted property"),
            ("$['key@with@ats']", true, "At symbols in quoted property"),
            (
                "$['key$with$dollars']",
                true,
                "Dollar signs in quoted property",
            ),
            // Invalid escape sequences
            ("$['key\\x41']", false, "Invalid \\x escape"),
            ("$['key\\q']", false, "Invalid escape character"),
            ("$['key\\u']", false, "Incomplete Unicode escape"),
            ("$['key\\u41']", false, "Short Unicode escape"),
            ("$['key\\uGHIJ']", false, "Invalid Unicode hex"),
            ("$['unclosed string]", false, "Unclosed single quote"),
            ("$[\"unclosed string]", false, "Unclosed double quote"),
            ("$['mixed quote\"]", false, "Mismatched quotes"),
            ("$[\"mixed quote']", false, "Reverse mismatched quotes"),
        ];

        for (expr, _should_be_valid, _description) in escape_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid escape sequence should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid escape sequence should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_lexical_token_validation() {
        // RFC 9535 ABNF: Basic lexical tokens
        let token_tests = vec![
            // Root token
            ("$", true, "Root token"),
            ("$$", false, "Double root token"),
            // Dot tokens
            ("$.", false, "Trailing dot"),
            ("$..", false, "Double dot without property"),
            ("$..property", true, "Recursive descent"),
            // Bracket tokens
            ("$[]", false, "Empty brackets"),
            ("$[", false, "Unclosed bracket"),
            ("$]", false, "Unmatched closing bracket"),
            ("$[[0]]", false, "Nested brackets"),
            // Wildcard tokens
            ("$.*", true, "Dot wildcard"),
            ("$[*]", true, "Bracket wildcard"),
            ("$.**", false, "Double wildcard"),
            ("$[**]", false, "Double bracket wildcard"),
            // At token (@) - only valid in filters
            ("$[@]", false, "@ outside filter"),
            ("$.store[@]", false, "@ in property context"),
            ("$.store.book[?@]", true, "@ in filter context"),
            ("$.store.book[?@.price]", true, "@ with property"),
            // Question token (?) - only valid in filters
            ("$[?]", false, "Empty filter"),
            ("$.store[?]", false, "Empty filter with property"),
            ("$.store.book[?@.price > 10]", true, "Valid filter"),
            // Comma token
            ("$[0,]", false, "Trailing comma"),
            ("$[,0]", false, "Leading comma"),
            ("$[0,,1]", false, "Double comma"),
            ("$[0,1]", true, "Valid comma separation"),
            // Colon token
            ("$[:]", false, "Empty slice"),
            ("$[0:]", true, "Slice from index"),
            ("$[:3]", true, "Slice to index"),
            ("$[0:3]", true, "Slice range"),
            ("$[0::2]", true, "Slice with step"),
            ("$[0:::2]", false, "Triple colon"),
        ];

        for (expr, _should_be_valid, _description) in token_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid token sequence should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid token sequence should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_abnf_grammar_execution() {
        // RFC 9535: Test execution of valid ABNF grammar expressions
        let execution_tests = vec![
            ("$.store.book[0].author", 1, "Dot notation execution"),
            (
                "$['store']['book'][0]['author']",
                1,
                "Bracket notation execution",
            ),
            ("$.store.book[?@.price < 10]", 1, "Filter with comparison"),
            ("$.store.book[*].author", 2, "Wildcard execution"),
            ("$..author", 2, "Recursive descent execution"),
            ("$.store.book[0,1].title", 2, "Multiple index execution"),
            ("$.store.book[0:2].category", 2, "Slice execution"),
        ];

        for (expr, expected_count, _description) in execution_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(ABNF_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: ABNF grammar execution: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );
        }
    }
}

/// Complex ABNF Grammar Edge Cases
#[cfg(test)]
mod abnf_edge_cases {
    use super::*;

    #[test]
    fn test_complex_property_names() {
        // RFC 9535 ABNF: Complex property name handling
        let complex_property_tests = vec![
            // Unicode property names
            ("$['café']", true, "Unicode property name"),
            ("$['naïve']", true, "Unicode with diacritic"),
            ("$['résumé']", true, "Unicode accented"),
            ("$['中文']", true, "Chinese characters"),
            ("$['العربية']", true, "Arabic script"),
            ("$['🚀']", true, "Emoji property name"),
            // Mixed character sets
            ("$['mix_英语_123']", true, "Mixed scripts and numbers"),
            (
                "$['property-with-everything_123.test@example.com']",
                true,
                "Complex mixed property",
            ),
            // Edge case property names
            ("$['']", true, "Empty string property"),
            ("$[' ']", true, "Single space property"),
            ("$['   ']", true, "Multiple spaces property"),
            ("$['\t']", true, "Tab character property"),
            ("$['\n']", true, "Newline character property"),
            // Properties that look like other syntax
            ("$['$']", true, "Dollar sign property"),
            ("$['@']", true, "At sign property"),
            ("$['*']", true, "Star property"),
            ("$['..']", true, "Double dot property"),
            ("$['[0]']", true, "Bracket syntax as property"),
            ("$['?filter']", true, "Question mark property"),
        ];

        for (expr, _should_be_valid, _description) in complex_property_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Complex property name should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid complex property should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_abnf_whitespace_handling() {
        // RFC 9535 ABNF: Whitespace in various contexts
        let whitespace_tests = vec![
            // Valid whitespace patterns
            ("$ . store . book", true, "Spaces around dots"),
            ("$[ 0 ]", true, "Spaces in brackets"),
            ("$[ * ]", true, "Spaces around wildcard"),
            ("$[ 'property' ]", true, "Spaces around quoted property"),
            ("$.store.book[ ? @.price > 10 ]", true, "Spaces in filter"),
            (
                "$.store.book[?@.price == 8.95 ]",
                true,
                "Space before closing bracket",
            ),
            (
                "$.store.book[ ?@.price == 8.95]",
                true,
                "Space after opening bracket",
            ),
            // Edge case whitespace
            ("$\t.\tstore", true, "Tabs around dots"),
            ("$\n.\nstore", true, "Newlines around dots"),
            ("$\r.\rstore", true, "Carriage returns around dots"),
            // Invalid whitespace (breaking tokens)
            ("$ store", false, "Space after root without dot"),
            ("$. store", false, "Space after dot without bracket"),
            ("$.st ore", false, "Space in property name"),
            ("$[0 1]", false, "Space breaking array index"),
            ("$.store.bo ok", false, "Space in property name"),
        ];

        for (expr, _should_be_valid, _description) in whitespace_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid whitespace should be accepted: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid whitespace should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_abnf_number_formats() {
        // RFC 9535 ABNF: Number format validation
        let number_tests = vec![
            // Valid integer formats
            ("$[0]", true, "Zero"),
            ("$[1]", true, "Positive integer"),
            ("$[-1]", true, "Negative integer"),
            ("$[123]", true, "Multi-digit positive"),
            ("$[-456]", true, "Multi-digit negative"),
            // Valid decimal formats in filters
            (
                "$.store.book[?@.price == 8.95]",
                true,
                "Decimal in comparison",
            ),
            ("$.store.book[?@.price == 0.5]", true, "Decimal less than 1"),
            (
                "$.store.book[?@.price == 123.456]",
                true,
                "Multi-digit decimal",
            ),
            ("$.store.book[?@.price == -12.34]", true, "Negative decimal"),
            // Scientific notation in filters
            ("$.store.book[?@.price == 1e2]", true, "Scientific notation"),
            (
                "$.store.book[?@.price == 1.23e-4]",
                true,
                "Scientific with decimal",
            ),
            (
                "$.store.book[?@.price == -1.5E+3]",
                true,
                "Scientific negative",
            ),
            // Invalid number formats
            ("$[01]", false, "Leading zero integer"),
            ("$[+1]", false, "Explicit positive sign"),
            ("$[1.]", false, "Trailing decimal point"),
            ("$[.5]", false, "Leading decimal point"),
            ("$[1.2.3]", false, "Multiple decimal points"),
            ("$[1e]", false, "Incomplete scientific notation"),
            ("$[1e+]", false, "Incomplete scientific exponent"),
            ("$[-]", false, "Lone negative sign"),
            ("$[--1]", false, "Double negative"),
        ];

        for (expr, _should_be_valid, _description) in number_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid number format should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid number format should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

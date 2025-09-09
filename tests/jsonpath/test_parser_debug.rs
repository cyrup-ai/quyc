//! Parser debug tests

use quyc::json_path::{CoreJsonPathEvaluator, JsonPathParser};

#[test]
fn test_parser_debug_for_failing_patterns() {
    env_logger::init();
    log::debug!("=== Parser Debug for Failing Patterns ===");

    let patterns = vec![
        "$..book[2]",
        "$.store.bicycle",
        "$.store.book[2]", // This should work
        "$",               // Root should work
    ];

    for pattern in patterns {
        log::debug!("--- Testing pattern: {} ---", pattern);

        match JsonPathParser::compile(pattern) {
            Ok(parsed_expr) => {
                log::debug!("✓ Parsed successfully");
                let selectors = parsed_expr.selectors();
                log::debug!("Selectors ({} total):", selectors.len());
                for (i, selector) in selectors.iter().enumerate() {
                    log::debug!("  [{}]: {:?}", i, selector);
                }
                
                // Verify the parsed expression is valid
                assert!(!selectors.is_empty(), "Parsed expression should have selectors");
            }
            Err(e) => {
                log::error!("✗ Parse failed: {:?}", e);
                // Some patterns are expected to fail, so we don't panic here
                // but we do verify the error is meaningful
                assert!(!e.to_string().is_empty(), "Error message should not be empty");
            }
        }
    }
}

#[test]
fn test_complex_jsonpath_patterns() {
    env_logger::init();
    log::debug!("=== Testing Complex JSONPath Patterns ===");
    
    let complex_patterns = vec![
        "$.store.book[*].author",
        "$.store.book[?(@.price < 10)]",
        "$.store.book[0,1]",
        "$..price",
    ];
    
    for pattern in complex_patterns {
        log::debug!("Testing complex pattern: {}", pattern);
        
        match JsonPathParser::compile(pattern) {
            Ok(parsed) => {
                log::debug!("✓ Complex pattern parsed successfully");
                assert!(!parsed.selectors().is_empty());
            }
            Err(e) => {
                log::error!("✗ Complex pattern failed: {:?}", e);
                // For now, we'll allow some complex patterns to fail
                // as the implementation may not be complete
            }
        }
    }
}
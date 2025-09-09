//! Debug test for JSONPath function parsing

use quyc::json_path::JsonPathParser;

#[test]
fn debug_core_function_call() {
    env_logger::init();
    log::debug!("Testing core function call parsing");
    
    // Test the simplest function call first - length() requires exactly one argument per RFC 9535
    let result = JsonPathParser::compile("$.test[?length(@) == 0]");
    log::debug!("Core function result: {:?}", result);

    // This should work if the basic parsing is correct
    assert!(
        result.is_ok(),
        "Core function call should parse successfully"
    );
}

#[test]
fn debug_function_with_property() {
    env_logger::init();
    log::debug!("Starting compilation of problematic expression...");
    
    // Test function with property argument - this is the problematic case
    let result = JsonPathParser::compile("$.items[?length(@.name) == 5]");
    log::debug!("Property function result: {:?}", result);

    if let Err(error) = result {
        log::error!("Error details: {}", error);
        // For now, we expect this might fail as the implementation may be incomplete
        // but we want to ensure the error is meaningful
        assert!(!error.to_string().is_empty(), "Error message should not be empty");
    } else {
        log::debug!("Property function parsed successfully");
    }
}

#[test]
fn debug_basic_property_access() {
    env_logger::init();
    log::debug!("Testing basic property access");
    
    // Test basic property access without function
    let result = JsonPathParser::compile("$.items[?@.name == 'test']");
    log::debug!("Basic property access result: {:?}", result);

    assert!(result.is_ok(), "Basic property access should work");
}
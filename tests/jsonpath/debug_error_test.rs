//! Debug test for @ error messages

use quyc::json_path::JsonPathParser;

#[test]
fn debug_at_error_messages() {
    env_logger::init();
    log::debug!("Testing @ error messages in JSONPath expressions...");

    let invalid_expressions = vec![
        "@",          // Bare @ as root
        "$.@",        // @ as segment
        "$.store[@]", // @ as selector (not in filter)
    ];

    for expr in invalid_expressions {
        log::debug!("Testing: {}", expr);
        match JsonPathParser::compile(expr) {
            Ok(_) => {
                log::warn!("Compiled successfully (unexpected!)");
                panic!("Expected compilation to fail for invalid expression: {}", expr);
            },
            Err(e) => {
                log::debug!("Failed as expected: {:?}", e);
                assert!(e.to_string().contains("@") || e.to_string().contains("invalid"), 
                       "Error message should mention @ or invalid syntax");
            }
        }
    }
}
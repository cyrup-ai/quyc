//! Debug test for @ parsing

use quyc::json_path::JsonPathParser;

#[test]
fn debug_at_parsing() {
    env_logger::init();
    log::debug!("Testing @ parsing in JSONPath expressions...");

    let test_expressions = vec![
        "$.store.books[?@.active]",
        "$.store.books[?@.id > 1]",
        "$.store.books[?@.value >= 15.0]",
    ];

    for expr in test_expressions {
        log::debug!("Testing: {}", expr);
        match JsonPathParser::compile(expr) {
            Ok(_) => log::debug!("  ✓ Compiled successfully"),
            Err(e) => log::error!("  ✗ Failed: {:?}", e),
        }
    }
}
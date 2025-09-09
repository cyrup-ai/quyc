use quyc_client::jsonpath::type_system::{FunctionSignature, FunctionType, TypeSystem, TypedValue};

#[test]
fn test_function_signatures() {
    let length_sig = TypeSystem::get_function_signature("length")
        .expect("Failed to get function signature for 'length'");
    assert_eq!(length_sig.parameter_types.len(), 1);
    assert_eq!(length_sig.parameter_types[0], FunctionType::ValueType);
    assert_eq!(length_sig.return_type, FunctionType::ValueType);

    let count_sig = TypeSystem::get_function_signature("count")
        .expect("Failed to get function signature for 'count'");
    assert_eq!(count_sig.parameter_types.len(), 1);
    assert_eq!(count_sig.parameter_types[0], FunctionType::NodesType);
    assert_eq!(count_sig.return_type, FunctionType::ValueType);

    assert!(TypeSystem::get_function_signature("unknown").is_none());
}

#[test]
fn test_type_conversions() {
    // ValueType to LogicalType
    let string_val = TypedValue::Value(serde_json::json!("hello"));
    let logical = TypeSystem::convert_type(string_val, FunctionType::LogicalType)
        .expect("Failed to convert string ValueType to LogicalType");
    assert!(matches!(logical, TypedValue::Logical(true)));

    let empty_string_val = TypedValue::Value(serde_json::json!(""));
    let logical = TypeSystem::convert_type(empty_string_val, FunctionType::LogicalType)
        .expect("Failed to convert empty string ValueType to LogicalType");
    assert!(matches!(logical, TypedValue::Logical(false)));

    // NodesType to ValueType (single node)
    let single_node = TypedValue::Nodes(vec![serde_json::json!("value")]);
    let value = TypeSystem::convert_type(single_node, FunctionType::ValueType)
        .expect("Failed to convert single node NodesType to ValueType");
    assert!(matches!(value, TypedValue::Value(_)));

    // NodesType to ValueType (multiple nodes - should fail)
    let multi_nodes = TypedValue::Nodes(vec![serde_json::json!("a"), serde_json::json!("b")]);
    assert!(TypeSystem::convert_type(multi_nodes, FunctionType::ValueType).is_err());
}

#[test]
fn test_value_to_logical() {
    assert!(!TypeSystem::value_to_logical(&serde_json::Value::Null));
    assert!(!TypeSystem::value_to_logical(&serde_json::json!(false)));
    assert!(TypeSystem::value_to_logical(&serde_json::json!(true)));
    assert!(!TypeSystem::value_to_logical(&serde_json::json!(0)));
    assert!(TypeSystem::value_to_logical(&serde_json::json!(1)));
    assert!(!TypeSystem::value_to_logical(&serde_json::json!("")));
    assert!(TypeSystem::value_to_logical(&serde_json::json!("hello")));
    assert!(TypeSystem::value_to_logical(&serde_json::json!([])));
    assert!(TypeSystem::value_to_logical(&serde_json::json!({})));
}
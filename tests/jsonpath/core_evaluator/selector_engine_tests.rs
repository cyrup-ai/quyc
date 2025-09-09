//! Tests for selector engine implementation
//! 
//! Extracted from src/jsonpath/core_evaluator/selector_engine.rs
//! Tests selector application engine

use serde_json::json;
use quyc_client::jsonpath::core_evaluator::selector_engine::SelectorEngine;
use quyc_client::jsonpath::parser::JsonSelector;

#[test]
fn test_root_selector() {
    let json = json!({"test": "value"});
    let selector = JsonSelector::Root;
    let results = SelectorEngine::apply_selector(&json, &selector)
        .expect("Failed to apply root selector");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json);
}

#[test]
fn test_child_selector() {
    let json = json!({"store": {"name": "test"}});
    let selector = JsonSelector::Child {
        name: "store".to_string(),
        quoted: false,
    };
    let results = SelectorEngine::apply_selector(&json, &selector)
        .expect("Failed to apply child selector");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], json!({"name": "test"}));
}

#[test]
fn test_wildcard_selector() {
    let json = json!({"a": 1, "b": 2, "c": 3});
    let selector = JsonSelector::Wildcard;
    let results = SelectorEngine::apply_selector(&json, &selector)
        .expect("Failed to apply wildcard selector");
    assert_eq!(results.len(), 3);
    assert!(results.contains(&json!(1)));
    assert!(results.contains(&json!(2)));
    assert!(results.contains(&json!(3)));
}

#[test]
fn test_selector_complexity() {
    assert_eq!(SelectorEngine::selector_complexity(&JsonSelector::Root), 1);
    assert_eq!(SelectorEngine::selector_complexity(&JsonSelector::Wildcard), 10);
    assert_eq!(SelectorEngine::selector_complexity(&JsonSelector::RecursiveDescent), 50);
}
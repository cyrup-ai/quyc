use quyc_client::jsonpath::normalized_paths::{NormalizedPath, NormalizedPathProcessor};

#[test]
fn test_root_path() {
    let root = NormalizedPath::root();
    assert_eq!(root.as_str(), "$");
    assert!(root.is_root());
    assert_eq!(root.depth(), 0);
    assert!(root.parent().is_none());
}

#[test]
fn test_member_path() {
    let root = NormalizedPath::root();
    let member_path = root
        .child_member("store")
        .expect("Failed to create child member path for 'store'");
    assert_eq!(member_path.as_str(), "$['store']");
    assert!(!member_path.is_root());
    assert_eq!(member_path.depth(), 1);
    assert_eq!(
        member_path
            .parent()
            .expect("Failed to get parent of member path")
            .as_str(),
        "$"
    );
}

#[test]
fn test_index_path() {
    let root = NormalizedPath::root();
    let member_path = root
        .child_member("items")
        .expect("Failed to create child member path for 'items'");
    let index_path = member_path
        .child_index(0)
        .expect("Failed to create child index path for index 0");
    assert_eq!(index_path.as_str(), "$['items'][0]");
    assert_eq!(index_path.depth(), 2);
}#[test]
fn test_complex_path() {
    let root = NormalizedPath::root();
    let complex = root
        .child_member("store")
        .expect("Failed to create child member path for 'store'")
        .child_member("book")
        .expect("Failed to create child member path for 'book'")
        .child_index(0)
        .expect("Failed to create child index path for index 0")
        .child_member("title")
        .expect("Failed to create child member path for 'title'");

    assert_eq!(complex.as_str(), "$['store']['book'][0]['title']");
    assert_eq!(complex.depth(), 4);
}

#[test]
fn test_parse_normalized_path() {
    let parsed =
        NormalizedPathProcessor::parse_normalized_path("$['store']['book'][0]['title']")
            .expect("Failed to parse normalized path expression");
    assert_eq!(parsed.as_str(), "$['store']['book'][0]['title']");
    assert_eq!(parsed.depth(), 4);
}

#[test]
fn test_path_relationships() {
    let parent = NormalizedPath::root()
        .child_member("store")
        .expect("Failed to create child member path for 'store'");
    let child = parent
        .child_member("book")
        .expect("Failed to create child member path for 'book'");

    assert!(child.is_descendant_of(&parent));
    assert!(parent.is_ancestor_of(&child));
    assert!(!parent.is_descendant_of(&child));
}

#[test]
fn test_invalid_paths() {
    // Negative index
    assert!(NormalizedPath::root().child_index(-1).is_err());

    // Invalid parse - no bracket notation
    assert!(NormalizedPathProcessor::parse_normalized_path("$.store").is_err());

    // Leading zeros
    assert!(NormalizedPathProcessor::parse_normalized_path("$[01]").is_err());
}
//! Recursive descent tests for JSONPath core evaluator
//!
//! Tests for recursive descent operator (..) with various selectors,
//! ensuring proper traversal and RFC 9535 compliance.

use serde_json::json;
use quyc::json_path::core_evaluator::evaluator::CoreJsonPathEvaluator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_descent() {
        // Use RFC 9535 compliant recursive descent with bracket selector
        let evaluator = CoreJsonPathEvaluator::new("$..[?@.author]")
            .expect("Failed to create evaluator for test");
        let json = json!({
            "store": {
                "book": [
                    {"author": "Author 1"},
                    {"author": "Author 2"}
                ]
            }
        });
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate recursive descent filter expression");

        // DEBUG: Print all results to understand what's being returned
        println!("=== DEBUG: Recursive descent results ===");
        println!("Total results: {}", results.len());
        for (i, result) in results.iter().enumerate() {
            let has_author = result.get("author").is_some();
            println!(
                "Result {}: has_author={}, value={:?}",
                i + 1,
                has_author,
                result
            );
        }

        // RFC-compliant filter returns only objects that have author property
        assert_eq!(results.len(), 2); // Only the 2 book objects that have author
        // Verify the book objects with authors are included
        assert!(
            results
                .iter()
                .any(|v| v.get("author").map_or(false, |a| a == "Author 1"))
        );
        assert!(
            results
                .iter()
                .any(|v| v.get("author").map_or(false, |a| a == "Author 2"))
        );
    }

    #[test]
    fn test_recursive_descent_fix() {
        println!("=== Testing recursive descent fix ===");

        let bookstore_json = json!({
            "store": {
                "book": [
                    {"author": "Author1", "title": "Book1"},
                    {"author": "Author2", "title": "Book2"}
                ],
                "bicycle": {"color": "red", "price": 19.95}
            }
        });

        // Test recursive descent for authors
        println!("Test: Recursive descent $..author");
        let evaluator = CoreJsonPathEvaluator::new("$..author")
            .expect("Failed to create evaluator for recursive descent '$..author'");
        let results = evaluator
            .evaluate(&bookstore_json)
            .expect("Failed to evaluate recursive descent against JSON");
        println!("$..author -> {} results: {:?}", results.len(), results);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&json!("Author1")));
        assert!(results.contains(&json!("Author2")));
    }

    #[test]
    fn test_recursive_descent_all_values() {
        let json = json!({
            "level1": {
                "level2": {
                    "target": "found",
                    "other": "value"
                },
                "target": "also_found"
            }
        });

        let evaluator = CoreJsonPathEvaluator::new("$..target")
            .expect("Failed to create evaluator for recursive descent");
        let results = evaluator
            .evaluate(&json)
            .expect("Failed to evaluate recursive descent");
        
        assert_eq!(results.len(), 2);
        assert!(results.contains(&json!("found")));
        assert!(results.contains(&json!("also_found")));
    }
}
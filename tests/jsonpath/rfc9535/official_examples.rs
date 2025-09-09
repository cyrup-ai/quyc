//! RFC 9535 Official Examples Tests (Section 1.5)
//!
//! Tests all official examples from RFC 9535 using the canonical bookstore JSON

use bytes::Bytes;
use quyc::jsonpath::JsonArrayStream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BookModel {
    category: String,
    author: String,
    title: String,
    price: f64,
    isbn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BicycleModel {
    color: String,
    price: f64,
}

/// Canonical RFC 9535 bookstore JSON data
const BOOKSTORE_JSON: &str = r#"{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century",
        "price": 8.95
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh", 
        "title": "Sword of Honour",
        "price": 12.99
      },
      {
        "category": "fiction",
        "author": "Herman Melville",
        "title": "Moby Dick",
        "isbn": "0-553-21311-3",
        "price": 8.99
      },
      {
        "category": "fiction",
        "author": "J. R. R. Tolkien",
        "title": "The Lord of the Rings",
        "isbn": "0-395-19395-8", 
        "price": 22.99
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95
    }
  }
}"#;

/// RFC 9535 Section 1.5 - Official Examples Tests
#[cfg(test)]
mod rfc_examples_tests {
    use super::*;

    #[test]
    fn test_all_book_authors() {
        // RFC 9535: $.store.book[*].author → All book authors
        let mut stream = JsonArrayStream::<String>::new("$.store.book[*].author");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        let expected_authors = vec![
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien",
        ];

        assert_eq!(results.len(), 4, "Should find all 4 book authors");
        for author in expected_authors {
            assert!(
                results.contains(&author.to_string()),
                "Should contain author: {}",
                author
            );
        }
    }

    #[test]
    fn test_all_authors_descendant() {
        // RFC 9535: $..author → All authors (descendant search)
        let mut stream = JsonArrayStream::<String>::new("$..author");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            4,
            "Descendant search should find all 4 authors"
        );

        let expected_authors = vec![
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien",
        ];

        for author in expected_authors {
            assert!(
                results.contains(&author.to_string()),
                "Descendant search should contain author: {}",
                author
            );
        }
    }

    #[test]
    fn test_all_things_in_store() {
        // RFC 9535: $.store.* → All things in store
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store.*");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            2,
            "Store should contain 2 things: book array and bicycle"
        );

        // Verify we get both the book array and bicycle object
        let mut has_book_array = false;
        let mut has_bicycle = false;

        for result in results {
            if result.is_array() {
                has_book_array = true;
            } else if result.is_object() && result.get("color").is_some() {
                has_bicycle = true;
            }
        }

        assert!(has_book_array, "Should include book array");
        assert!(has_bicycle, "Should include bicycle object");
    }

    #[test]
    fn test_all_prices_in_store() {
        // RFC 9535: $.store..price → All prices in store
        let mut stream = JsonArrayStream::<f64>::new("$.store..price");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            5,
            "Should find 5 prices (4 books + 1 bicycle)"
        );

        let expected_prices = vec![8.95, 12.99, 8.99, 22.99, 19.95];

        for price in expected_prices {
            assert!(results.contains(&price), "Should contain price: {}", price);
        }
    }

    #[test]
    fn test_third_book() {
        // RFC 9535: $..book[2] → Third book (0-indexed)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[2]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 1, "Should find exactly one book");
        assert_eq!(
            results[0].title, "Moby Dick",
            "Third book should be Moby Dick"
        );
        assert_eq!(
            results[0].author, "Herman Melville",
            "Author should be Herman Melville"
        );
    }

    #[test]
    fn test_last_book() {
        // RFC 9535: $..book[-1] → Last book
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[-1]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 1, "Should find exactly one book");
        assert_eq!(
            results[0].title, "The Lord of the Rings",
            "Last book should be LOTR"
        );
        assert_eq!(
            results[0].author, "J. R. R. Tolkien",
            "Author should be Tolkien"
        );
    }

    #[test]
    fn test_first_two_books_union() {
        // RFC 9535: $..book[0,1] → First two books (union selector)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[0,1]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 2, "Should find exactly two books");

        let titles: Vec<String> = results.iter().map(|book| book.title.clone()).collect();
        assert!(
            titles.contains(&"Sayings of the Century".to_string()),
            "Should contain first book"
        );
        assert!(
            titles.contains(&"Sword of Honour".to_string()),
            "Should contain second book"
        );
    }

    #[test]
    fn test_first_two_books_slice() {
        // RFC 9535: $..book[:2] → First two books (slice selector)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[:2]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 2, "Should find exactly two books");

        // Results should be in order
        assert_eq!(
            results[0].title, "Sayings of the Century",
            "First book should be first"
        );
        assert_eq!(
            results[1].title, "Sword of Honour",
            "Second book should be second"
        );
    }

    #[test]
    fn test_books_with_isbn() {
        // RFC 9535: $..book[?@.isbn] → Books with ISBN
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.isbn]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 2, "Should find 2 books with ISBN");

        let titles: Vec<String> = results.iter().map(|book| book.title.clone()).collect();
        assert!(
            titles.contains(&"Moby Dick".to_string()),
            "Moby Dick has ISBN"
        );
        assert!(
            titles.contains(&"The Lord of the Rings".to_string()),
            "LOTR has ISBN"
        );

        // Verify all results actually have ISBN
        for book in results {
            assert!(book.isbn.is_some(), "Book should have ISBN: {}", book.title);
        }
    }

    #[test]
    fn test_books_cheaper_than_10() {
        // RFC 9535: $..book[?@.price<10] → Books cheaper than 10
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.price<10]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 2, "Should find 2 books cheaper than 10");

        let expected_books = vec![("Sayings of the Century", 8.95), ("Moby Dick", 8.99)];

        for (title, price) in expected_books {
            let found = results
                .iter()
                .any(|book| book.title == title && book.price == price);
            assert!(found, "Should find book: {} at price: {}", title, price);
        }

        // Verify all results are actually cheaper than 10
        for book in results {
            assert!(
                book.price < 10.0,
                "Book should be <$10: {} at ${}",
                book.title,
                book.price
            );
        }
    }

    #[test]
    fn test_third_book_author() {
        // RFC 9535: $..book[2].author → Third book's author (Herman Melville)
        let mut stream = JsonArrayStream::<String>::new("$..book[2].author");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            1,
            "Should find exactly one author (third book's author)"
        );
        assert_eq!(
            results[0], "Herman Melville",
            "Third book's author should be Herman Melville"
        );
    }

    #[test]
    fn test_third_book_publisher() {
        // RFC 9535: $..book[2].publisher → Third book's publisher (should be empty - field doesn't exist)
        let mut stream = JsonArrayStream::<String>::new("$..book[2].publisher");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            0,
            "Should find no publisher (field doesn't exist in third book)"
        );
    }

    #[test]
    fn test_universal_descendant_search() {
        // RFC 9535: $..* → Universal descendant search (all values in the structure)
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..*");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 1000 {
                // Higher safety bound for universal search
                panic!("Too many results - likely infinite loop");
            }
        }

        // Universal descendant search should find many values throughout the structure
        // This includes: store object, book array, each book object, bicycle object,
        // and all primitive values within them
        assert!(
            results.len() >= 25,
            "Universal descendant search should find at least 25 values in bookstore JSON, found: {}",
            results.len()
        );

        // Verify we find some expected values
        let has_store_object = results
            .iter()
            .any(|v| v.is_object() && v.get("book").is_some());
        let has_book_array = results
            .iter()
            .any(|v| v.is_array() && v.as_array().unwrap().len() == 4);
        let has_book_titles = results
            .iter()
            .any(|v| v.is_string() && v.as_str().unwrap() == "Moby Dick");
        let has_prices = results
            .iter()
            .any(|v| v.is_number() && v.as_f64().unwrap() == 8.99);

        assert!(
            has_store_object,
            "Universal search should find store object"
        );
        assert!(has_book_array, "Universal search should find book array");
        assert!(has_book_titles, "Universal search should find book titles");
        assert!(has_prices, "Universal search should find price values");
    }
}

/// Extended RFC Examples with Complex Queries
#[cfg(test)]
mod extended_examples_tests {
    use super::*;

    #[test]
    fn test_fiction_books_only() {
        // Extension: Fiction books only
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.category=='fiction']");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 3, "Should find 3 fiction books");

        for book in results {
            assert_eq!(book.category, "fiction", "All books should be fiction");
        }
    }

    #[test]
    fn test_expensive_books() {
        // Extension: Expensive books (>$15)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.price>15]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 1, "Should find 1 expensive book");
        assert_eq!(
            results[0].title, "The Lord of the Rings",
            "LOTR should be the expensive book"
        );
        assert!(results[0].price > 15.0, "Book should be >$15");
    }

    #[test]
    fn test_books_by_author_pattern() {
        // Extension: Books by authors with specific name patterns
        let expressions = vec![
            ("$..book[?@.author=='Nigel Rees']", 1), // Exact match
            ("$..book[?@.author!='Nigel Rees']", 3), // Not equal
        ];

        for (expr, expected_count) in expressions {
            let mut stream = JsonArrayStream::<BookModel>::new(expr);

            let chunk = Bytes::from(BOOKSTORE_JSON);

            // Add timeout bounds to prevent infinite loops
            let timeout = std::time::Duration::from_secs(5);
            let start = std::time::Instant::now();
            let mut results = Vec::new();

            for result in stream.process_chunk(chunk).collect() {
                if start.elapsed() > timeout {
                    panic!(
                        "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                    );
                }
                results.push(result);
                if results.len() > 100 {
                    // Safety bound
                    panic!("Too many results - likely infinite loop");
                }
            }

            assert_eq!(
                results.len(),
                expected_count,
                "Author filter '{}' should return {} books",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_complex_logical_expressions() {
        // Extension: Complex logical expressions
        let expressions = vec![
            ("$..book[?@.category=='fiction' && @.price<15]", 2), // Fiction AND cheap
            ("$..book[?@.category=='reference' || @.isbn]", 3),   // Reference OR has ISBN
        ];

        for (expr, expected_count) in expressions {
            let mut stream = JsonArrayStream::<BookModel>::new(expr);

            let chunk = Bytes::from(BOOKSTORE_JSON);

            // Add timeout bounds to prevent infinite loops
            let timeout = std::time::Duration::from_secs(5);
            let start = std::time::Instant::now();
            let mut results = Vec::new();

            for result in stream.process_chunk(chunk).collect() {
                if start.elapsed() > timeout {
                    panic!(
                        "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                    );
                }
                results.push(result);
                if results.len() > 100 {
                    // Safety bound
                    panic!("Too many results - likely infinite loop");
                }
            }

            assert_eq!(
                results.len(),
                expected_count,
                "Complex filter '{}' should return {} books",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_bicycle_selection() {
        // Extension: Bicycle selection
        let mut stream = JsonArrayStream::<BicycleModel>::new("$.store.bicycle");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(results.len(), 1, "Should find the bicycle");
        assert_eq!(results[0].color, "red", "Bicycle should be red");
        assert_eq!(results[0].price, 19.95, "Bicycle should cost $19.95");
    }

    #[test]
    fn test_all_store_items_with_prices() {
        // Extension: All items in store that have a price
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store..*[?@.price]");

        let chunk = Bytes::from(BOOKSTORE_JSON);

        // Add timeout bounds to prevent infinite loops
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for result in stream.process_chunk(chunk).collect() {
            if start.elapsed() > timeout {
                panic!(
                    "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                );
            }
            results.push(result);
            if results.len() > 100 {
                // Safety bound
                panic!("Too many results - likely infinite loop");
            }
        }

        assert_eq!(
            results.len(),
            5,
            "Should find 5 items with prices (4 books + 1 bicycle)"
        );
    }
}

/// Performance Tests with RFC Examples
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_descendant_search_performance() {
        // Test performance of descendant searches
        let expressions = vec![
            "$..author",      // Deep descendant search
            "$.store..price", // Descendant with property
            "$..*",           // Universal descendant wildcard
        ];

        for expr in expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(BOOKSTORE_JSON);
            let start_time = std::time::Instant::now();

            // Add timeout bounds to prevent infinite loops
            let timeout = std::time::Duration::from_secs(5);
            let mut results = Vec::new();

            for result in stream.process_chunk(chunk).collect() {
                if start_time.elapsed() > timeout {
                    panic!(
                        "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                    );
                }
                results.push(result);
                if results.len() > 100 {
                    // Safety bound
                    panic!("Too many results - likely infinite loop");
                }
            }

            let duration = start_time.elapsed();

            // Performance assertion - should complete quickly on small dataset
            assert!(
                duration.as_millis() < 100,
                "Expression '{}' should complete in <100ms",
                expr
            );
        }
    }

    #[test]
    fn test_filter_expression_performance() {
        // Test performance of filter expressions
        let filter_expressions = vec![
            "$..book[?@.price<10]",
            "$..book[?@.category=='fiction']",
            "$..book[?@.isbn]",
            "$..book[?@.category=='fiction' && @.price<15]",
        ];

        for expr in filter_expressions {
            let mut stream = JsonArrayStream::<BookModel>::new(expr);

            let chunk = Bytes::from(BOOKSTORE_JSON);
            let start_time = std::time::Instant::now();

            // Add timeout bounds to prevent infinite loops
            let timeout = std::time::Duration::from_secs(5);
            let mut results = Vec::new();

            for result in stream.process_chunk(chunk).collect() {
                if start_time.elapsed() > timeout {
                    panic!(
                        "Test timed out after 5 seconds - likely infinite loop in JSONPath processing"
                    );
                }
                results.push(result);
                if results.len() > 100 {
                    // Safety bound
                    panic!("Too many results - likely infinite loop");
                }
            }

            let duration = start_time.elapsed();

            // Performance assertion
            assert!(
                duration.as_millis() < 50,
                "Filter '{}' should complete in <50ms",
                expr
            );
        }
    }
}

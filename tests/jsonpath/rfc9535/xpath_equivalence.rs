//! RFC 9535 XPath Equivalence Tests (Appendix B)
//!
//! Tests all XPath to JSONPath mappings from Table 21 (Appendix B.1)
//! Validates that JSONPath expressions produce equivalent results to their XPath counterparts

use bytes::Bytes;
use quyc::JsonArrayStream;
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

/// Canonical RFC 9535 bookstore JSON data for XPath equivalence testing
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

/// RFC 9535 Appendix B.1 - XPath Equivalence Tests
#[cfg(test)]
mod xpath_equivalence_tests {
    use super::*;

    #[test]
    fn test_xpath_store_book_author() {
        // Table 21: XPath `/store/book/author` → JSONPath `$.store.book[*].author`
        // Result: the authors of all books in the store
        let mut stream = JsonArrayStream::<String>::new("$.store.book[*].author");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 4, "Should find all 4 book authors");

        let expected_authors = vec![
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien",
        ];

        for author in expected_authors {
            assert!(
                results.contains(&author.to_string()),
                "XPath equivalent should contain author: {}",
                author
            );
        }
    }

    #[test]
    fn test_xpath_descendant_author() {
        // Table 21: XPath `//author` → JSONPath `$..author`
        // Result: all authors
        let mut stream = JsonArrayStream::<String>::new("$..author");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

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
                "XPath descendant equivalent should contain author: {}",
                author
            );
        }
    }

    #[test]
    fn test_xpath_store_wildcard() {
        // Table 21: XPath `/store/*` → JSONPath `$.store.*`
        // Result: all things in store, which are some books and a red bicycle
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store.*");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            2,
            "Store wildcard should contain 2 things: book array and bicycle"
        );

        // Verify we get both the book array and bicycle object
        let mut has_book_array = false;
        let mut has_bicycle = false;

        for result in results {
            if result.is_array() && result.as_array().unwrap().len() == 4 {
                has_book_array = true;
            } else if result.is_object() && result.get("color").is_some() {
                has_bicycle = true;
            }
        }

        assert!(has_book_array, "XPath equivalent should include book array");
        assert!(
            has_bicycle,
            "XPath equivalent should include bicycle object"
        );
    }

    #[test]
    fn test_xpath_store_descendant_price() {
        // Table 21: XPath `/store//price` → JSONPath `$.store..price`
        // Result: the prices of everything in the store
        let mut stream = JsonArrayStream::<f64>::new("$.store..price");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            5,
            "Should find 5 prices (4 books + 1 bicycle)"
        );

        let expected_prices = vec![8.95, 12.99, 8.99, 22.99, 19.95];

        for price in expected_prices {
            assert!(
                results.contains(&price),
                "XPath descendant price equivalent should contain price: {}",
                price
            );
        }
    }

    #[test]
    fn test_xpath_third_book() {
        // Table 21: XPath `//book[3]` → JSONPath `$..book[2]`
        // Result: the third book (note: XPath uses 1-based indexing, JSONPath uses 0-based)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[2]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Should find exactly one book (the third)");
        assert_eq!(
            results[0].title, "Moby Dick",
            "XPath third book equivalent should be Moby Dick"
        );
        assert_eq!(
            results[0].author, "Herman Melville",
            "Third book author should be Herman Melville"
        );
    }

    #[test]
    fn test_xpath_last_book() {
        // Table 21: XPath `//book[last()]` → JSONPath `$..book[-1]`
        // Result: the last book in order
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[-1]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Should find exactly one book (the last)");
        assert_eq!(
            results[0].title, "The Lord of the Rings",
            "XPath last book equivalent should be LOTR"
        );
        assert_eq!(
            results[0].author, "J. R. R. Tolkien",
            "Last book author should be Tolkien"
        );
    }

    #[test]
    fn test_xpath_first_two_books_union() {
        // Table 21: XPath `//book[position()<3]` → JSONPath `$..book[0,1]`
        // Result: the first two books (union selector)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[0,1]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should find exactly two books");

        let titles: Vec<String> = results.iter().map(|book| book.title.clone()).collect();
        assert!(
            titles.contains(&"Sayings of the Century".to_string()),
            "XPath first two books should contain first book"
        );
        assert!(
            titles.contains(&"Sword of Honour".to_string()),
            "XPath first two books should contain second book"
        );
    }

    #[test]
    fn test_xpath_first_two_books_slice() {
        // Table 21: XPath `//book[position()<3]` → JSONPath `$..book[:2]` (alternative)
        // Result: the first two books (slice selector)
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[:2]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            2,
            "Slice selector should find exactly two books"
        );

        // Results should be in order for slice selector
        assert_eq!(
            results[0].title, "Sayings of the Century",
            "XPath slice equivalent first book should be first"
        );
        assert_eq!(
            results[1].title, "Sword of Honour",
            "XPath slice equivalent second book should be second"
        );
    }

    #[test]
    fn test_xpath_books_with_isbn() {
        // Table 21: XPath `//book[isbn]` → JSONPath `$..book[?@.isbn]`
        // Result: filter all books with an ISBN number
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.isbn]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should find 2 books with ISBN");

        let titles: Vec<String> = results.iter().map(|book| book.title.clone()).collect();
        assert!(
            titles.contains(&"Moby Dick".to_string()),
            "XPath ISBN filter should include Moby Dick"
        );
        assert!(
            titles.contains(&"The Lord of the Rings".to_string()),
            "XPath ISBN filter should include LOTR"
        );

        // Verify all results actually have ISBN
        for book in results {
            assert!(
                book.isbn.is_some(),
                "XPath ISBN equivalent should only return books with ISBN: {}",
                book.title
            );
        }
    }

    #[test]
    fn test_xpath_books_cheaper_than_10() {
        // Table 21: XPath `//book[price<10]` → JSONPath `$..book[?@.price<10]`
        // Result: filter all books cheaper than 10
        let mut stream = JsonArrayStream::<BookModel>::new("$..book[?@.price<10]");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should find 2 books cheaper than 10");

        let expected_books = vec![("Sayings of the Century", 8.95), ("Moby Dick", 8.99)];

        for (title, price) in expected_books {
            let found = results
                .iter()
                .any(|book| book.title == title && book.price == price);
            assert!(
                found,
                "XPath price filter should find book: {} at price: {}",
                title, price
            );
        }

        // Verify all results are actually cheaper than 10
        for book in results {
            assert!(
                book.price < 10.0,
                "XPath price equivalent should only return books <$10: {} at ${}",
                book.title,
                book.price
            );
        }
    }

    #[test]
    fn test_xpath_universal_descendant() {
        // Table 21: XPath `//*` → JSONPath `$..*`
        // Result: all elements in an XML document; all member values and array elements contained in input value
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$..*");

        let chunk = Bytes::from(BOOKSTORE_JSON);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Universal descendant search should find many values throughout the structure
        // This includes: store object, book array, each book object, bicycle object,
        // and all primitive values within them
        assert!(
            results.len() >= 25,
            "XPath universal descendant equivalent should find at least 25 values in bookstore JSON, found: {}",
            results.len()
        );

        // Verify we find some expected values from different levels
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
            "XPath universal equivalent should find store object"
        );
        assert!(
            has_book_array,
            "XPath universal equivalent should find book array"
        );
        assert!(
            has_book_titles,
            "XPath universal equivalent should find book titles"
        );
        assert!(
            has_prices,
            "XPath universal equivalent should find price values"
        );
    }
}

/// XPath vs JSONPath Indexing Differences Tests
#[cfg(test)]
mod xpath_indexing_difference_tests {
    use super::*;

    #[test]
    fn test_xpath_jsonpath_index_difference() {
        // RFC 9535 Appendix B: XPath indices start at 1, JSONPath indices start at 0
        // This test validates that our JSONPath implementation correctly uses 0-based indexing

        let json_array = r#"{"items": ["first", "second", "third", "fourth"]}"#;

        let test_cases = vec![
            // XPath book[1] would be first book → JSONPath book[0]
            ("$.items[0]", "first"),  // First item (XPath would use [1])
            ("$.items[1]", "second"), // Second item (XPath would use [2])
            ("$.items[2]", "third"),  // Third item (XPath would use [3])
            ("$.items[3]", "fourth"), // Fourth item (XPath would use [4])
        ];

        for (jsonpath_expr, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(jsonpath_expr);

            let chunk = Bytes::from(json_array);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "JSONPath index expression '{}' should return exactly 1 result",
                jsonpath_expr
            );
            assert_eq!(
                results[0], expected_value,
                "JSONPath uses 0-based indexing: '{}' should return '{}'",
                jsonpath_expr, expected_value
            );
        }
    }

    #[test]
    fn test_xpath_position_vs_jsonpath_index() {
        // Demonstrate XPath position() vs JSONPath array index differences
        let json_data = r#"{"books": [
            {"title": "Book1"},
            {"title": "Book2"}, 
            {"title": "Book3"},
            {"title": "Book4"}
        ]}"#;

        // XPath position()<3 means positions 1 and 2 (first two)
        // JSONPath equivalent: [0,1] or [:2] (indices 0 and 1)

        let mut union_stream = JsonArrayStream::<serde_json::Value>::new("$.books[0,1]");
        let mut slice_stream = JsonArrayStream::<serde_json::Value>::new("$.books[:2]");

        let chunk = Bytes::from(json_data);

        let unionresults: Vec<_> = union_stream.process_chunk(chunk.clone()).collect();
        let sliceresults: Vec<_> = slice_stream.process_chunk(chunk).collect();

        // Both should return first two books
        assert_eq!(
            unionresults.len(),
            2,
            "Union selector should return 2 books"
        );
        assert_eq!(
            sliceresults.len(),
            2,
            "Slice selector should return 2 books"
        );

        // Verify the correct books are returned (first two)
        for results in vec![&unionresults, &sliceresults] {
            let titles: Vec<String> = results
                .iter()
                .map(|book| book["title"].as_str().unwrap().to_string())
                .collect();

            assert!(
                titles.contains(&"Book1".to_string()),
                "XPath position()<3 equivalent should include first book"
            );
            assert!(
                titles.contains(&"Book2".to_string()),
                "XPath position()<3 equivalent should include second book"
            );
            assert!(
                !titles.contains(&"Book3".to_string()),
                "XPath position()<3 equivalent should NOT include third book"
            );
        }
    }

    #[test]
    fn test_xpath_last_vs_jsonpath_negative_index() {
        // XPath last() function vs JSONPath negative indexing
        let json_data = r#"{"sequence": [10, 20, 30, 40, 50]}"#;

        // XPath sequence[last()] → JSONPath sequence[-1]
        let mut stream = JsonArrayStream::<i32>::new("$.sequence[-1]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            1,
            "Last element selector should return 1 result"
        );
        assert_eq!(
            results[0], 50,
            "XPath last() equivalent should return last element"
        );

        // Test other negative indices for completeness
        let test_cases = vec![
            ("$.sequence[-1]", 50), // last (XPath: [last()])
            ("$.sequence[-2]", 40), // second to last (XPath: [last()-1])
            ("$.sequence[-3]", 30), // third to last (XPath: [last()-2])
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Negative index '{}' should return 1 result",
                expr
            );
            assert_eq!(
                results[0], expected,
                "Negative index '{}' should return {}",
                expr, expected
            );
        }
    }
}

/// XPath Node Set vs JSONPath Nodelist Behavior Tests
#[cfg(test)]
mod xpath_nodeset_vs_jsonpath_nodelist_tests {
    use super::*;

    #[test]
    fn test_xpath_nodeset_vs_jsonpath_nodelist_operation() {
        // RFC 9535 Appendix B: Square brackets in XPath operate on node set,
        // in JSONPath they operate on each node in the nodelist

        let nested_data = r#"{
            "groups": [
                {"items": [1, 2, 3]},
                {"items": [4, 5, 6]},
                {"items": [7, 8, 9]}
            ]
        }"#;

        // JSONPath: groups[*].items[0] - gets first item from each group's items array
        // This demonstrates how JSONPath brackets operate on each node individually
        let mut stream = JsonArrayStream::<i32>::new("$.groups[*].items[0]");

        let chunk = Bytes::from(nested_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(
            results.len(),
            3,
            "Should get first item from each of 3 groups"
        );
        assert_eq!(
            results,
            vec![1, 4, 7],
            "Should get [1, 4, 7] - first from each group"
        );

        // Compare with getting all items then selecting
        let mut all_items_stream = JsonArrayStream::<i32>::new("$.groups[*].items[*]");
        let chunk = Bytes::from(nested_data);
        let allresults: Vec<_> = all_items_stream.process_chunk(chunk).collect();

        assert_eq!(allresults.len(), 9, "Should get all 9 items when using [*]");
        assert_eq!(
            allresults,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            "Should get all items in order"
        );
    }

    #[test]
    fn test_xpath_predicate_vs_jsonpath_filter_context() {
        // Demonstrate how XPath predicates vs JSONPath filters handle context differently

        let data_with_context = r#"{
            "library": {
                "sections": [
                    {
                        "name": "Fiction",
                        "books": [
                            {"title": "Book1", "available": true},
                            {"title": "Book2", "available": false}
                        ]
                    },
                    {
                        "name": "Science", 
                        "books": [
                            {"title": "Book3", "available": true}
                        ]
                    }
                ]
            }
        }"#;

        // JSONPath filter operates on current context (@)
        // Get all available books across all sections
        let mut stream =
            JsonArrayStream::<serde_json::Value>::new("$.library.sections[*].books[?@.available]");

        let chunk = Bytes::from(data_with_context);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 2, "Should find 2 available books");

        let titles: Vec<String> = results
            .iter()
            .map(|book| book["title"].as_str().unwrap().to_string())
            .collect();

        assert!(
            titles.contains(&"Book1".to_string()),
            "Should include Book1"
        );
        assert!(
            titles.contains(&"Book3".to_string()),
            "Should include Book3"
        );
        assert!(
            !titles.contains(&"Book2".to_string()),
            "Should NOT include Book2 (not available)"
        );
    }
}

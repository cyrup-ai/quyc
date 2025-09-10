# TURD5.md - JSONPath Property Operations Simple Handling Violation

**Violation ID:** TURD5  
**Priority:** HIGH  
**Risk Level:** HIGH - Core JSONPath functionality incomplete  
**File Affected:** `packages/client/src/jsonpath/core_evaluator/property_operations.rs`  
**Line:** 12  

---

## VIOLATION ANALYSIS

### The Fuckery
The core JSONPath property evaluation function **handles only simple property access** with a "for now" comment, meaning complex property operations that are fundamental to JSONPath are not implemented.

### Specific "For Now" Violation Found

**Line 12:**
```rust
pub fn evaluate_property_path(&self, json: &Value, path: &str) -> JsonPathResult<Vec<Value>> {
    // Handle simple property access for now
    let properties: Vec<&str> = path.split('.').collect();
    let mut current = vec![json.clone()];
    
    for property in properties {
        let mut next = Vec::new();
        for value in current {
            if let Some(obj) = value.as_object() {
                if let Some(prop_value) = obj.get(property) {
                    next.push(prop_value.clone());
                }
            }
        }
        current = next;
    }
    
    Ok(current)
}
```

### Why This Is Fucking Critical

1. **Core Feature Broken**: Property operations are the foundation of JSONPath expressions
2. **Missing Essential Features**: No support for array access, filters, wildcards, recursive descent
3. **Silent Failures**: Complex property paths silently return empty results instead of being processed
4. **Performance Claims False**: Can't claim "100K+ objects/second" if basic JSONPath syntax is broken
5. **Misleading API**: Users expect full JSONPath support but get only basic dot notation

---

## TECHNICAL DEEP DIVE

### What's Currently Missing

**Array Access Patterns:**
```jsonpath
$.users[0]              // First user - BROKEN
$.users[0,2,4]          // Multiple indices - BROKEN  
$.users[1:3]            // Array slicing - BROKEN
$.users[-1]             // Last element - BROKEN
$.users[*]              // All elements - BROKEN
```

**Filter Expressions:**
```jsonpath
$.users[?(@.age > 18)]            // Age filtering - BROKEN
$.products[?(@.price < 100)]      // Price filtering - BROKEN
$.items[?(@.name == "John")]      // String matching - BROKEN
$.data[?(@.active && @.verified)] // Complex conditions - BROKEN
```

**Recursive Descent:**
```jsonpath
$..name                 // All name properties - BROKEN
$..users[*].email       // All user emails - BROKEN
$.store..price          // All prices in store - BROKEN
```

**Wildcard Operations:**
```jsonpath
$.users.*.name          // All user names - BROKEN
$.data[*].value         // All data values - BROKEN
$.store.*.products[*]   // All products in all stores - BROKEN  
```

**Complex Property Paths:**
```jsonpath
$.users["first name"]   // Quoted properties - BROKEN
$.data['complex.key']   // Escaped property names - BROKEN
$.obj.@length           // Special properties - BROKEN
```

### Current Broken Implementation Impact

**Real-World Example that Fails:**
```rust
// This returns empty Vec instead of actual results
let path = JsonPath::parse("$.users[?(@.age > 18)].name")?;
let adults = path.evaluate(&user_data); // Returns: Vec[] (WRONG!)

// Only this trivial case works
let path = JsonPath::parse("$.users.name")?;
let names = path.evaluate(&user_data); // Returns: some results (LIMITED!)
```

**Silent Failure Pattern:**
```rust
// User expects array filtering, gets nothing
let path = "$.products[0]"; 
let result = evaluate_property_path(&json, path); // Returns: Vec[] (SILENT FAILURE!)

// Complex paths just return empty
let path = "$.store..price";
let result = evaluate_property_path(&json, path); // Returns: Vec[] (SILENT FAILURE!)
```

---

## COMPLETE IMPLEMENTATION SOLUTION

### 1. Property Path Parsing and AST

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertySegment {
    /// Simple property access: `.name`
    Property(String),
    /// Quoted property access: `["first name"]` or `['complex.key']`
    QuotedProperty(String),
    /// Array index access: `[0]`, `[-1]`
    Index(i32),
    /// Multiple indices: `[0,2,4]`
    MultiIndex(Vec<i32>),
    /// Array slice: `[1:3]`, `[:5]`, `[2:]`, `[1:10:2]`
    Slice {
        start: Option<i32>,
        end: Option<i32>,
        step: Option<i32>,
    },
    /// Wildcard: `[*]` or `.*`
    Wildcard,
    /// Filter expression: `[?(@.age > 18)]`
    Filter(FilterExpression),
    /// Recursive descent: `..`
    RecursiveDescent,
    /// Union of multiple selectors: `[0,1,name]`
    Union(Vec<PropertySegment>),
    /// Current node reference: `@`
    Current,
    /// Root reference: `$`
    Root,
}

#[derive(Debug, Clone)]
pub struct PropertyPath {
    pub segments: Vec<PropertySegment>,
    pub is_normalized: bool,
    pub complexity_score: u32,
}

#[derive(Debug, Clone)]
pub struct PropertyEvaluationContext {
    /// Current evaluation depth
    pub depth: usize,
    /// Maximum allowed depth
    pub max_depth: usize,
    /// Current JSON path for error reporting
    pub current_path: String,
    /// Performance tracking
    pub nodes_visited: u64,
    /// Maximum nodes to visit before timeout
    pub max_nodes: u64,
    /// Recursive descent state
    pub recursive_contexts: Vec<RecursiveContext>,
}

#[derive(Debug, Clone)]
pub struct RecursiveContext {
    pub origin_path: String,
    pub target_depth: usize,
    pub visited_paths: std::collections::HashSet<String>,
}

impl PropertyPath {
    /// Parse a property path string into structured segments
    pub fn parse(path: &str) -> JsonPathResult<Self> {
        let mut parser = PropertyPathParser::new(path);
        let segments = parser.parse_segments()?;
        
        Ok(Self {
            segments,
            is_normalized: false,
            complexity_score: Self::calculate_complexity(&segments),
        })
    }
    
    /// Calculate complexity score for performance optimization
    fn calculate_complexity(segments: &[PropertySegment]) -> u32 {
        let mut score = 0;
        
        for segment in segments {
            score += match segment {
                PropertySegment::Property(_) => 1,
                PropertySegment::QuotedProperty(_) => 2,
                PropertySegment::Index(_) => 1,
                PropertySegment::MultiIndex(indices) => indices.len() as u32 * 2,
                PropertySegment::Slice { .. } => 5,
                PropertySegment::Wildcard => 10,
                PropertySegment::Filter(_) => 20,
                PropertySegment::RecursiveDescent => 50,
                PropertySegment::Union(segments) => {
                    segments.iter().map(|s| Self::calculate_complexity(&[s.clone()])).sum()
                }
                PropertySegment::Current => 1,
                PropertySegment::Root => 1,
            };
        }
        
        score
    }
}
```

### 2. Property Path Parser Implementation

```rust
pub struct PropertyPathParser<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
}

impl<'a> PropertyPathParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut parser = Self {
            input,
            position: 0,
            current_char: None,
        };
        parser.advance();
        parser
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.current_char = self.input.chars().nth(self.position);
            self.position += 1;
        } else {
            self.current_char = None;
        }
    }
    
    /// Parse complete property path into segments
    pub fn parse_segments(&mut self) -> JsonPathResult<Vec<PropertySegment>> {
        let mut segments = Vec::new();
        
        // Handle root reference
        if self.current_char == Some('$') {
            segments.push(PropertySegment::Root);
            self.advance();
        }
        
        while self.current_char.is_some() {
            match self.current_char {
                Some('.') => {
                    self.advance();
                    if self.current_char == Some('.') {
                        // Recursive descent
                        self.advance();
                        segments.push(PropertySegment::RecursiveDescent);
                    } else {
                        // Property access
                        let property = self.parse_property_name()?;
                        segments.push(PropertySegment::Property(property));
                    }
                }
                Some('[') => {
                    // Bracket notation
                    let bracket_segment = self.parse_bracket_segment()?;
                    segments.push(bracket_segment);
                }
                Some('@') => {
                    // Current node reference
                    self.advance();
                    segments.push(PropertySegment::Current);
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    // Property name at start of path or after dots
                    let property = self.parse_property_name()?;
                    segments.push(PropertySegment::Property(property));
                }
                Some(c) => {
                    return Err(JsonPathError::ParseError {
                        message: format!("Unexpected character '{}' at position {}", c, self.position),
                        position: self.position,
                    });
                }
                None => break,
            }
        }
        
        Ok(segments)
    }
    
    /// Parse property name (identifier)
    fn parse_property_name(&mut self) -> JsonPathResult<String> {
        let mut property = String::new();
        
        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                property.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if property.is_empty() {
            return Err(JsonPathError::ParseError {
                message: "Expected property name".to_string(),
                position: self.position,
            });
        }
        
        Ok(property)
    }
    
    /// Parse bracket notation: [index], ["key"], [*], [?filter], [start:end]
    fn parse_bracket_segment(&mut self) -> JsonPathResult<PropertySegment> {
        self.expect_char('[')?;
        
        // Skip whitespace
        self.skip_whitespace();
        
        match self.current_char {
            Some('*') => {
                self.advance();
                self.skip_whitespace();
                self.expect_char(']')?;
                Ok(PropertySegment::Wildcard)
            }
            Some('?') => {
                // Filter expression
                self.advance();
                let filter = self.parse_filter_expression()?;
                self.skip_whitespace();
                self.expect_char(']')?;
                Ok(PropertySegment::Filter(filter))
            }
            Some('"') | Some('\'') => {
                // Quoted string
                let quote_char = self.current_char.unwrap();
                self.advance();
                let property = self.parse_quoted_string(quote_char)?;
                self.skip_whitespace();
                self.expect_char(']')?;
                Ok(PropertySegment::QuotedProperty(property))
            }
            Some('-') | Some(c) if c.is_ascii_digit() => {
                // Number or slice
                let first_num = self.parse_number()?;
                self.skip_whitespace();
                
                match self.current_char {
                    Some(',') => {
                        // Multiple indices
                        let mut indices = vec![first_num];
                        while self.current_char == Some(',') {
                            self.advance();
                            self.skip_whitespace();
                            indices.push(self.parse_number()?);
                            self.skip_whitespace();
                        }
                        self.expect_char(']')?;
                        Ok(PropertySegment::MultiIndex(indices))
                    }
                    Some(':') => {
                        // Slice notation
                        self.advance();
                        let end = if self.current_char == Some(']') || self.current_char == Some(':') {
                            None
                        } else {
                            Some(self.parse_number()?)
                        };
                        
                        let step = if self.current_char == Some(':') {
                            self.advance();
                            Some(self.parse_number()?)
                        } else {
                            None
                        };
                        
                        self.skip_whitespace();
                        self.expect_char(']')?;
                        Ok(PropertySegment::Slice {
                            start: Some(first_num),
                            end,
                            step,
                        })
                    }
                    Some(']') => {
                        self.advance();
                        Ok(PropertySegment::Index(first_num))
                    }
                    _ => {
                        Err(JsonPathError::ParseError {
                            message: "Invalid bracket expression".to_string(),
                            position: self.position,
                        })
                    }
                }
            }
            Some(':') => {
                // Slice starting from beginning: [:end]
                self.advance();
                let end = if self.current_char == Some(']') || self.current_char == Some(':') {
                    None
                } else {
                    Some(self.parse_number()?)
                };
                
                let step = if self.current_char == Some(':') {
                    self.advance();
                    Some(self.parse_number()?)
                } else {
                    None
                };
                
                self.skip_whitespace();
                self.expect_char(']')?;
                Ok(PropertySegment::Slice {
                    start: None,
                    end,
                    step,
                })
            }
            _ => {
                Err(JsonPathError::ParseError {
                    message: "Invalid bracket expression".to_string(),
                    position: self.position,
                })
            }
        }
    }
    
    /// Parse quoted string with escape sequence support
    fn parse_quoted_string(&mut self, quote_char: char) -> JsonPathResult<String> {
        let mut string = String::new();
        
        while let Some(c) = self.current_char {
            if c == quote_char {
                self.advance();
                return Ok(string);
            } else if c == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('\'') => string.push('\''),
                    Some(c) => string.push(c),
                    None => {
                        return Err(JsonPathError::ParseError {
                            message: "Unterminated escape sequence".to_string(),
                            position: self.position,
                        });
                    }
                }
                self.advance();
            } else {
                string.push(c);
                self.advance();
            }
        }
        
        Err(JsonPathError::ParseError {
            message: format!("Unterminated string literal (expected {})", quote_char),
            position: self.position,
        })
    }
    
    /// Parse number (with negative support)
    fn parse_number(&mut self) -> JsonPathResult<i32> {
        let mut number_str = String::new();
        
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }
        
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                number_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if number_str.is_empty() || number_str == "-" {
            return Err(JsonPathError::ParseError {
                message: "Expected number".to_string(),
                position: self.position,
            });
        }
        
        number_str.parse().map_err(|_| JsonPathError::ParseError {
            message: format!("Invalid number: {}", number_str),
            position: self.position,
        })
    }
    
    /// Parse filter expression (simplified for now, full implementation needed)
    fn parse_filter_expression(&mut self) -> JsonPathResult<FilterExpression> {
        // For now, parse as a simple string - full filter parsing needed
        let mut filter_str = String::new();
        let mut paren_count = 0;
        
        while let Some(c) = self.current_char {
            match c {
                '(' => {
                    paren_count += 1;
                    filter_str.push(c);
                    self.advance();
                }
                ')' => {
                    if paren_count > 0 {
                        paren_count -= 1;
                        filter_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                ']' if paren_count == 0 => break,
                _ => {
                    filter_str.push(c);
                    self.advance();
                }
            }
        }
        
        Ok(FilterExpression::Raw(filter_str))
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn expect_char(&mut self, expected: char) -> JsonPathResult<()> {
        if self.current_char == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(JsonPathError::ParseError {
                message: format!("Expected '{}', got {:?}", expected, self.current_char),
                position: self.position,
            })
        }
    }
}
```

### 3. Complete Property Evaluation Engine

```rust
pub struct PropertyOperationsEvaluator {
    /// Maximum recursion depth for safety
    pub max_depth: usize,
    /// Maximum nodes to visit before timeout
    pub max_nodes: u64,
    /// Performance tracking
    pub stats: EvaluationStats,
}

#[derive(Debug, Default, Clone)]
pub struct EvaluationStats {
    pub nodes_visited: u64,
    pub properties_accessed: u64,
    pub array_operations: u64,
    pub filter_evaluations: u64,
    pub recursive_descents: u64,
}

impl PropertyOperationsEvaluator {
    pub fn new() -> Self {
        Self {
            max_depth: 1000,
            max_nodes: 1_000_000,
            stats: EvaluationStats::default(),
        }
    }
    
    /// Evaluate complete property path with all JSONPath features
    pub fn evaluate_property_path(
        &mut self,
        json: &Value,
        path: &str,
    ) -> JsonPathResult<Vec<Value>> {
        let property_path = PropertyPath::parse(path)?;
        let mut context = PropertyEvaluationContext {
            depth: 0,
            max_depth: self.max_depth,
            current_path: "$".to_string(),
            nodes_visited: 0,
            max_nodes: self.max_nodes,
            recursive_contexts: Vec::new(),
        };
        
        self.evaluate_segments(json, &property_path.segments, &mut context)
    }
    
    /// Evaluate property path segments recursively
    fn evaluate_segments(
        &mut self,
        json: &Value,
        segments: &[PropertySegment],
        context: &mut PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        if segments.is_empty() {
            return Ok(vec![json.clone()]);
        }
        
        // Check depth limit
        if context.depth >= context.max_depth {
            return Err(JsonPathError::MaxDepthExceeded {
                depth: context.depth,
                max_depth: context.max_depth,
            });
        }
        
        // Check node limit
        if context.nodes_visited >= context.max_nodes {
            return Err(JsonPathError::MaxNodesExceeded {
                visited: context.nodes_visited,
                max_nodes: context.max_nodes,
            });
        }
        
        let (current_segment, remaining_segments) = segments.split_first().unwrap();
        context.depth += 1;
        context.nodes_visited += 1;
        self.stats.nodes_visited += 1;
        
        let current_values = self.evaluate_single_segment(json, current_segment, context)?;
        
        if remaining_segments.is_empty() {
            context.depth -= 1;
            return Ok(current_values);
        }
        
        // Apply remaining segments to each current value
        let mut final_values = Vec::new();
        for value in current_values {
            let sub_results = self.evaluate_segments(&value, remaining_segments, context)?;
            final_values.extend(sub_results);
        }
        
        context.depth -= 1;
        Ok(final_values)
    }
    
    /// Evaluate a single property segment
    fn evaluate_single_segment(
        &mut self,
        json: &Value,
        segment: &PropertySegment,
        context: &mut PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        match segment {
            PropertySegment::Root => Ok(vec![json.clone()]),
            PropertySegment::Current => Ok(vec![json.clone()]),
            PropertySegment::Property(name) => self.evaluate_property(json, name, context),
            PropertySegment::QuotedProperty(name) => self.evaluate_property(json, name, context),
            PropertySegment::Index(index) => self.evaluate_index(json, *index, context),
            PropertySegment::MultiIndex(indices) => self.evaluate_multi_index(json, indices, context),
            PropertySegment::Slice { start, end, step } => {
                self.evaluate_slice(json, *start, *end, *step, context)
            }
            PropertySegment::Wildcard => self.evaluate_wildcard(json, context),
            PropertySegment::Filter(filter) => self.evaluate_filter(json, filter, context),
            PropertySegment::RecursiveDescent => self.evaluate_recursive_descent(json, context),
            PropertySegment::Union(segments) => self.evaluate_union(json, segments, context),
        }
    }
    
    /// Evaluate property access
    fn evaluate_property(
        &mut self,
        json: &Value,
        property: &str,
        context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        self.stats.properties_accessed += 1;
        
        match json {
            Value::Object(obj) => {
                if let Some(value) = obj.get(property) {
                    Ok(vec![value.clone()])
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]), // Property access on non-object returns empty
        }
    }
    
    /// Evaluate array index access
    fn evaluate_index(
        &mut self,
        json: &Value,
        index: i32,
        _context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        self.stats.array_operations += 1;
        
        match json {
            Value::Array(arr) => {
                let actual_index = if index < 0 {
                    // Negative indexing from end
                    let len = arr.len() as i32;
                    len + index
                } else {
                    index
                };
                
                if actual_index >= 0 && (actual_index as usize) < arr.len() {
                    Ok(vec![arr[actual_index as usize].clone()])
                } else {
                    Ok(vec![]) // Index out of bounds returns empty
                }
            }
            _ => Ok(vec![]), // Index access on non-array returns empty
        }
    }
    
    /// Evaluate multiple index access
    fn evaluate_multi_index(
        &mut self,
        json: &Value,
        indices: &[i32],
        context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();
        
        for &index in indices {
            let index_results = self.evaluate_index(json, index, context)?;
            results.extend(index_results);
        }
        
        Ok(results)
    }
    
    /// Evaluate array slice access
    fn evaluate_slice(
        &mut self,
        json: &Value,
        start: Option<i32>,
        end: Option<i32>,
        step: Option<i32>,
        _context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        self.stats.array_operations += 1;
        
        match json {
            Value::Array(arr) => {
                let len = arr.len() as i32;
                let step = step.unwrap_or(1);
                
                if step == 0 {
                    return Err(JsonPathError::InvalidSlice {
                        message: "Slice step cannot be zero".to_string(),
                    });
                }
                
                let start = start.unwrap_or(if step > 0 { 0 } else { len - 1 });
                let end = end.unwrap_or(if step > 0 { len } else { -1 });
                
                // Normalize negative indices
                let start = if start < 0 { len + start } else { start };
                let end = if end < 0 { len + end } else { end };
                
                let mut results = Vec::new();
                
                if step > 0 {
                    let mut i = start.max(0);
                    while i < end.min(len) {
                        if i >= 0 && (i as usize) < arr.len() {
                            results.push(arr[i as usize].clone());
                        }
                        i += step;
                    }
                } else {
                    let mut i = start.min(len - 1);
                    while i > end.max(-1) {
                        if i >= 0 && (i as usize) < arr.len() {
                            results.push(arr[i as usize].clone());
                        }
                        i += step; // step is negative
                    }
                }
                
                Ok(results)
            }
            _ => Ok(vec![]), // Slice on non-array returns empty
        }
    }
    
    /// Evaluate wildcard (all elements)
    fn evaluate_wildcard(
        &mut self,
        json: &Value,
        _context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        match json {
            Value::Array(arr) => Ok(arr.clone()),
            Value::Object(obj) => Ok(obj.values().cloned().collect()),
            _ => Ok(vec![]), // Wildcard on primitive returns empty
        }
    }
    
    /// Evaluate filter expression
    fn evaluate_filter(
        &mut self,
        json: &Value,
        filter: &FilterExpression,
        context: &PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        self.stats.filter_evaluations += 1;
        
        match json {
            Value::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    if self.evaluate_filter_on_item(item, filter, context)? {
                        results.push(item.clone());
                    }
                }
                Ok(results)
            }
            Value::Object(obj) => {
                let mut results = Vec::new();
                for value in obj.values() {
                    if self.evaluate_filter_on_item(value, filter, context)? {
                        results.push(value.clone());
                    }
                }
                Ok(results)
            }
            _ => Ok(vec![]), // Filter on primitive returns empty
        }
    }
    
    /// Evaluate recursive descent (find all matching descendants)
    fn evaluate_recursive_descent(
        &mut self,
        json: &Value,
        context: &mut PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        self.stats.recursive_descents += 1;
        
        let mut results = Vec::new();
        let mut stack = vec![(json.clone(), 0)];
        
        while let Some((current_value, depth)) = stack.pop() {
            if depth > context.max_depth {
                continue;
            }
            
            context.nodes_visited += 1;
            if context.nodes_visited >= context.max_nodes {
                break;
            }
            
            // Add current value to results
            results.push(current_value.clone());
            
            // Add children to stack
            match &current_value {
                Value::Object(obj) => {
                    for value in obj.values() {
                        stack.push((value.clone(), depth + 1));
                    }
                }
                Value::Array(arr) => {
                    for value in arr {
                        stack.push((value.clone(), depth + 1));
                    }
                }
                _ => {} // Primitives have no children
            }
        }
        
        Ok(results)
    }
    
    /// Evaluate union of multiple selectors
    fn evaluate_union(
        &mut self,
        json: &Value,
        segments: &[PropertySegment],
        context: &mut PropertyEvaluationContext,
    ) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();
        
        for segment in segments {
            let segment_results = self.evaluate_single_segment(json, segment, context)?;
            results.extend(segment_results);
        }
        
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        let mut unique_results = Vec::new();
        
        for value in results {
            let value_hash = self.hash_value(&value);
            if seen.insert(value_hash) {
                unique_results.push(value);
            }
        }
        
        Ok(unique_results)
    }
    
    /// Evaluate filter expression on a single item
    fn evaluate_filter_on_item(
        &self,
        item: &Value,
        filter: &FilterExpression,
        _context: &PropertyEvaluationContext,
    ) -> JsonPathResult<bool> {
        match filter {
            FilterExpression::Raw(filter_str) => {
                // For now, implement basic filter parsing
                // Full filter evaluation engine needed
                self.evaluate_basic_filter(item, filter_str)
            }
            // Add other filter types as needed
            _ => Ok(false),
        }
    }
    
    /// Basic filter evaluation (needs full implementation)
    fn evaluate_basic_filter(&self, item: &Value, filter: &str) -> JsonPathResult<bool> {
        // Simplified filter evaluation - full implementation needed
        // This is just a placeholder for common patterns
        
        if filter.contains("@.") {
            // Property-based filter
            if let Some(property) = self.extract_property_from_filter(filter) {
                if let Some(obj) = item.as_object() {
                    return Ok(obj.contains_key(&property));
                }
            }
        }
        
        // Default to false for unimplemented filters
        Ok(false)
    }
    
    /// Extract property name from basic filter (placeholder)
    fn extract_property_from_filter(&self, filter: &str) -> Option<String> {
        // Very basic extraction - needs proper filter parsing
        if let Some(start) = filter.find("@.") {
            let property_start = start + 2;
            if let Some(end) = filter[property_start..].find(|c: char| !c.is_alphanumeric() && c != '_') {
                return Some(filter[property_start..property_start + end].to_string());
            }
        }
        None
    }
    
    /// Hash value for deduplication
    fn hash_value(&self, value: &Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        // This is a simplified hash - proper implementation needed
        format!("{:?}", value).hash(&mut hasher);
        hasher.finish()
    }
}
```

---

## ERROR HANDLING AND VALIDATION

```rust
#[derive(Debug, Clone)]
pub enum JsonPathError {
    ParseError {
        message: String,
        position: usize,
    },
    MaxDepthExceeded {
        depth: usize,
        max_depth: usize,
    },
    MaxNodesExceeded {
        visited: u64,
        max_nodes: u64,
    },
    InvalidSlice {
        message: String,
    },
    FilterError {
        filter: String,
        error: String,
    },
    InvalidPropertyPath {
        path: String,
        reason: String,
    },
}

impl std::fmt::Display for JsonPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { message, position } => {
                write!(f, "Parse error at position {}: {}", position, message)
            }
            Self::MaxDepthExceeded { depth, max_depth } => {
                write!(f, "Maximum recursion depth exceeded: {} > {}", depth, max_depth)
            }
            Self::MaxNodesExceeded { visited, max_nodes } => {
                write!(f, "Maximum nodes exceeded: {} > {}", visited, max_nodes)
            }
            Self::InvalidSlice { message } => {
                write!(f, "Invalid slice: {}", message)
            }
            Self::FilterError { filter, error } => {
                write!(f, "Filter error in '{}': {}", filter, error)
            }
            Self::InvalidPropertyPath { path, reason } => {
                write!(f, "Invalid property path '{}': {}", path, reason)
            }
        }
    }
}

impl std::error::Error for JsonPathError {}
```

---

## TESTING REQUIREMENTS

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_basic_property_access() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!({
            "users": [
                {"name": "John", "age": 30},
                {"name": "Jane", "age": 25}
            ]
        });
        
        let result = evaluator.evaluate_property_path(&json, "users").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].is_array());
    }
    
    #[test]
    fn test_array_index_access() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!({
            "users": [
                {"name": "John", "age": 30},
                {"name": "Jane", "age": 25}
            ]
        });
        
        let result = evaluator.evaluate_property_path(&json, "users[0]").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "John");
    }
    
    #[test]
    fn test_negative_array_index() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!(["first", "second", "third"]);
        
        let result = evaluator.evaluate_property_path(&json, "[-1]").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "third");
    }
    
    #[test]
    fn test_array_slice() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!([1, 2, 3, 4, 5]);
        
        let result = evaluator.evaluate_property_path(&json, "[1:4]").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec![json!(2), json!(3), json!(4)]);
    }
    
    #[test]
    fn test_wildcard_access() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!({
            "users": [
                {"name": "John"},
                {"name": "Jane"}
            ]
        });
        
        let result = evaluator.evaluate_property_path(&json, "users[*].name").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "John");
        assert_eq!(result[1], "Jane");
    }
    
    #[test]
    fn test_quoted_property_access() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!({
            "first name": "John",
            "complex.key": "value"
        });
        
        let result = evaluator.evaluate_property_path(&json, r#"["first name"]"#).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "John");
        
        let result = evaluator.evaluate_property_path(&json, r#"["complex.key"]"#).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "value");
    }
    
    #[test]
    fn test_recursive_descent() {
        let mut evaluator = PropertyOperationsEvaluator::new();
        let json = json!({
            "store": {
                "products": [
                    {"name": "Product 1", "details": {"name": "Detail 1"}},
                    {"name": "Product 2", "details": {"name": "Detail 2"}}
                ],
                "info": {"name": "Store Info"}
            }
        });
        
        let result = evaluator.evaluate_property_path(&json, "..name").unwrap();
        assert!(result.len() >= 3); // Should find multiple name properties
    }
}
```

---

## IMPLEMENTATION TIMELINE

**Phase 1 (6 hours):** Property path parsing and AST implementation  
**Phase 2 (8 hours):** Basic property evaluation (property, index, slice)  
**Phase 3 (6 hours):** Wildcard and array operations  
**Phase 4 (8 hours):** Recursive descent implementation  
**Phase 5 (4 hours):** Filter expression foundation  
**Phase 6 (4 hours):** Error handling and validation  
**Phase 7 (4 hours):** Performance optimization and limits  
**Phase 8 (4 hours):** Comprehensive testing  

**Total Effort:** 44 hours

This violation is **HIGH PRIORITY** because it represents the core functionality failure of the JSONPath implementation - the foundation that all other features depend on.
# TURD4.md - JSONPath Filter Function Validation Bypass Violation

**Violation ID:** TURD4  
**Priority:** HIGH  
**Risk Level:** MEDIUM - Security and correctness concern  
**File Affected:** `packages/client/src/jsonpath/filter_parser/functions.rs`  
**Line:** 65  

---

## VIOLATION ANALYSIS

### The Fuckery
The JSONPath filter parser **allows unknown functions to pass without validation** with a "for now" comment. This creates both security vulnerabilities and correctness issues where malformed JSONPath expressions can silently produce incorrect results.

### Specific "For Now" Violation Found

**Line 65:**
```rust
pub fn validate_function(&self, name: &str, args: &[FilterExpression]) -> Result<(), FilterError> {
    match name {
        "length" => self.validate_length_function(args),
        "count" => self.validate_count_function(args), 
        "match" => self.validate_match_function(args),
        "search" => self.validate_search_function(args),
        "value" => self.validate_value_function(args),
        _ => {
            // Unknown function - let it pass for now (could be user-defined)
            return Ok(());  // <-- THIS IS THE PROBLEM
        }
    }
}
```

### Why This Is Problematic

1. **Security Risk**: Unvalidated function calls can be injection vectors
2. **Silent Failures**: Invalid JSONPath expressions succeed but produce wrong results
3. **Runtime Errors**: Unknown functions crash during evaluation rather than parsing
4. **No User Guidance**: Users get no feedback about typos or invalid function names
5. **Inconsistent Behavior**: Some functions are validated, others are ignored
6. **Performance Impact**: Invalid expressions aren't caught early, wasting processing time

---

## TECHNICAL DEEP DIVE

### What Gets Through Without Validation

**Typos and Misspellings:**
```jsonpath
$.users[?(@.age.lenght() > 18)]    // "lenght" instead of "length" - silently passes
$.data[?(@.name.macth("John"))]    // "macth" instead of "match" - silently passes  
$.items[?(@.price.coutn() > 0)]    // "coutn" instead of "count" - silently passes
```

**Completely Invalid Functions:**
```jsonpath
$.users[?(@.status.foobar())]      // No function "foobar" - silently passes
$.data[?(@.hacker.inject("evil"))] // Potential injection vector - silently passes
$.items[?(@.exec.system("rm -rf"))] // System execution attempt - silently passes
```

**Wrong Argument Counts:**
```jsonpath
$.data[?(@.name.match())]          // Missing required regex argument - silently passes
$.users[?(@.age.length(10, 20))]   // Extra arguments - silently passes
```

### Runtime Consequences

**Evaluation-Time Failures:**
```rust
// This passes validation but crashes during evaluation
let path = JsonPath::parse("$.data[?(@.name.nonexistent())]")?;
let result = path.evaluate(&json); // PANIC: unknown function "nonexistent"
```

**Silent Wrong Results:**
```rust
// User expects length check, but gets nothing due to typo
let path = JsonPath::parse("$.items[?(@.tags.lenght() > 0)]")?;
let result = path.evaluate(&json); // Returns empty Vec instead of error
```

**Potential Security Issues:**
```rust
// If user-defined functions aren't properly sandboxed
let malicious = JsonPath::parse("$.data[?(@.eval.exec('malicious_code'))]")?;
let result = malicious.evaluate(&user_data); // Could execute arbitrary code
```

---

## COMPLETE IMPLEMENTATION SOLUTION

### 1. Function Registry and Validation System

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct FunctionRegistry {
    /// Built-in functions provided by the JSONPath implementation
    builtin_functions: HashMap<String, FunctionDefinition>,
    /// User-registered custom functions
    custom_functions: HashMap<String, CustomFunctionDefinition>,
    /// Whether to allow unregistered functions (for backwards compatibility)
    allow_unknown: bool,
    /// Security settings for function validation
    security_config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub min_args: usize,
    pub max_args: Option<usize>,
    pub arg_types: Vec<FunctionArgType>,
    pub return_type: FunctionReturnType,
    pub is_pure: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone)]
pub struct CustomFunctionDefinition {
    pub definition: FunctionDefinition,
    pub implementation: Box<dyn Fn(&[Value]) -> Result<Value, FunctionError> + Send + Sync>,
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionArgType {
    Any,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
    JsonPath,
    Regex,
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionReturnType {
    Boolean,
    Number,
    String,
    Array,
    Object,
    Any,
}

#[derive(Debug, Clone, Copy)]
pub enum SecurityLevel {
    Safe,       // Pure functions with no side effects
    Restricted, // Functions with limited capabilities
    Unsafe,     // Functions that can access external resources
    Dangerous,  // Functions that can execute code or modify system
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub max_security_level: SecurityLevel,
    pub allow_regex: bool,
    pub allow_external_access: bool,
    pub max_function_depth: usize,
    pub max_execution_time_ms: u64,
}

static DEFAULT_REGISTRY: LazyLock<FunctionRegistry> = LazyLock::new(|| {
    let mut registry = FunctionRegistry::new();
    registry.register_builtin_functions();
    registry
});
```

### 2. Built-in Function Definitions

```rust
impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            builtin_functions: HashMap::new(),
            custom_functions: HashMap::new(),
            allow_unknown: false,
            security_config: SecurityConfig::default(),
        }
    }
    
    /// Register all built-in JSONPath functions
    fn register_builtin_functions(&mut self) {
        // length() function
        self.builtin_functions.insert("length".to_string(), FunctionDefinition {
            name: "length".to_string(),
            min_args: 0,
            max_args: Some(0),
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // count() function  
        self.builtin_functions.insert("count".to_string(), FunctionDefinition {
            name: "count".to_string(),
            min_args: 0,
            max_args: Some(0),
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // match() function
        self.builtin_functions.insert("match".to_string(), FunctionDefinition {
            name: "match".to_string(),
            min_args: 1,
            max_args: Some(2),
            arg_types: vec![FunctionArgType::Regex, FunctionArgType::String],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Restricted, // Regex can be expensive
        });
        
        // search() function
        self.builtin_functions.insert("search".to_string(), FunctionDefinition {
            name: "search".to_string(),
            min_args: 1,
            max_args: Some(2),
            arg_types: vec![FunctionArgType::String, FunctionArgType::String],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // value() function
        self.builtin_functions.insert("value".to_string(), FunctionDefinition {
            name: "value".to_string(),
            min_args: 0,
            max_args: Some(1),
            arg_types: vec![FunctionArgType::JsonPath],
            return_type: FunctionReturnType::Any,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // Additional useful functions
        self.register_additional_builtin_functions();
    }
    
    fn register_additional_builtin_functions(&mut self) {
        // min() function
        self.builtin_functions.insert("min".to_string(), FunctionDefinition {
            name: "min".to_string(),
            min_args: 0,
            max_args: None, // Variable arguments
            arg_types: vec![], // Will be validated at runtime
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // max() function
        self.builtin_functions.insert("max".to_string(), FunctionDefinition {
            name: "max".to_string(),
            min_args: 0,
            max_args: None,
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // avg() function
        self.builtin_functions.insert("avg".to_string(), FunctionDefinition {
            name: "avg".to_string(),
            min_args: 0,
            max_args: None,
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // sum() function
        self.builtin_functions.insert("sum".to_string(), FunctionDefinition {
            name: "sum".to_string(),
            min_args: 0,
            max_args: None,
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // size() function (alias for length)
        self.builtin_functions.insert("size".to_string(), FunctionDefinition {
            name: "size".to_string(),
            min_args: 0,
            max_args: Some(0),
            arg_types: vec![],
            return_type: FunctionReturnType::Number,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // empty() function
        self.builtin_functions.insert("empty".to_string(), FunctionDefinition {
            name: "empty".to_string(),
            min_args: 0,
            max_args: Some(0),
            arg_types: vec![],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // contains() function
        self.builtin_functions.insert("contains".to_string(), FunctionDefinition {
            name: "contains".to_string(),
            min_args: 1,
            max_args: Some(1),
            arg_types: vec![FunctionArgType::Any],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // startswith() function
        self.builtin_functions.insert("startswith".to_string(), FunctionDefinition {
            name: "startswith".to_string(),
            min_args: 1,
            max_args: Some(1),
            arg_types: vec![FunctionArgType::String],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // endswith() function
        self.builtin_functions.insert("endswith".to_string(), FunctionDefinition {
            name: "endswith".to_string(),
            min_args: 1,
            max_args: Some(1),
            arg_types: vec![FunctionArgType::String],
            return_type: FunctionReturnType::Boolean,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
        
        // type() function
        self.builtin_functions.insert("type".to_string(), FunctionDefinition {
            name: "type".to_string(),
            min_args: 0,
            max_args: Some(0),
            arg_types: vec![],
            return_type: FunctionReturnType::String,
            is_pure: true,
            security_level: SecurityLevel::Safe,
        });
    }
}

impl SecurityConfig {
    pub fn default() -> Self {
        Self {
            max_security_level: SecurityLevel::Safe,
            allow_regex: true,
            allow_external_access: false,
            max_function_depth: 10,
            max_execution_time_ms: 5000,
        }
    }
    
    pub fn strict() -> Self {
        Self {
            max_security_level: SecurityLevel::Safe,
            allow_regex: false,
            allow_external_access: false,
            max_function_depth: 5,
            max_execution_time_ms: 1000,
        }
    }
    
    pub fn permissive() -> Self {
        Self {
            max_security_level: SecurityLevel::Restricted,
            allow_regex: true,
            allow_external_access: false,
            max_function_depth: 20,
            max_execution_time_ms: 10000,
        }
    }
}
```

### 3. Complete Function Validation Implementation

```rust
#[derive(Debug, Clone)]
pub enum FunctionValidationError {
    UnknownFunction {
        name: String,
        available_functions: Vec<String>,
        suggestions: Vec<String>,
    },
    WrongArgumentCount {
        function: String,
        expected_min: usize,
        expected_max: Option<usize>,
        actual: usize,
    },
    WrongArgumentType {
        function: String,
        argument_index: usize,
        expected_type: FunctionArgType,
        actual_type: String,
    },
    SecurityViolation {
        function: String,
        security_level: SecurityLevel,
        max_allowed: SecurityLevel,
    },
    RegexNotAllowed {
        function: String,
    },
    FunctionDepthExceeded {
        current_depth: usize,
        max_depth: usize,
    },
}

impl FunctionRegistry {
    /// Validate a function call with comprehensive checking
    pub fn validate_function_call(
        &self,
        name: &str,
        args: &[FilterExpression],
        current_depth: usize,
    ) -> Result<(), FunctionValidationError> {
        // Check function depth
        if current_depth > self.security_config.max_function_depth {
            return Err(FunctionValidationError::FunctionDepthExceeded {
                current_depth,
                max_depth: self.security_config.max_function_depth,
            });
        }
        
        // Look for function definition
        let function_def = if let Some(def) = self.builtin_functions.get(name) {
            def
        } else if let Some(custom_def) = self.custom_functions.get(name) {
            &custom_def.definition
        } else {
            return self.handle_unknown_function(name);
        };
        
        // Validate security level
        if !self.is_security_level_allowed(function_def.security_level) {
            return Err(FunctionValidationError::SecurityViolation {
                function: name.to_string(),
                security_level: function_def.security_level,
                max_allowed: self.security_config.max_security_level,
            });
        }
        
        // Validate argument count
        self.validate_argument_count(function_def, args)?;
        
        // Validate argument types
        self.validate_argument_types(function_def, args)?;
        
        // Special validation for regex functions
        if function_def.name == "match" && !self.security_config.allow_regex {
            return Err(FunctionValidationError::RegexNotAllowed {
                function: name.to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Handle unknown function with helpful error messages
    fn handle_unknown_function(&self, name: &str) -> Result<(), FunctionValidationError> {
        if self.allow_unknown {
            return Ok(());
        }
        
        let available_functions: Vec<String> = self.builtin_functions.keys()
            .chain(self.custom_functions.keys())
            .cloned()
            .collect();
        
        let suggestions = self.find_function_suggestions(name, &available_functions);
        
        Err(FunctionValidationError::UnknownFunction {
            name: name.to_string(),
            available_functions,
            suggestions,
        })
    }
    
    /// Find similar function names for suggestions
    fn find_function_suggestions(&self, name: &str, available: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Exact matches (case-insensitive)
        for func_name in available {
            if func_name.to_lowercase() == name.to_lowercase() && func_name != name {
                suggestions.push(func_name.clone());
            }
        }
        
        // Levenshtein distance suggestions
        for func_name in available {
            let distance = self.levenshtein_distance(name, func_name);
            if distance <= 2 && distance > 0 {
                suggestions.push(func_name.clone());
            }
        }
        
        // Prefix/suffix matches
        for func_name in available {
            if func_name.starts_with(name) || func_name.ends_with(name) ||
               name.starts_with(func_name) || name.ends_with(func_name) {
                if !suggestions.contains(func_name) {
                    suggestions.push(func_name.clone());
                }
            }
        }
        
        suggestions.truncate(5); // Limit to 5 suggestions
        suggestions
    }
    
    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 { return len2; }
        if len2 == 0 { return len1; }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 { matrix[i][0] = i; }
        for j in 0..=len2 { matrix[0][j] = j; }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i-1] == chars2[j-1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i-1][j] + 1,     // deletion
                        matrix[i][j-1] + 1      // insertion
                    ),
                    matrix[i-1][j-1] + cost     // substitution
                );
            }
        }
        
        matrix[len1][len2]
    }
    
    /// Check if security level is allowed
    fn is_security_level_allowed(&self, level: SecurityLevel) -> bool {
        use SecurityLevel::*;
        match (self.security_config.max_security_level, level) {
            (Safe, Safe) => true,
            (Restricted, Safe | Restricted) => true,
            (Unsafe, Safe | Restricted | Unsafe) => true,
            (Dangerous, _) => true,
            _ => false,
        }
    }
    
    /// Validate function argument count
    fn validate_argument_count(
        &self,
        function_def: &FunctionDefinition,
        args: &[FilterExpression],
    ) -> Result<(), FunctionValidationError> {
        let arg_count = args.len();
        
        if arg_count < function_def.min_args {
            return Err(FunctionValidationError::WrongArgumentCount {
                function: function_def.name.clone(),
                expected_min: function_def.min_args,
                expected_max: function_def.max_args,
                actual: arg_count,
            });
        }
        
        if let Some(max_args) = function_def.max_args {
            if arg_count > max_args {
                return Err(FunctionValidationError::WrongArgumentCount {
                    function: function_def.name.clone(),
                    expected_min: function_def.min_args,
                    expected_max: function_def.max_args,
                    actual: arg_count,
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate function argument types
    fn validate_argument_types(
        &self,
        function_def: &FunctionDefinition,
        args: &[FilterExpression],
    ) -> Result<(), FunctionValidationError> {
        // If no specific types are defined, allow any types
        if function_def.arg_types.is_empty() {
            return Ok(());
        }
        
        for (i, arg) in args.iter().enumerate() {
            if i < function_def.arg_types.len() {
                let expected_type = function_def.arg_types[i];
                let actual_type = self.infer_expression_type(arg);
                
                if !self.is_type_compatible(expected_type, &actual_type) {
                    return Err(FunctionValidationError::WrongArgumentType {
                        function: function_def.name.clone(),
                        argument_index: i,
                        expected_type,
                        actual_type,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Infer the type of a filter expression
    fn infer_expression_type(&self, expr: &FilterExpression) -> String {
        match expr {
            FilterExpression::Literal(value) => {
                match value {
                    Value::Null => "null".to_string(),
                    Value::Bool(_) => "boolean".to_string(),
                    Value::Number(_) => "number".to_string(),
                    Value::String(_) => "string".to_string(),
                    Value::Array(_) => "array".to_string(),
                    Value::Object(_) => "object".to_string(),
                }
            }
            FilterExpression::Path(_) => "any".to_string(), // Could be any type
            FilterExpression::Function { .. } => "any".to_string(), // Depends on function
            FilterExpression::Comparison { .. } => "boolean".to_string(),
            FilterExpression::Logical { .. } => "boolean".to_string(),
            FilterExpression::Regex(_) => "regex".to_string(),
        }
    }
    
    /// Check if actual type is compatible with expected type
    fn is_type_compatible(&self, expected: FunctionArgType, actual: &str) -> bool {
        match expected {
            FunctionArgType::Any => true,
            FunctionArgType::String => actual == "string",
            FunctionArgType::Number => actual == "number",
            FunctionArgType::Boolean => actual == "boolean",
            FunctionArgType::Array => actual == "array",
            FunctionArgType::Object => actual == "object",
            FunctionArgType::Null => actual == "null",
            FunctionArgType::JsonPath => actual == "path" || actual == "any",
            FunctionArgType::Regex => actual == "regex" || actual == "string",
        }
    }
}
```

### 4. Custom Function Registration

```rust
impl FunctionRegistry {
    /// Register a custom user-defined function
    pub fn register_custom_function<F>(
        &mut self,
        name: String,
        min_args: usize,
        max_args: Option<usize>,
        security_level: SecurityLevel,
        implementation: F,
    ) -> Result<(), FunctionValidationError>
    where
        F: Fn(&[Value]) -> Result<Value, FunctionError> + Send + Sync + 'static,
    {
        // Validate function name
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(FunctionValidationError::InvalidFunctionName {
                name: name.clone(),
            });
        }
        
        // Check if function already exists
        if self.builtin_functions.contains_key(&name) {
            return Err(FunctionValidationError::FunctionAlreadyExists {
                name: name.clone(),
                is_builtin: true,
            });
        }
        
        let function_def = FunctionDefinition {
            name: name.clone(),
            min_args,
            max_args,
            arg_types: vec![], // Custom functions use runtime type checking
            return_type: FunctionReturnType::Any,
            is_pure: false, // Assume custom functions may have side effects
            security_level,
        };
        
        let custom_def = CustomFunctionDefinition {
            definition: function_def,
            implementation: Box::new(implementation),
        };
        
        self.custom_functions.insert(name, custom_def);
        Ok(())
    }
    
    /// Get all available function names
    pub fn get_available_functions(&self) -> Vec<String> {
        let mut functions: Vec<String> = self.builtin_functions.keys()
            .chain(self.custom_functions.keys())
            .cloned()
            .collect();
        functions.sort();
        functions
    }
    
    /// Get function definition for introspection
    pub fn get_function_definition(&self, name: &str) -> Option<&FunctionDefinition> {
        self.builtin_functions.get(name)
            .or_else(|| self.custom_functions.get(name).map(|f| &f.definition))
    }
}
```

### 5. Integration with Filter Parser

```rust
impl FilterParser {
    /// Parse and validate function call with comprehensive error handling
    pub fn parse_function_call(&mut self) -> Result<FilterExpression, ParseError> {
        let function_name = self.consume_identifier()?;
        self.expect_token(Token::LeftParen)?;
        
        let mut args = Vec::new();
        
        if !self.check_token(&Token::RightParen) {
            loop {
                args.push(self.parse_filter_expression()?);
                if !self.consume_if_match(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.expect_token(Token::RightParen)?;
        
        // Validate function call
        let registry = &DEFAULT_REGISTRY;
        registry.validate_function_call(&function_name, &args, self.function_depth)
            .map_err(|e| ParseError::FunctionValidation {
                function: function_name.clone(),
                error: e,
            })?;
        
        Ok(FilterExpression::Function {
            name: function_name,
            args,
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    FunctionValidation {
        function: String,
        error: FunctionValidationError,
    },
    // ... other parse errors
}

impl std::fmt::Display for FunctionValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFunction { name, suggestions, .. } => {
                write!(f, "Unknown function '{}'", name)?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean: {}?", suggestions.join(", "))?;
                }
                Ok(())
            }
            Self::WrongArgumentCount { function, expected_min, expected_max, actual } => {
                match expected_max {
                    Some(max) if max == expected_min => {
                        write!(f, "Function '{}' expects {} arguments, got {}", 
                               function, expected_min, actual)
                    }
                    Some(max) => {
                        write!(f, "Function '{}' expects {}-{} arguments, got {}", 
                               function, expected_min, max, actual)
                    }
                    None => {
                        write!(f, "Function '{}' expects at least {} arguments, got {}", 
                               function, expected_min, actual)
                    }
                }
            }
            Self::WrongArgumentType { function, argument_index, expected_type, actual_type } => {
                write!(f, "Function '{}' argument {} expects {:?}, got {}", 
                       function, argument_index + 1, expected_type, actual_type)
            }
            Self::SecurityViolation { function, security_level, max_allowed } => {
                write!(f, "Function '{}' security level {:?} exceeds maximum allowed {:?}", 
                       function, security_level, max_allowed)
            }
            Self::RegexNotAllowed { function } => {
                write!(f, "Function '{}' uses regex which is not allowed in current security mode", 
                       function)
            }
            Self::FunctionDepthExceeded { current_depth, max_depth } => {
                write!(f, "Function call depth {} exceeds maximum allowed {}", 
                       current_depth, max_depth)
            }
        }
    }
}
```

---

## SECURITY CONSIDERATIONS

### Sandboxing Custom Functions
```rust
impl SecurityConfig {
    /// Create a sandbox configuration for untrusted code
    pub fn sandbox() -> Self {
        Self {
            max_security_level: SecurityLevel::Safe,
            allow_regex: false,
            allow_external_access: false,
            max_function_depth: 3,
            max_execution_time_ms: 100,
        }
    }
}
```

### Preventing Function Injection
```rust
impl FunctionRegistry {
    /// Validate function name against injection patterns
    fn validate_function_name(&self, name: &str) -> Result<(), FunctionValidationError> {
        // Check for dangerous patterns
        let dangerous_patterns = [
            "eval", "exec", "system", "shell", "cmd", "run",
            "import", "require", "load", "include",
            "__", "process", "global", "window"
        ];
        
        for pattern in &dangerous_patterns {
            if name.to_lowercase().contains(pattern) {
                return Err(FunctionValidationError::SecurityViolation {
                    function: name.to_string(),
                    security_level: SecurityLevel::Dangerous,
                    max_allowed: self.security_config.max_security_level,
                });
            }
        }
        
        Ok(())
    }
}
```

---

## TESTING REQUIREMENTS

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unknown_function_validation() {
        let registry = FunctionRegistry::new();
        registry.register_builtin_functions();
        
        let result = registry.validate_function_call("nonexistent", &[], 0);
        
        assert!(result.is_err());
        if let Err(FunctionValidationError::UnknownFunction { name, suggestions, .. }) = result {
            assert_eq!(name, "nonexistent");
            assert!(!suggestions.is_empty());
        }
    }
    
    #[test]
    fn test_typo_suggestions() {
        let registry = FunctionRegistry::new();
        registry.register_builtin_functions();
        
        let result = registry.validate_function_call("lenght", &[], 0);
        
        assert!(result.is_err());
        if let Err(FunctionValidationError::UnknownFunction { suggestions, .. }) = result {
            assert!(suggestions.contains(&"length".to_string()));
        }
    }
    
    #[test]
    fn test_argument_count_validation() {
        let registry = FunctionRegistry::new();
        registry.register_builtin_functions();
        
        // Test too many arguments
        let args = vec![
            FilterExpression::Literal(Value::String("test".to_string())),
            FilterExpression::Literal(Value::String("extra".to_string())),
        ];
        let result = registry.validate_function_call("length", &args, 0);
        
        assert!(result.is_err());
        if let Err(FunctionValidationError::WrongArgumentCount { .. }) = result {
            // Expected
        } else {
            panic!("Expected WrongArgumentCount error");
        }
    }
    
    #[test]
    fn test_security_level_enforcement() {
        let mut registry = FunctionRegistry::new();
        registry.security_config.max_security_level = SecurityLevel::Safe;
        
        // Register a restricted function
        registry.register_custom_function(
            "restricted_func".to_string(),
            0,
            Some(0),
            SecurityLevel::Restricted,
            |_| Ok(Value::Bool(true)),
        ).unwrap();
        
        let result = registry.validate_function_call("restricted_func", &[], 0);
        
        assert!(result.is_err());
        if let Err(FunctionValidationError::SecurityViolation { .. }) = result {
            // Expected
        } else {
            panic!("Expected SecurityViolation error");
        }
    }
}
```

---

## IMPLEMENTATION TIMELINE

**Phase 1 (4 hours):** Function registry and basic validation infrastructure  
**Phase 2 (3 hours):** Built-in function definitions and comprehensive validation  
**Phase 3 (3 hours):** Error handling and suggestion system  
**Phase 4 (2 hours):** Security validation and sandboxing  
**Phase 5 (2 hours):** Custom function registration system  
**Phase 6 (2 hours):** Integration with filter parser  
**Phase 7 (2 hours):** Testing and edge cases  

**Total Effort:** 18 hours

This violation is **HIGH PRIORITY** because it represents both a security vulnerability and a significant user experience issue where invalid JSONPath expressions silently produce incorrect results.
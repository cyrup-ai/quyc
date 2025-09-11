//! `JSONPath` Abstract Syntax Tree (AST) definitions
//!
//! Core type definitions for representing `JSONPath` expressions as structured data.
//! Provides zero-allocation AST nodes optimized for streaming evaluation.

/// Individual `JSONPath` selector component
#[derive(Debug, Clone)]
pub enum JsonSelector {
    /// Root selector ($)
    Root,

    /// Child property access (.property or ['property'])
    Child {
        /// Name of the child property to access
        name: String,
        /// Whether to use exact string matching (true) or case-insensitive (false)
        exact_match: bool,
    },

    /// Recursive descent (..)
    RecursiveDescent,

    /// Array index access ([0], [-1], etc.)
    Index {
        /// Array index value (negative indices count from end)
        index: i64,
        /// For negative indices, whether to count from end
        from_end: bool,
    },

    /// Array slice ([start:end], [start:], [:end])
    Slice {
        /// Start index for slice (None means from beginning)
        start: Option<i64>,
        /// End index for slice (None means to end)
        end: Option<i64>,
        /// Step size for slice (None means step of 1)
        step: Option<i64>,
    },

    /// Wildcard selector ([*] or .*)
    Wildcard,

    /// Filter expression ([?(@.property > value)])
    Filter {
        /// Filter expression AST
        expression: FilterExpression,
    },

    /// Multiple selectors (union operator)
    Union {
        /// List of selectors in the union
        selectors: Vec<JsonSelector>,
    },
}

/// Filter expression AST for `JSONPath` predicates
#[derive(Debug, Clone)]
pub enum FilterExpression {
    /// Current node reference (@)
    Current,

    /// Property access (@.property)
    Property {
        /// Property path components
        path: Vec<String>,
    },

    /// Complex `JSONPath` expressions (@.items[*], @.data[0:5], etc.)
    JsonPath {
        /// Selectors in the `JSONPath` expression
        selectors: Vec<JsonSelector>,
    },

    /// Literal values (strings, numbers, booleans)
    Literal {
        /// The literal value
        value: FilterValue,
    },

    /// Comparison operations
    Comparison {
        /// Left operand of comparison
        left: Box<FilterExpression>,
        /// Comparison operator
        operator: ComparisonOp,
        /// Right operand of comparison
        right: Box<FilterExpression>,
    },

    /// Logical operations (&&, ||)
    Logical {
        /// Left operand of logical operation
        left: Box<FilterExpression>,
        /// Logical operator
        operator: LogicalOp,
        /// Right operand of logical operation
        right: Box<FilterExpression>,
    },

    /// Regular expression matching
    Regex {
        /// Target expression to match against
        target: Box<FilterExpression>,
        /// Regular expression pattern
        pattern: String,
    },

    /// Function calls (length, type, etc.)
    Function {
        /// Function name
        name: String,
        /// Function arguments
        args: Vec<FilterExpression>,
    },
}

/// Filter expression literal values
#[derive(Debug, Clone)]
pub enum FilterValue {
    /// String literal value
    String(String),
    /// Floating-point number literal value
    Number(f64),
    /// Integer literal value
    Integer(i64),
    /// Boolean literal value
    Boolean(bool),
    /// Null literal value
    Null,
    /// Missing property (different from null per RFC 9535)
    Missing,
}

/// Comparison operators for filter expressions
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    /// Equality comparison (==)
    Equal,
    /// Inequality comparison (!=)
    NotEqual,
    /// Less than comparison (<)
    Less,
    /// Less than or equal comparison (<=)
    LessEq,
    /// Greater than comparison (>)
    Greater,
    /// Greater than or equal comparison (>=)
    GreaterEq,
    /// Membership test (in)
    In,
    /// Non-membership test (not in)
    NotIn,
    /// Contains substring test (contains)
    Contains,
    /// Starts with prefix test (starts with)
    StartsWith,
    /// Ends with suffix test (ends with)
    EndsWith,
    /// Regular expression match (=~)
    Match,
    /// Regular expression non-match (!~)
    NotMatch,
}

/// Logical operators for filter expressions
#[derive(Debug, Clone, Copy)]
pub enum LogicalOp {
    /// Logical AND operator (&&)
    And,
    /// Logical OR operator (||)
    Or,
}

/// Comprehensive complexity metrics for `JSONPath` expression analysis
///
/// Provides detailed breakdown of complexity factors for performance optimization guidance.
/// All metrics are computed at compile time for zero runtime overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplexityMetrics {
    /// Count of recursive descent (..) selectors and their effective nesting depth
    pub recursive_descent_depth: u32,
    /// Total number of selectors in the expression chain
    pub total_selector_count: u32,
    /// Sum of all filter expression complexity scores
    pub filter_complexity_sum: u32,
    /// Largest slice range for slice operations (end - start)
    pub max_slice_range: u32,
    /// Number of selectors within union operations
    pub union_selector_count: u32,
}

impl Default for ComplexityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityMetrics {
    /// Create new complexity metrics with zero values
    #[inline]
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            recursive_descent_depth: 0,
            total_selector_count: 0,
            filter_complexity_sum: 0,
            max_slice_range: 0,
            union_selector_count: 0,
        }
    }

    /// Add metrics from another `ComplexityMetrics` instance
    #[inline]
    pub fn add(&mut self, other: &ComplexityMetrics) {
        self.recursive_descent_depth = self
            .recursive_descent_depth
            .saturating_add(other.recursive_descent_depth);
        self.total_selector_count = self
            .total_selector_count
            .saturating_add(other.total_selector_count);
        self.filter_complexity_sum = self
            .filter_complexity_sum
            .saturating_add(other.filter_complexity_sum);
        self.max_slice_range = self.max_slice_range.max(other.max_slice_range);
        self.union_selector_count = self
            .union_selector_count
            .saturating_add(other.union_selector_count);
    }
}

impl FilterExpression {
    /// Calculate complexity score for filter expressions
    #[inline]
    #[must_use] 
    pub fn complexity_score(&self) -> u32 {
        match self {
            FilterExpression::Current => 1,
            FilterExpression::Property { path } => path.len() as u32,
            FilterExpression::Literal { .. } => 1,
            FilterExpression::Comparison { left, right, .. } => {
                2 + left.complexity_score() + right.complexity_score()
            }
            FilterExpression::Logical { left, right, .. } => {
                3 + left.complexity_score() + right.complexity_score()
            }
            FilterExpression::Regex { target, .. } => {
                5 + target.complexity_score() // Regex operations are more expensive
            }
            FilterExpression::Function { args, .. } => {
                5 + args.iter().map(FilterExpression::complexity_score).sum::<u32>()
            }
            FilterExpression::JsonPath { selectors } => {
                selectors.len() as u32 * 2 // Complex JSONPath expressions are more expensive
            }
        }
    }
}

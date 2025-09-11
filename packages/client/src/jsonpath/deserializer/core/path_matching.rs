use serde::de::DeserializeOwned;

use super::types::JsonPathDeserializer;

impl<T> JsonPathDeserializer<'_, T>
where
    T: DeserializeOwned,
{
    /// Check if current position matches `JSONPath` expression
    #[inline]
    pub(super) fn matches_current_path(&self) -> bool {
        // Simplified JSONPath matching for basic patterns
        // This handles common cases like $.data[*], $.items[*]

        let expression = self.path_expression.as_string();

        // For array wildcard patterns like $.data[*], $.items[*]
        if expression.starts_with("$.") && expression.ends_with("[*]") {
            // Match when we're inside the target array to capture array elements
            return self.in_target_array;
        }

        // For root array patterns like $[*]
        if expression == "$[*]" {
            // Match when we're inside the root array
            return self.in_target_array;
        }

        // Default fallback - match if we're in target array
        self.in_target_array
    }

    /// Read a JSON property name from the current position
    #[inline]
    pub(super) fn read_property_name(&mut self) -> crate::jsonpath::error::JsonPathResult<String> {
        let mut property_name = String::new();
        let mut escaped = false;

        while let Some(byte) = self.read_next_byte()? {
            if escaped {
                escaped = false;
                match byte {
                    b'"' => property_name.push('"'),
                    b'\\' => property_name.push('\\'),
                    b'/' => property_name.push('/'),
                    b'b' => property_name.push('\u{0008}'),
                    b'f' => property_name.push('\u{000C}'),
                    b'n' => property_name.push('\n'),
                    b'r' => property_name.push('\r'),
                    b't' => property_name.push('\t'),
                    _ => {
                        property_name.push('\\');
                        property_name.push(byte as char);
                    }
                }
            } else {
                match byte {
                    b'"' => break, // End of property name
                    b'\\' => escaped = true,
                    _ => property_name.push(byte as char),
                }
            }
        }

        Ok(property_name)
    }
}

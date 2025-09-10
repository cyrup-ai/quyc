#[cfg(test)]
mod tests {
    use quyc_client::jsonpath::{
        deserializer::core::types::{StreamingJsonPathState, SelectorAdvanceResult, PathSegment, RecursiveDescentFrame},
        parser::JsonPathExpression,
        ast::JsonSelector,
        error::JsonPathError,
    };
    
    #[test]
    fn test_streaming_state_selector_advancement() {
        let expression = JsonPathExpression::new(
            vec![JsonSelector::Root, JsonSelector::Property("data".to_string()), JsonSelector::Wildcard],
            "$.data[*]".to_string(),
            false,
        );
        let mut state = StreamingJsonPathState::new(&expression);
        
        assert_eq!(state.current_selector_index, 0);
        
        match state.advance_selector() {
            SelectorAdvanceResult::Advanced(1) => {
                assert_eq!(state.current_selector_index, 1);
            }
            _ => panic!("Expected selector advancement"),
        }
    }
    
    #[test]
    fn test_recursive_descent_stack_management() {
        let expression = JsonPathExpression::new(
            vec![JsonSelector::Root, JsonSelector::RecursiveDescent, JsonSelector::Wildcard],
            "$..[*]".to_string(),
            false,
        );
        let mut state = StreamingJsonPathState::new(&expression);
        
        // Enter recursive descent
        state.enter_recursive_descent("$.test".to_string(), 1).unwrap();
        
        assert!(state.in_recursive_descent);
        assert_eq!(state.recursive_descent_stack.len(), 1);
        assert_eq!(state.evaluation_stats.recursive_descents_performed, 1);
        
        // Exit recursive descent
        let frame = state.exit_recursive_descent();
        assert!(frame.is_some());
        assert!(!state.in_recursive_descent);
        assert_eq!(state.recursive_descent_stack.len(), 0);
    }
    
    #[test]  
    fn test_path_breadcrumb_reconstruction() {
        let expression = JsonPathExpression::new(
            vec![JsonSelector::Root, JsonSelector::Property("users".to_string()), JsonSelector::Index(0)],
            "$.users[0]".to_string(),
            false,
        );
        let mut state = StreamingJsonPathState::new(&expression);
        
        state.push_navigation_frame(PathSegment::Property("users".to_string()), true);
        state.push_navigation_frame(PathSegment::ArrayIndex(0), true);
        
        let current_path = state.current_json_path();
        assert_eq!(current_path, "$.users[0]");
        
        assert_eq!(state.evaluation_stats.matches_found, 2);
    }
    
    #[test]
    fn test_depth_tracking_and_limits() {
        let expression = JsonPathExpression::new(
            vec![JsonSelector::Root, JsonSelector::RecursiveDescent],
            "$..".to_string(),
            false,
        );
        let mut state = StreamingJsonPathState::new(&expression);
        state.max_depth = 5; // Lower limit for testing
        
        // Test normal depth tracking
        for _ in 0..4 {
            state.enter_depth();
        }
        assert_eq!(state.current_depth, 4);
        assert_eq!(state.evaluation_stats.max_depth_reached, 4);
        
        // Test depth limit enforcement
        state.enter_depth(); // depth = 5
        let result = state.enter_recursive_descent("$.deep".to_string(), 0);
        
        // Should fail because current_depth (5) > max_depth (5) 
        assert!(result.is_err());
    }
    
    #[test]
    fn test_performance_under_load() {
        let expression = JsonPathExpression::new(
            vec![JsonSelector::Root, JsonSelector::Property("data".to_string()), JsonSelector::Wildcard],
            "$.data[*]".to_string(),
            false,
        );
        let mut state = StreamingJsonPathState::new(&expression);
        
        let start_time = std::time::Instant::now();
        
        // Simulate processing 100K objects
        for i in 0..100_000 {
            state.evaluation_stats.nodes_processed += 1;
            
            if i % 100 == 0 {
                state.push_navigation_frame(
                    PathSegment::ArrayIndex(i / 100), 
                    true
                );
                state.pop_navigation_frame();
            }
        }
        
        let duration = start_time.elapsed();
        let objects_per_second = 100_000.0 / duration.as_secs_f64();
        
        // Verify performance target
        assert!(objects_per_second > 100_000.0, 
            "Performance target not met: {:.0} objects/second", objects_per_second);
        
        // Verify memory usage is reasonable (< 8KB total state size)
        let state_size = std::mem::size_of_val(&state) + 
                        state.recursive_descent_stack.capacity() * std::mem::size_of::<RecursiveDescentFrame>() +
                        state.path_breadcrumbs.capacity() * std::mem::size_of::<quyc_client::jsonpath::deserializer::core::types::PathNavigationFrame>();
        assert!(state_size < 8192, "Memory usage too high: {} bytes", state_size);
    }
}
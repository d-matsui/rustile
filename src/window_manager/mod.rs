//! Window manager module with focused sub-modules
//!
//! This module organizes window management functionality into focused areas:
//! - Core: Initialization, main loop, and configuration
//! - Events: X11 event handling and dispatching
//! - Focus: Window focus management and visual indicators
//! - Window Operations: Window lifecycle and manipulation

// Re-export the main public interface
pub use core::WindowManager;

// Internal modules
mod core;
mod events;
mod focus;
mod window_ops;

#[cfg(test)]
mod tests {
    use x11rb::protocol::xproto::Window;
    
    // Test helper functions for window management logic
    // These test the core algorithms without requiring X11 connections
    
    /// Helper to test focus_next logic
    fn test_focus_next_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }
        
        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let next_idx = (current_idx + 1) % windows.len();
                Some(windows[next_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }
    
    /// Helper to test focus_prev logic
    fn test_focus_prev_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }
        
        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let prev_idx = if current_idx == 0 {
                    windows.len() - 1
                } else {
                    current_idx - 1
                };
                Some(windows[prev_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }
    
    /// Helper to test swap_with_master logic
    fn test_swap_with_master_logic(windows: &mut Vec<Window>, focused: Option<Window>) -> bool {
        if windows.len() < 2 {
            return false;
        }
        
        if let Some(focused) = focused {
            if let Some(focused_idx) = windows.iter().position(|&w| w == focused) {
                if focused_idx != 0 {
                    windows.swap(0, focused_idx);
                    return true;
                }
            }
        }
        false
    }
    
    #[test]
    fn test_focus_next_empty_windows() {
        let windows = vec![];
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_focus_next_single_window() {
        let windows = vec![10];
        
        // Should focus the only window when none focused
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // Should stay on same window when already focused
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_next_multiple_windows() {
        let windows = vec![10, 20, 30];
        
        // From no focus, should focus first window
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // From first window, should focus second
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(20));
        
        // From second window, should focus third
        let result = test_focus_next_logic(&windows, Some(20));
        assert_eq!(result, Some(30));
        
        // From last window, should wrap to first
        let result = test_focus_next_logic(&windows, Some(30));
        assert_eq!(result, Some(10));
        
        // From non-existent window, should focus first
        let result = test_focus_next_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_prev_empty_windows() {
        let windows = vec![];
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_focus_prev_single_window() {
        let windows = vec![10];
        
        // Should focus the only window when none focused
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // Should stay on same window when already focused
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_focus_prev_multiple_windows() {
        let windows = vec![10, 20, 30];
        
        // From no focus, should focus first window
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));
        
        // From first window, should wrap to last
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(30));
        
        // From second window, should focus first
        let result = test_focus_prev_logic(&windows, Some(20));
        assert_eq!(result, Some(10));
        
        // From last window, should focus second
        let result = test_focus_prev_logic(&windows, Some(30));
        assert_eq!(result, Some(20));
        
        // From non-existent window, should focus first
        let result = test_focus_prev_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }
    
    #[test]
    fn test_swap_with_master_empty_windows() {
        let mut windows = vec![];
        let result = test_swap_with_master_logic(&mut windows, None);
        assert!(!result); // No swap should occur
        assert!(windows.is_empty());
    }
    
    #[test]
    fn test_swap_with_master_single_window() {
        let mut windows = vec![10];
        let result = test_swap_with_master_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10]);
    }
    
    #[test]
    fn test_swap_with_master_multiple_windows() {
        // Test swapping non-master with master
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(30));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![30, 20, 10]);
        
        // Test swapping master with master (no-op)
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
        
        // Test swapping middle window with master
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(20));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![20, 10, 30]);
        
        // Test non-existent focused window
        let mut windows = vec![10, 20, 30];
        let result = test_swap_with_master_logic(&mut windows, Some(999));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }
    
    #[test]
    fn test_focus_cycling_edge_cases() {
        // Test with duplicate windows (should still work)
        let windows = vec![10, 10, 20];
        
        // Should find first occurrence and move correctly
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10)); // Next occurrence of 10
        
        // Test wrapping behavior
        let result = test_focus_next_logic(&windows, Some(20));  
        assert_eq!(result, Some(10)); // Wrap to first
    }
    
    #[test] 
    fn test_window_order_preservation() {
        // Test that window order is preserved correctly during swaps
        let mut windows = vec![1, 2, 3, 4, 5];
        
        // Swap last with master
        test_swap_with_master_logic(&mut windows, Some(5));
        assert_eq!(windows, vec![5, 2, 3, 4, 1]);
        
        // Swap back  
        test_swap_with_master_logic(&mut windows, Some(1));
        assert_eq!(windows, vec![1, 2, 3, 4, 5]);
    }
}
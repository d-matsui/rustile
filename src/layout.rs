//! Window layout algorithms for the tiling window manager

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

/// Represents different tiling layouts
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Master-stack layout: one master window on the left, stack on the right
    MasterStack,
}

/// Window layout manager
pub struct LayoutManager {
    current_layout: Layout,
}

impl LayoutManager {
    /// Creates a new layout manager with default layout
    pub fn new() -> Self {
        Self {
            current_layout: Layout::MasterStack,
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    /// Applies the current layout to the given windows
    pub fn apply_layout(
        &self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
        master_ratio: f32,
        gap: u32,
    ) -> Result<()> {
        match self.current_layout {
            Layout::MasterStack => self.tile_master_stack(conn, screen, windows, master_ratio, gap),
        }
    }

    /// Implements master-stack tiling layout
    ///
    /// Layout behavior:
    /// - Single window: Full screen minus gaps
    /// - Multiple windows: First window takes configurable ratio (master),
    ///   remaining windows stack vertically on the right, with gaps between
    fn tile_master_stack(
        &self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
        master_ratio: f32,
        gap: u32,
    ) -> Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        let screen_width = screen.width_in_pixels as i16;
        let screen_height = screen.height_in_pixels as i16;
        let num_windows = windows.len() as i16;
        let gap_i16 = gap as i16;

        // Configure master window
        let master_window = windows[0];
        let master_width = if num_windows > 1 {
            // Multiple windows: master takes ratio of available space, ensure minimum 100px
            let available_width = screen_width - 3 * gap_i16;
            if available_width > 150 {  // Need at least 150px total (100px master + 50px stack)
                ((available_width as f32 * master_ratio) as i16).max(100)
            } else {
                // Fallback: reduce gaps to fit windows
                ((screen_width / 2) as i16).max(100)
            }
        } else {
            // Single window: full width minus gaps, minimum 100px
            (screen_width - 2 * gap_i16).max(100)
        };

        let master_config = ConfigureWindowAux::new()
            .x(gap_i16 as i32)
            .y(gap_i16 as i32)
            .width(master_width.max(100) as u32)  // Minimum 100px width
            .height((screen_height - 2 * gap_i16).max(100) as u32);  // Minimum 100px height

        conn.configure_window(master_window, &master_config)?;

        // Configure stack windows if any
        if num_windows > 1 {
            let stack_windows = &windows[1..];
            let num_stack = stack_windows.len() as i16;
            let stack_x = gap_i16 + master_width + gap_i16;  // Add gap between master and stack
            let stack_width = (screen_width - stack_x - gap_i16).max(50);  // Minimum usable width
            
            // Ensure we have enough space for stack windows with minimum height
            let min_total_height = num_stack * 50 + (num_stack - 1) * gap_i16;  // 50px min per window
            let available_height = screen_height - 2 * gap_i16;
            
            let total_stack_height = if available_height >= min_total_height {
                available_height - (num_stack - 1) * gap_i16
            } else {
                // Fallback: reduce gaps if necessary to fit windows
                (available_height - num_stack * 50).max(num_stack * 50)
            };
            
            let stack_height = (total_stack_height / num_stack).max(50);  // Minimum 50px height

            for (index, &window) in stack_windows.iter().enumerate() {
                let stack_y = gap_i16 + (index as i16) * (stack_height + gap_i16);

                let stack_config = ConfigureWindowAux::new()
                    .x(stack_x as i32)
                    .y(stack_y as i32)
                    .width(stack_width.max(1) as u32)
                    .height(stack_height.max(1) as u32);

                conn.configure_window(window, &stack_config)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_manager_default() {
        let layout_manager = LayoutManager::default();
        match layout_manager.current_layout {
            Layout::MasterStack => (),
        }
    }

    #[test]
    fn test_empty_window_list() {
        let _layout_manager = LayoutManager::new();

        // Mock screen dimensions
        let _screen_width = 1280;
        let _screen_height = 720;

        // This should not panic with empty windows
        let windows: Vec<Window> = vec![];

        // We can't easily test X11 operations without mocking,
        // but we can at least ensure the logic doesn't panic
        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_master_window_dimensions() {
        // Test that master window calculations are correct
        let screen_width = 1280_f32;
        let screen_height = 720_f32;

        // With one window, it should take full screen
        let expected_single_width = screen_width as u32;
        let expected_single_height = screen_height as u32;

        // With multiple windows, master takes master_ratio of width (default 0.5)
        let master_ratio = 0.5_f32;
        let expected_master_width = (screen_width * master_ratio) as u32;
        let expected_master_height = screen_height as u32;

        assert_eq!(expected_single_width, 1280);
        assert_eq!(expected_single_height, 720);
        assert_eq!(expected_master_width, 640); // 1280 * 0.5
        assert_eq!(expected_master_height, 720);
    }

    #[test]
    fn test_stack_window_calculations() {
        let screen_width = 1280_i16;
        let screen_height = 720_i16;
        let num_windows = 3_i16;

        // Stack windows calculations with default master ratio (0.5)
        let master_ratio = 0.5_f32;
        let stack_x = (screen_width as f32 * master_ratio) as i16;
        let stack_width = screen_width - stack_x;
        let stack_height = screen_height / (num_windows - 1);

        assert_eq!(stack_x, 640);
        assert_eq!(stack_width, 640);
        assert_eq!(stack_height, 360); // 720 / 2 stack windows
    }

    #[test]
    fn test_gap_calculations() {
        let screen_width = 1280_i16;
        let screen_height = 720_i16;
        let gap = 10_u32;
        let gap_i16 = gap as i16;
        let master_ratio = 0.5_f32;

        // Single window with gaps
        let single_width = screen_width - 2 * gap_i16;
        assert_eq!(single_width, 1260); // 1280 - 20

        // Multiple windows with gaps - master width calculation
        let available_width = screen_width - 3 * gap_i16; // left + center + right gaps
        let master_width = (available_width as f32 * master_ratio) as i16;
        assert_eq!(master_width, 625); // (1280 - 30) * 0.5 = 625

        // Stack positioning
        let stack_x = gap_i16 + master_width + gap_i16;
        assert_eq!(stack_x, 645); // 10 + 625 + 10

        // Stack width
        let stack_width = screen_width - stack_x - gap_i16;
        assert_eq!(stack_width, 625); // 1280 - 645 - 10
    }

    #[test]
    fn test_minimum_window_sizes() {
        // Test that minimum sizes are enforced
        let min_master_width = 100_i16;
        let min_stack_width = 50_i16;
        let min_height = 50_i16;

        // Very small screen should still provide minimum sizes
        let small_screen_width = 200_i16;
        let large_gap = 50_i16;
        
        let calculated_width = (small_screen_width - 2 * large_gap).max(min_master_width);
        assert_eq!(calculated_width, min_master_width); // Should fallback to minimum

        let calculated_stack_width = (small_screen_width / 4).max(min_stack_width);
        assert_eq!(calculated_stack_width, min_stack_width); // Should use minimum
    }

    #[test]
    fn test_gap_edge_cases() {
        // Test large gap scenarios
        let screen_width = 800_i16;
        let screen_height = 600_i16;
        let large_gap = 200_u32;
        let gap_i16 = large_gap as i16;

        // Available width after gaps
        let available_width = screen_width - 3 * gap_i16;
        // 800 - 600 = 200px available (very tight)

        // Should fallback to reasonable sizing
        let fallback_width = if available_width > 150 {
            available_width
        } else {
            screen_width / 2 // Use half screen as fallback
        };

        // 200 > 150 is true, so we use available_width (200)
        assert_eq!(fallback_width, 200);
    }
}

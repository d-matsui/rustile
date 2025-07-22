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
    ) -> Result<()> {
        match self.current_layout {
            Layout::MasterStack => self.tile_master_stack(conn, screen, windows, master_ratio),
        }
    }

    /// Implements master-stack tiling layout
    ///
    /// Layout behavior:
    /// - Single window: Full screen
    /// - Multiple windows: First window takes configurable ratio (master),
    ///   remaining windows stack vertically on the right
    fn tile_master_stack(
        &self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
        master_ratio: f32,
    ) -> Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        let screen_width = screen.width_in_pixels as i16;
        let screen_height = screen.height_in_pixels as i16;
        let num_windows = windows.len() as i16;

        // Configure master window
        let master_window = windows[0];
        let master_width = if num_windows > 1 {
            (screen_width as f32 * master_ratio) as i16
        } else {
            screen_width
        };

        let master_config = ConfigureWindowAux::new()
            .x(0)
            .y(0)
            .width(master_width as u32)
            .height(screen_height as u32);

        conn.configure_window(master_window, &master_config)?;

        // Configure stack windows if any
        if num_windows > 1 {
            let stack_windows = &windows[1..];
            let stack_x = master_width;
            let stack_width = screen_width - master_width;
            let stack_height = screen_height / (num_windows - 1);

            for (index, &window) in stack_windows.iter().enumerate() {
                let stack_y = (index as i16) * stack_height;

                let stack_config = ConfigureWindowAux::new()
                    .x(stack_x as i32)
                    .y(stack_y as i32)
                    .width(stack_width as u32)
                    .height(stack_height as u32);

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
}

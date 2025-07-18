//! Window layout algorithms for the tiling window manager

use crate::config::MASTER_RATIO;
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
    ) -> Result<()> {
        match self.current_layout {
            Layout::MasterStack => self.tile_master_stack(conn, screen, windows),
        }
    }

    /// Implements master-stack tiling layout
    ///
    /// Layout behavior:
    /// - Single window: Full screen
    /// - Multiple windows: First window takes left half (master),
    ///   remaining windows stack vertically on the right
    fn tile_master_stack(
        &self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
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
            (screen_width as f32 * MASTER_RATIO) as i16
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

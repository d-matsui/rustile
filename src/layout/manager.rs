//! Layout manager that coordinates different layout algorithms

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::bsp::BspTree;
use super::types::Layout;

/// Window layout manager
///
/// Future enhancements could include:
/// - Dynamic minimum sizes based on screen resolution
/// - Per-application minimum size rules
/// - Adaptive layout switching based on window count
pub struct LayoutManager {
    current_layout: Layout,
    bsp_tree: BspTree,
}

impl LayoutManager {
    /// Creates a new layout manager with default master-stack layout
    pub fn new() -> Self {
        Self {
            current_layout: Layout::MasterStack,
            bsp_tree: BspTree::new(),
        }
    }

    /// Switches to a different layout algorithm
    pub fn set_layout(&mut self, layout: Layout) {
        self.current_layout = layout;
        // Reset BSP tree when switching layouts
        if matches!(layout, Layout::Bsp) {
            self.bsp_tree = BspTree::new();
        }
    }

    /// Gets the current layout type
    pub fn current_layout(&self) -> Layout {
        self.current_layout
    }

    /// Adds a window to the current layout
    pub fn add_window(&mut self, window: Window, focused_window: Option<Window>, split_ratio: f32) {
        match self.current_layout {
            Layout::MasterStack => {
                // Master-stack doesn't need tree management
            }
            Layout::Bsp => {
                self.bsp_tree.add_window(window, focused_window, split_ratio);
            }
        }
    }

    /// Removes a window from the current layout
    pub fn remove_window(&mut self, window: Window) {
        match self.current_layout {
            Layout::MasterStack => {
                // Master-stack doesn't need tree management
            }
            Layout::Bsp => {
                self.bsp_tree.remove_window(window);
            }
        }
    }

    /// Applies the current layout to arrange windows
    #[allow(clippy::too_many_arguments)]
    pub fn apply_layout<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        screen_width: u16,
        screen_height: u16,
        master_ratio: f32,
        bsp_split_ratio: f32,
        min_window_width: u32,
        min_window_height: u32,
        gap: u32,
    ) -> Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        match self.current_layout {
            Layout::MasterStack => {
                super::master_stack::tile_master_stack(
                    conn,
                    windows,
                    screen_width,
                    screen_height,
                    master_ratio,
                    min_window_width,
                    min_window_height,
                    gap,
                )?;
            }
            Layout::Bsp => {
                self.tile_bsp(
                    conn,
                    windows,
                    focused_window,
                    screen_width,
                    screen_height,
                    bsp_split_ratio,
                    min_window_width,
                    min_window_height,
                    gap,
                )?;
            }
        }

        Ok(())
    }

    /// Rebuild BSP tree from window list and apply layout
    fn tile_bsp<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        screen_width: u16,
        screen_height: u16,
        split_ratio: f32,
        min_window_width: u32,
        min_window_height: u32,
        gap: u32,
    ) -> Result<()> {
        // Rebuild BSP tree from current windows
        super::bsp::rebuild_bsp_tree(&mut self.bsp_tree, windows, focused_window, split_ratio);
        
        // Apply the BSP layout
        super::bsp::tile_bsp_windows(
            conn,
            &self.bsp_tree,
            windows,
            focused_window,
            screen_width,
            screen_height,
            split_ratio,
            min_window_width,
            min_window_height,
            gap,
        )
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}
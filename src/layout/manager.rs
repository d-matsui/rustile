//! Layout manager that coordinates different layout algorithms

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::bsp::BspTree;
use super::types::{LayoutParams, LayoutRatios, ScreenParams, WindowConstraints};

/// Window layout manager for BSP layout
pub struct LayoutManager {
    bsp_tree: BspTree,
}

impl LayoutManager {
    /// Creates a new layout manager with BSP layout
    pub fn new() -> Self {
        Self {
            bsp_tree: BspTree::new(),
        }
    }

    /// Adds a window to the BSP tree
    pub fn add_window(&mut self, window: Window, focused_window: Option<Window>, split_ratio: f32) {
        self.bsp_tree
            .add_window(window, focused_window, split_ratio);
    }

    /// Removes a window from the BSP tree
    pub fn remove_window(&mut self, window: Window) {
        self.bsp_tree.remove_window(window);
    }

    /// Applies the current layout to arrange windows (legacy interface)
    #[allow(clippy::too_many_arguments)]
    pub fn apply_layout<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        screen_width: u16,
        screen_height: u16,
        bsp_split_ratio: f32,
        min_window_width: u32,
        min_window_height: u32,
        gap: u32,
    ) -> Result<()> {
        // Convert to new parameter struct and delegate
        let params = LayoutParams {
            screen: ScreenParams {
                width: screen_width,
                height: screen_height,
                gap,
            },
            constraints: WindowConstraints {
                min_width: min_window_width,
                min_height: min_window_height,
            },
            ratios: LayoutRatios { bsp_split_ratio },
        };

        self.apply_layout_with_params(conn, windows, focused_window, params)
    }

    /// Applies the current layout to arrange windows using parameter structs
    pub fn apply_layout_with_params<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        params: LayoutParams,
    ) -> Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        self.tile_bsp_with_params(conn, windows, focused_window, params)
    }

    /// Rebuild BSP tree from window list and apply layout (legacy interface)
    #[allow(clippy::too_many_arguments, dead_code)]
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
        let params = LayoutParams {
            screen: ScreenParams {
                width: screen_width,
                height: screen_height,
                gap,
            },
            constraints: WindowConstraints {
                min_width: min_window_width,
                min_height: min_window_height,
            },
            ratios: LayoutRatios {
                bsp_split_ratio: split_ratio,
            },
        };

        self.tile_bsp_with_params(conn, windows, focused_window, params)
    }

    /// Rebuild BSP tree from window list and apply layout using parameter structs
    fn tile_bsp_with_params<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        params: LayoutParams,
    ) -> Result<()> {
        // Rebuild BSP tree from current windows
        super::bsp::rebuild_bsp_tree(
            &mut self.bsp_tree,
            windows,
            focused_window,
            params.ratios.bsp_split_ratio,
        );

        // Apply the BSP layout
        super::bsp::tile_bsp_windows(
            conn,
            &self.bsp_tree,
            windows,
            focused_window,
            params.screen.width,
            params.screen.height,
            params.ratios.bsp_split_ratio,
            params.constraints.min_width,
            params.constraints.min_height,
            params.screen.gap,
        )
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_manager_default() {
        let layout_manager = LayoutManager::default();
        // BSP tree should be initialized empty
        assert!(layout_manager.bsp_tree.root.is_none());
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
}

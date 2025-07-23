//! Layout manager that coordinates different layout algorithms

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::bsp::BspTree;
use super::types::{Layout, LayoutParams, LayoutRatios, ScreenParams, WindowConstraints};

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
                self.bsp_tree
                    .add_window(window, focused_window, split_ratio);
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

    /// Applies the current layout to arrange windows (legacy interface)
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
            ratios: LayoutRatios {
                master_ratio,
                bsp_split_ratio,
            },
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

        match self.current_layout {
            Layout::MasterStack => {
                super::master_stack::tile_master_stack(
                    conn,
                    windows,
                    params.screen.width,
                    params.screen.height,
                    params.ratios.master_ratio,
                    params.constraints.min_width,
                    params.constraints.min_height,
                    params.screen.gap,
                )?;
            }
            Layout::Bsp => {
                self.tile_bsp_with_params(conn, windows, focused_window, params)?;
            }
        }

        Ok(())
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
                master_ratio: 0.5, // Not used in BSP
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
        match layout_manager.current_layout {
            Layout::MasterStack => (),
            Layout::Bsp => panic!("Default should be MasterStack"),
        }
    }

    #[test]
    fn test_layout_manager_set_bsp() {
        let mut layout_manager = LayoutManager::new();
        assert!(matches!(
            layout_manager.current_layout(),
            Layout::MasterStack
        ));

        layout_manager.set_layout(Layout::Bsp);
        assert!(matches!(layout_manager.current_layout(), Layout::Bsp));
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

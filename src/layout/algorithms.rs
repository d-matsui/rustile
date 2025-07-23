//! Concrete implementations of layout algorithms using the LayoutAlgorithm trait

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::Window;

use super::bsp::BspTree;
use super::traits::LayoutAlgorithm;
use super::types::LayoutParams;

/// Master-Stack layout algorithm implementation
#[derive(Debug)]
pub struct MasterStackAlgorithm;

impl MasterStackAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

impl LayoutAlgorithm for MasterStackAlgorithm {
    fn name(&self) -> &'static str {
        "master_stack"
    }
    
    fn add_window(&mut self, _window: Window, _focused_window: Option<Window>, _params: &LayoutParams) {
        // Master-stack doesn't need to track window additions
        // The algorithm works directly on the window list provided to apply_layout
    }
    
    fn remove_window(&mut self, _window: Window) {
        // Master-stack doesn't need to track window removals
        // The algorithm works directly on the window list provided to apply_layout
    }
    
    fn apply_layout<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        _focused_window: Option<Window>,
        params: &LayoutParams,
    ) -> Result<()> {
        super::master_stack::tile_master_stack(
            conn,
            windows,
            params.screen.width,
            params.screen.height,
            params.ratios.master_ratio,
            params.constraints.min_width,
            params.constraints.min_height,
            params.screen.gap,
        )
    }
}

impl Default for MasterStackAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

/// BSP (Binary Space Partitioning) layout algorithm implementation
#[derive(Debug)]
pub struct BspAlgorithm {
    bsp_tree: BspTree,
}

impl BspAlgorithm {
    pub fn new() -> Self {
        Self {
            bsp_tree: BspTree::new(),
        }
    }
}

impl LayoutAlgorithm for BspAlgorithm {
    fn name(&self) -> &'static str {
        "bsp"
    }
    
    fn add_window(&mut self, window: Window, focused_window: Option<Window>, params: &LayoutParams) {
        self.bsp_tree.add_window(window, focused_window, params.ratios.bsp_split_ratio);
    }
    
    fn remove_window(&mut self, window: Window) {
        self.bsp_tree.remove_window(window);
    }
    
    fn apply_layout<C: Connection>(
        &mut self,
        conn: &C,
        windows: &[Window],
        focused_window: Option<Window>,
        params: &LayoutParams,
    ) -> Result<()> {
        // Rebuild BSP tree from current windows
        super::bsp::rebuild_bsp_tree(&mut self.bsp_tree, windows, focused_window, params.ratios.bsp_split_ratio);
        
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
    
    fn on_activate(&mut self) {
        // Reset BSP tree when switching to BSP layout
        self.bsp_tree = BspTree::new();
    }
}

impl Default for BspAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}
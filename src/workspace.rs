//! Workspace state management

use crate::bsp::BspTree;
use x11rb::protocol::xproto::Window;

/// Represents a single workspace with independent window layout state
pub struct Workspace {
    bsp_tree: BspTree,
    focused_window: Option<Window>,
    fullscreen_window: Option<Window>,
    zoomed_window: Option<Window>,
}

impl Workspace {
    /// Creates a new empty workspace
    pub fn new() -> Self {
        Self {
            bsp_tree: BspTree::new(),
            focused_window: None,
            fullscreen_window: None,
            zoomed_window: None,
        }
    }

    /// Gets the focused window
    pub fn focused_window(&self) -> Option<Window> {
        self.focused_window
    }

    /// Sets the focused window
    pub fn set_focused_window(&mut self, window: Option<Window>) {
        self.focused_window = window;
    }

    /// Gets the fullscreen window
    pub fn fullscreen_window(&self) -> Option<Window> {
        self.fullscreen_window
    }

    /// Sets the fullscreen window
    pub fn set_fullscreen_window(&mut self, window: Option<Window>) {
        self.fullscreen_window = window;
    }

    /// Gets the zoomed window
    pub fn zoomed_window(&self) -> Option<Window> {
        self.zoomed_window
    }

    /// Sets the zoomed window
    pub fn set_zoomed_window(&mut self, window: Option<Window>) {
        self.zoomed_window = window;
    }

    /// Gets all windows in this workspace
    pub fn get_all_windows(&self) -> Vec<Window> {
        self.bsp_tree.all_windows()
    }

    /// Checks if this workspace contains a window
    pub fn has_window(&self, window: Window) -> bool {
        self.bsp_tree.has_window(window)
    }

    /// Gets the first window in the layout, or None if empty
    pub fn get_first_window(&self) -> Option<Window> {
        self.get_all_windows().first().copied()
    }

    /// Adds a window to this workspace
    pub fn add_window(&mut self, window: Window) {
        // Clear zoom when adding new window (consistent with WindowState)
        self.zoomed_window = None;
        // Use default split ratio 0.5 (will need Config access in future)
        self.bsp_tree.add_window(window, self.focused_window, 0.5);
    }

    /// Removes a window from this workspace
    pub fn remove_window(&mut self, window: Window) {
        // Clear zoom if removing the zoomed window
        if self.zoomed_window == Some(window) {
            self.zoomed_window = None;
        }
        self.bsp_tree.remove_window(window);
    }

    /// Clears fullscreen state
    pub fn clear_fullscreen(&mut self) {
        self.fullscreen_window = None;
    }

    /// Clears focused window
    pub fn clear_focus(&mut self) {
        self.focused_window = None;
    }

    /// Gets a reference to the BSP tree
    pub fn bsp_tree(&self) -> &BspTree {
        &self.bsp_tree
    }

    /// Gets a mutable reference to the BSP tree
    pub fn bsp_tree_mut(&mut self) -> &mut BspTree {
        &mut self.bsp_tree
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

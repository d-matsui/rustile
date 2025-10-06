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

    /// Gets a reference to the BSP tree
    pub fn bsp_tree(&self) -> &BspTree {
        &self.bsp_tree
    }

    /// Gets a mutable reference to the BSP tree
    pub fn bsp_tree_mut(&mut self) -> &mut BspTree {
        &mut self.bsp_tree
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

    /// Gets the total number of windows in this workspace
    pub fn window_count(&self) -> usize {
        self.bsp_tree.window_count()
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

    /// Gets the next window in the layout
    pub fn next_window(&self, current: Window) -> Option<Window> {
        self.bsp_tree.next_window(current)
    }

    /// Gets the previous window in the layout
    pub fn prev_window(&self, current: Window) -> Option<Window> {
        self.bsp_tree.prev_window(current)
    }

    /// Swaps two windows in the BSP tree
    pub fn swap_windows(&mut self, window1: Window, window2: Window) {
        self.bsp_tree.swap_windows(window1, window2);
    }

    /// Rotates a window in the BSP tree
    pub fn rotate_window(&mut self, window: Window) -> bool {
        // Clear zoom when rotating (consistent with WindowState)
        self.zoomed_window = None;
        self.bsp_tree.rotate_window(window)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new();

        // Verify all state is initialized to None/empty
        assert!(workspace.focused_window.is_none());
        assert!(workspace.fullscreen_window.is_none());
        assert!(workspace.zoomed_window.is_none());
        assert_eq!(workspace.bsp_tree.window_count(), 0);
    }

    #[test]
    fn test_workspace_default() {
        let workspace = Workspace::default();

        assert!(workspace.focused_window.is_none());
        assert_eq!(workspace.bsp_tree.window_count(), 0);
    }

    #[test]
    fn test_workspace_state_management() {
        let mut workspace = Workspace::new();

        // Test focus state
        assert!(workspace.focused_window().is_none());
        workspace.set_focused_window(Some(10));
        assert_eq!(workspace.focused_window(), Some(10));
        workspace.set_focused_window(None);
        assert!(workspace.focused_window().is_none());

        // Test fullscreen state
        assert!(workspace.fullscreen_window().is_none());
        workspace.set_fullscreen_window(Some(20));
        assert_eq!(workspace.fullscreen_window(), Some(20));

        // Test zoom state
        assert!(workspace.zoomed_window().is_none());
        workspace.set_zoomed_window(Some(30));
        assert_eq!(workspace.zoomed_window(), Some(30));
    }

    #[test]
    fn test_workspace_bsp_tree_access() {
        let mut workspace = Workspace::new();

        // Read-only access
        assert_eq!(workspace.bsp_tree().window_count(), 0);

        // Mutable access
        workspace.bsp_tree_mut().add_window(100, None, 0.5);
        assert_eq!(workspace.bsp_tree().window_count(), 1);
    }

    #[test]
    fn test_workspace_window_queries() {
        let mut workspace = Workspace::new();

        // Empty workspace
        assert_eq!(workspace.get_all_windows(), Vec::<Window>::new());
        assert_eq!(workspace.window_count(), 0);
        assert!(!workspace.has_window(100));
        assert_eq!(workspace.get_first_window(), None);

        // Add windows
        workspace.add_window(100);
        assert_eq!(workspace.window_count(), 1);
        assert!(workspace.has_window(100));
        assert_eq!(workspace.get_first_window(), Some(100));

        workspace.set_focused_window(Some(100));
        workspace.add_window(200);
        assert_eq!(workspace.window_count(), 2);
        assert!(workspace.has_window(200));
        assert_eq!(workspace.get_all_windows().len(), 2);
    }

    #[test]
    fn test_workspace_window_add_remove() {
        let mut workspace = Workspace::new();

        // Add first window
        workspace.add_window(100);
        assert_eq!(workspace.window_count(), 1);

        // Add second window (need focused window set)
        workspace.set_focused_window(Some(100));
        workspace.add_window(200);
        assert_eq!(workspace.window_count(), 2);

        // Remove window
        workspace.remove_window(100);
        assert_eq!(workspace.window_count(), 1);
        assert!(!workspace.has_window(100));
        assert!(workspace.has_window(200));

        // Remove last window
        workspace.remove_window(200);
        assert_eq!(workspace.window_count(), 0);
    }

    #[test]
    fn test_workspace_window_navigation() {
        let mut workspace = Workspace::new();

        // Add windows
        workspace.add_window(100);
        workspace.set_focused_window(Some(100));
        workspace.add_window(200);
        workspace.set_focused_window(Some(200));
        workspace.add_window(300);

        // Navigate
        assert_eq!(workspace.next_window(100), Some(200));
        assert_eq!(workspace.next_window(200), Some(300));
        assert_eq!(workspace.prev_window(200), Some(100));
        assert_eq!(workspace.prev_window(300), Some(200));
    }

    #[test]
    fn test_workspace_swap_windows() {
        let mut workspace = Workspace::new();

        // Add windows
        workspace.add_window(100);
        workspace.set_focused_window(Some(100));
        workspace.add_window(200);

        let windows_before = workspace.get_all_windows();
        assert_eq!(windows_before[0], 100);
        assert_eq!(windows_before[1], 200);

        // Swap windows
        workspace.swap_windows(100, 200);

        let windows_after = workspace.get_all_windows();
        assert_eq!(windows_after[0], 200);
        assert_eq!(windows_after[1], 100);
    }

    #[test]
    fn test_workspace_rotate_window() {
        let mut workspace = Workspace::new();

        // Add windows to create a structure that can be rotated
        workspace.add_window(100);
        workspace.set_focused_window(Some(100));
        workspace.add_window(200);

        // Rotate (returns true if successful)
        let rotated = workspace.rotate_window(100);
        assert!(rotated);
    }
}

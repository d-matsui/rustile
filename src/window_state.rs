//! Window state management and queries

use std::collections::HashSet;
use x11rb::protocol::xproto::Window;

use crate::bsp::BspTree;
use crate::config::Config;
use crate::window_renderer::{BspRect, LayoutParams, WindowGeometry};

/// Manages window state and provides queries
pub struct WindowState {
    focused_window: Option<Window>,
    bsp_tree: BspTree,
    fullscreen_window: Option<Window>,
    zoomed_window: Option<Window>,
    intentionally_unmapped: HashSet<Window>,
    config: Config,
    screen_num: usize,
}

impl WindowState {
    /// Creates a new window state manager
    pub fn new(config: Config, screen_num: usize) -> Self {
        Self {
            focused_window: None,
            bsp_tree: BspTree::new(),
            fullscreen_window: None,
            zoomed_window: None,
            intentionally_unmapped: HashSet::new(),
            config,
            screen_num,
        }
    }

    /// Gets the currently focused window
    pub fn get_focused_window(&self) -> Option<Window> {
        self.focused_window
    }

    /// Sets the focused window
    pub fn set_focused_window(&mut self, window: Option<Window>) {
        self.focused_window = window;
    }

    /// Clears the focused window
    pub fn clear_focus(&mut self) {
        self.focused_window = None;
    }

    /// Gets the current fullscreen window
    pub fn get_fullscreen_window(&self) -> Option<Window> {
        self.fullscreen_window
    }

    /// Sets the fullscreen window
    pub fn set_fullscreen_window(&mut self, window: Option<Window>) {
        self.fullscreen_window = window;
    }

    /// Clears fullscreen state
    pub fn clear_fullscreen(&mut self) {
        self.fullscreen_window = None;
    }

    /// Checks if we're in fullscreen mode
    pub fn is_in_fullscreen_mode(&self) -> bool {
        self.fullscreen_window.is_some()
    }

    /// Gets the current zoomed window
    pub fn get_zoomed_window(&self) -> Option<Window> {
        self.zoomed_window
    }

    /// Sets the zoomed window
    pub fn set_zoomed_window(&mut self, window: Option<Window>) {
        self.zoomed_window = window;
    }

    /// Clears zoom state
    pub fn clear_zoom(&mut self) {
        self.zoomed_window = None;
    }

    /// Gets all windows currently managed by the layout
    pub fn get_all_windows(&self) -> Vec<Window> {
        self.bsp_tree.all_windows()
    }

    /// Gets the total number of windows in the layout
    pub fn window_count(&self) -> usize {
        self.bsp_tree.window_count()
    }

    /// Checks if a window is managed by the layout
    pub fn has_window(&self, window: Window) -> bool {
        self.bsp_tree.has_window(window)
    }

    /// Gets the first window in the layout, or None if empty
    pub fn get_first_window(&self) -> Option<Window> {
        self.get_all_windows().first().copied()
    }

    /// Gets the next window in the layout
    pub fn next_window(&self, current: Window) -> Option<Window> {
        self.bsp_tree.next_window(current)
    }

    /// Gets the previous window in the layout
    pub fn prev_window(&self, current: Window) -> Option<Window> {
        self.bsp_tree.prev_window(current)
    }

    /// Checks if a window is intentionally unmapped
    pub fn is_intentionally_unmapped(&self, window: Window) -> bool {
        self.intentionally_unmapped.contains(&window)
    }

    /// Marks a window as intentionally unmapped
    pub fn mark_intentionally_unmapped(&mut self, window: Window) {
        self.intentionally_unmapped.insert(window);
    }

    /// Removes a window from the intentionally unmapped set
    pub fn remove_intentionally_unmapped(&mut self, window: Window) {
        self.intentionally_unmapped.remove(&window);
    }

    /// Adds a window to the layout manager
    pub fn add_window_to_layout(&mut self, window: Window) {
        // Clear zoom when adding new window (as per ADR-010)
        self.clear_zoom();
        self.bsp_tree
            .add_window(window, self.focused_window, self.config.bsp_split_ratio());
    }

    /// Removes a window from the layout manager
    pub fn remove_window_from_layout(&mut self, window: Window) {
        // Clear zoom if removing the zoomed window
        if self.zoomed_window == Some(window) {
            self.clear_zoom();
        }
        self.bsp_tree.remove_window(window);
    }

    /// Swaps two windows in the BSP tree
    pub fn swap_windows(&mut self, window1: Window, window2: Window) {
        self.bsp_tree.swap_windows(window1, window2);
    }

    /// Rotates a window in the BSP tree
    pub fn rotate_window(&mut self, window: Window) -> bool {
        // Clear zoom when rotating (as per ADR-010)
        self.clear_zoom();
        self.bsp_tree.rotate_window(window)
    }

    /// Gets the border width from config
    pub fn border_width(&self) -> u32 {
        self.config.border_width()
    }

    /// Gets the screen number
    pub fn screen_num(&self) -> usize {
        self.screen_num
    }

    /// Gets a reference to the BSP tree
    pub fn bsp_tree(&self) -> &BspTree {
        &self.bsp_tree
    }

    /// Creates layout parameters bundle from config - helper to reduce parameter duplication
    pub fn layout_params(&self) -> LayoutParams {
        LayoutParams {
            min_window_width: self.config.min_window_width(),
            min_window_height: self.config.min_window_height(),
            gap: self.config.gap(),
        }
    }

    /// Returns appropriate border color based on window focus state - helper to reduce duplication
    pub fn border_color_for_window(&self, window: Window) -> u32 {
        if Some(window) == self.focused_window {
            self.config.focused_border_color()
        } else {
            self.config.unfocused_border_color()
        }
    }

    /// Calculates window geometries from the BSP tree (pure calculation - no X11 calls)
    pub fn calculate_window_geometries(
        &self,
        screen_width: u16,
        screen_height: u16,
    ) -> Vec<WindowGeometry> {
        let params = self.layout_params();
        crate::window_renderer::calculate_bsp_geometries(
            &self.bsp_tree,
            screen_width,
            screen_height,
            params,
        )
    }

    /// Calculates the screen rectangle with gap and minimum size constraints
    pub fn calculate_screen_rect(&self, screen_width: u16, screen_height: u16) -> BspRect {
        let params = self.layout_params();
        BspRect {
            x: params.gap as i32,
            y: params.gap as i32,
            width: (screen_width as i32 - 2 * params.gap as i32)
                .max(params.min_window_width as i32),
            height: (screen_height as i32 - 2 * params.gap as i32)
                .max(params.min_window_height as i32),
        }
    }

    /// Balances the BSP tree by calculating optimal split ratios
    pub fn balance_tree(&mut self) {
        self.bsp_tree.balance_tree();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_window_state_creation() {
        let config = create_test_config();
        let state = WindowState::new(config, 0);

        assert_eq!(state.get_focused_window(), None);
        assert_eq!(state.get_fullscreen_window(), None);
        assert_eq!(state.window_count(), 0);
        assert!(!state.is_in_fullscreen_mode());
    }

    #[test]
    fn test_focus_management() {
        let config = create_test_config();
        let mut state = WindowState::new(config, 0);

        // Initially no focus
        assert_eq!(state.get_focused_window(), None);

        // Set focus
        state.set_focused_window(Some(10));
        assert_eq!(state.get_focused_window(), Some(10));

        // Clear focus
        state.clear_focus();
        assert_eq!(state.get_focused_window(), None);
    }

    #[test]
    fn test_fullscreen_management() {
        let config = create_test_config();
        let mut state = WindowState::new(config, 0);

        // Initially not fullscreen
        assert_eq!(state.get_fullscreen_window(), None);
        assert!(!state.is_in_fullscreen_mode());

        // Set fullscreen
        state.set_fullscreen_window(Some(20));
        assert_eq!(state.get_fullscreen_window(), Some(20));
        assert!(state.is_in_fullscreen_mode());

        // Clear fullscreen
        state.clear_fullscreen();
        assert_eq!(state.get_fullscreen_window(), None);
        assert!(!state.is_in_fullscreen_mode());
    }

    #[test]
    fn test_intentionally_unmapped_tracking() {
        let config = create_test_config();
        let mut state = WindowState::new(config, 0);

        // Initially not unmapped
        assert!(!state.is_intentionally_unmapped(30));

        // Mark as unmapped
        state.mark_intentionally_unmapped(30);
        assert!(state.is_intentionally_unmapped(30));

        // Remove from unmapped
        state.remove_intentionally_unmapped(30);
        assert!(!state.is_intentionally_unmapped(30));
    }

    #[test]
    fn test_window_layout_management() {
        let config = create_test_config();
        let mut state = WindowState::new(config, 0);

        // Initially empty
        assert_eq!(state.window_count(), 0);
        assert!(!state.has_window(40));
        assert_eq!(state.get_first_window(), None);

        // Add window
        state.add_window_to_layout(40);
        assert_eq!(state.window_count(), 1);
        assert!(state.has_window(40));
        assert_eq!(state.get_first_window(), Some(40));

        // Set focus to enable adding more windows
        state.set_focused_window(Some(40));

        // Add second window
        state.add_window_to_layout(50);
        assert_eq!(state.window_count(), 2);
        assert!(state.has_window(50));

        // Remove window
        state.remove_window_from_layout(40);
        assert_eq!(state.window_count(), 1);
        assert!(!state.has_window(40));
        assert!(state.has_window(50));
    }

    #[test]
    fn test_config_access() {
        let config = create_test_config();
        let state = WindowState::new(config, 1);

        // Test config access methods
        assert_eq!(state.border_width(), state.config.border_width());
        assert_eq!(
            state.config.unfocused_border_color(),
            state.config.unfocused_border_color()
        );
        assert_eq!(state.screen_num(), 1);
    }

    #[test]
    fn test_border_color_selection() {
        let config = create_test_config();
        let mut state = WindowState::new(config, 0);

        // Add windows
        state.add_window_to_layout(60);
        state.add_window_to_layout(70);

        // No focus - should get unfocused color
        assert_eq!(
            state.border_color_for_window(60),
            state.config.unfocused_border_color()
        );
        assert_eq!(
            state.border_color_for_window(70),
            state.config.unfocused_border_color()
        );

        // Set focus - focused window should get focused color
        state.set_focused_window(Some(60));
        assert_eq!(
            state.border_color_for_window(60),
            state.config.focused_border_color()
        );
        assert_eq!(
            state.border_color_for_window(70),
            state.config.unfocused_border_color()
        );
    }
}

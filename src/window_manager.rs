//! Window manager module with focused functionality
//!
//! This module organizes window management functionality into logical sections:
//! - Core: Initialization, main loop, and configuration
//! - Events: X11 event handling and dispatching
//! - Focus: Window focus management and visual indicators
//! - Window Operations: Window lifecycle and manipulation

use anyhow::Result;
#[cfg(debug_assertions)]
use tracing::debug;
use tracing::{error, info};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::bsp::{BspTree, LayoutParams};
use crate::config::Config;
use crate::keyboard::KeyboardManager;

// =============================================================================
// Core Window Manager Structure and Initialization
// =============================================================================

/// Main window manager structure
pub struct WindowManager<C: Connection> {
    /// X11 connection
    pub(crate) conn: C,
    /// Screen information
    pub(crate) screen_num: usize,
    /// Currently focused window
    pub(crate) focused_window: Option<Window>,
    /// BSP tree for window arrangement (single source of truth for window layout)
    pub(crate) bsp_tree: BspTree,
    /// Keyboard manager for shortcuts
    pub(crate) keyboard_manager: KeyboardManager,
    /// Configuration
    pub(crate) config: Config,
    /// Currently fullscreen window (if any)
    pub(crate) fullscreen_window: Option<Window>,
    /// Windows we intentionally unmapped (to distinguish from user-closed windows)
    pub(crate) intentionally_unmapped: std::collections::HashSet<Window>,
}

impl<C: Connection> WindowManager<C> {
    /// Creates a new window manager instance
    pub fn new(conn: C, screen_num: usize) -> Result<Self> {
        // Load configuration
        let config = Config::load()?;
        info!(
            "Loaded configuration with {} shortcuts",
            config.shortcuts().len()
        );

        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;

        // Initialize keyboard manager
        let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

        // Register as window manager
        let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);

        if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
            error!("Another window manager is already running: {:?}", e);
            return Err(anyhow::anyhow!(
                "Failed to become window manager. Is another WM running?"
            ));
        }

        info!("Successfully became the window manager");

        // Register keyboard shortcuts from config
        keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

        // Create BSP tree for window layout
        let bsp_tree = BspTree::new();
        info!("Using BSP layout algorithm");

        Ok(Self {
            conn,
            screen_num,
            focused_window: None,
            bsp_tree,
            keyboard_manager,
            config,
            fullscreen_window: None,
            intentionally_unmapped: std::collections::HashSet::new(),
        })
    }

    /// Runs the main event loop
    pub fn run(mut self) -> Result<()> {
        info!("Starting window manager event loop");

        loop {
            self.conn.flush()?;
            let event = self.conn.wait_for_event()?;

            if let Err(e) = self.handle_event(event) {
                error!("Error handling event: {:?}", e);
                // Continue running despite errors
            }
        }
    }
}

// =============================================================================
// Focus Management
// =============================================================================

impl<C: Connection> WindowManager<C> {
    /// Sets focus to a specific window
    pub(crate) fn set_focus(&mut self, window: Window) -> Result<()> {
        if !self.has_window(window) {
            return Ok(());
        }

        // Set X11 input focus
        self.conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;

        // Update focus state
        self.focused_window = Some(window);

        // Update window borders
        self.update_window_borders()?;

        #[cfg(debug_assertions)]
        debug!("Focus set to window: {:?}", window);
        Ok(())
    }

    /// Updates window borders based on focus state
    fn update_window_borders(&self) -> Result<()> {
        for &window in &self.get_all_windows() {
            let border_color = self.border_color_for_window(window);
            self.configure_window_border(window, border_color)?;
        }
        Ok(())
    }

    /// Focuses the next window in the stack
    pub fn focus_next(&mut self) -> Result<()> {
        if self.window_count() == 0 {
            return Ok(());
        }

        let next_window = if let Some(current) = self.focused_window {
            // Use BSP tree navigation
            self.bsp_tree.next_window(current).unwrap_or(current)
        } else {
            // Focus first window if none focused
            match self.get_first_window() {
                Some(window) => window,
                None => return Ok(()), // No windows to focus
            }
        };

        // Exit fullscreen if trying to focus a different window
        if self.fullscreen_window.is_some() && self.fullscreen_window != Some(next_window) {
            info!("Exiting fullscreen mode to focus different window");
            self.fullscreen_window = None;
            self.apply_layout()?;
        }

        self.set_focus(next_window)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses the previous window in the stack
    pub fn focus_prev(&mut self) -> Result<()> {
        if self.window_count() == 0 {
            return Ok(());
        }

        let prev_window = if let Some(current) = self.focused_window {
            // Use BSP tree navigation
            self.bsp_tree.prev_window(current).unwrap_or(current)
        } else {
            // Focus first window if none focused
            match self.get_first_window() {
                Some(window) => window,
                None => return Ok(()), // No windows to focus
            }
        };

        // Exit fullscreen if trying to focus a different window
        if self.fullscreen_window.is_some() && self.fullscreen_window != Some(prev_window) {
            info!("Exiting fullscreen mode to focus different window");
            self.fullscreen_window = None;
            self.apply_layout()?;
        }

        self.set_focus(prev_window)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }

    /// Configures window border color and width - helper to reduce duplication
    pub(crate) fn configure_window_border(&self, window: Window, border_color: u32) -> Result<()> {
        let border_aux = ChangeWindowAttributesAux::new().border_pixel(border_color);
        self.conn.change_window_attributes(window, &border_aux)?;

        let config_aux = ConfigureWindowAux::new().border_width(self.config.border_width());
        self.conn.configure_window(window, &config_aux)?;

        Ok(())
    }

    /// Creates layout parameters bundle from config - helper to reduce parameter duplication
    fn layout_params(&self) -> LayoutParams {
        LayoutParams {
            min_window_width: self.config.min_window_width(),
            min_window_height: self.config.min_window_height(),
            gap: self.config.gap(),
        }
    }

    /// Returns appropriate border color based on window focus state - helper to reduce duplication
    fn border_color_for_window(&self, window: Window) -> u32 {
        if Some(window) == self.focused_window {
            self.config.focused_border_color()
        } else {
            self.config.unfocused_border_color()
        }
    }
}

// =============================================================================
// Window Operations and Layout Integration
// =============================================================================

/// Direction for window swapping operations
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,
    Previous,
}

impl<C: Connection> WindowManager<C> {
    /// Adds a window to the layout manager
    pub(crate) fn add_window_to_layout(&mut self, window: Window) {
        self.bsp_tree
            .add_window(window, self.focused_window, self.config.bsp_split_ratio());
    }

    /// Removes a window from the layout manager
    pub(crate) fn remove_window_from_layout(&mut self, window: Window) {
        self.bsp_tree.remove_window(window);
    }

    /// Gets all windows currently managed by the layout
    fn get_all_windows(&self) -> Vec<Window> {
        self.bsp_tree.all_windows()
    }

    /// Gets the total number of windows in the layout
    fn window_count(&self) -> usize {
        self.bsp_tree.window_count()
    }

    /// Checks if a window is managed by the layout
    pub(crate) fn has_window(&self, window: Window) -> bool {
        self.bsp_tree.has_window(window)
    }

    /// Gets the first window in the layout, or None if empty
    pub(crate) fn get_first_window(&self) -> Option<Window> {
        self.get_all_windows().first().copied()
    }

    /// Applies the current BSP tree layout without rebuilding the tree
    pub(crate) fn apply_layout(&mut self) -> Result<()> {
        if self.window_count() == 0 {
            return Ok(());
        }

        // If we're in fullscreen mode, apply fullscreen layout instead
        if self.fullscreen_window.is_some() {
            return self.apply_fullscreen_layout();
        }

        let setup = self.conn.setup();
        let screen = &setup.roots[self.screen_num];

        // Ensure all windows are mapped (visible) and have borders when not in fullscreen
        let border_width = self.config.border_width();
        for &window in &self.get_all_windows() {
            self.conn.map_window(window)?;
            // Remove from intentionally unmapped set when restoring
            self.intentionally_unmapped.remove(&window);
            // Restore border width
            self.conn.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;
        }

        // Calculate window geometries from existing BSP tree (preserves tree structure)
        let params = self.layout_params();
        let geometries = crate::bsp::calculate_bsp_geometries(
            &self.bsp_tree,
            screen.width_in_pixels,
            screen.height_in_pixels,
            params,
        );

        // Apply calculated geometries and update borders
        for geometry in &geometries {
            let border_color = self.border_color_for_window(geometry.window);

            // Set border color
            self.conn.change_window_attributes(
                geometry.window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;

            // Set geometry and border width
            self.conn.configure_window(
                geometry.window,
                &ConfigureWindowAux::new()
                    .x(geometry.x)
                    .y(geometry.y)
                    .width(geometry.width)
                    .height(geometry.height)
                    .border_width(border_width),
            )?;
        }

        // Update focus hints and raise focused window
        if let Some(focused) = self.focused_window {
            self.conn.configure_window(
                focused,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        self.conn.flush()?;

        #[cfg(debug_assertions)]
        tracing::debug!(
            "Applied existing BSP tree layout to {} windows",
            geometries.len()
        );

        Ok(())
    }

    /// Destroys (closes) the currently focused window
    pub fn destroy_focused_window(&mut self) -> Result<()> {
        if let Some(focused) = self.focused_window {
            info!("Destroying focused window: {:?}", focused);

            // Try to close the window gracefully first using WM_DELETE_WINDOW
            // If that fails, kill it forcefully
            self.close_window_gracefully(focused)
                .or_else(|_| self.kill_window_forcefully(focused))?;
        } else {
            info!("No focused window to destroy");
        }
        Ok(())
    }

    /// Attempts to close a window gracefully using WM_DELETE_WINDOW protocol
    fn close_window_gracefully(&self, window: x11rb::protocol::xproto::Window) -> Result<()> {
        use x11rb::protocol::xproto::*;

        // Get WM_DELETE_WINDOW and WM_PROTOCOLS atoms
        let wm_protocols = self.conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;
        let wm_delete_window = self
            .conn
            .intern_atom(false, b"WM_DELETE_WINDOW")?
            .reply()?
            .atom;

        // Check if the window supports WM_DELETE_WINDOW
        let protocols = self
            .conn
            .get_property(false, window, wm_protocols, AtomEnum::ATOM, 0, 1024)?
            .reply()?;

        if protocols.format == 32 {
            let atoms: Vec<Atom> = protocols
                .value32()
                .ok_or_else(|| anyhow::anyhow!("Failed to parse WM_PROTOCOLS"))?
                .collect();

            if atoms.contains(&wm_delete_window) {
                // Window supports graceful close, send WM_DELETE_WINDOW message
                let event = ClientMessageEvent {
                    response_type: CLIENT_MESSAGE_EVENT,
                    format: 32,
                    sequence: 0,
                    window,
                    type_: wm_protocols,
                    data: ClientMessageData::from([wm_delete_window, x11rb::CURRENT_TIME, 0, 0, 0]),
                };

                self.conn
                    .send_event(false, window, EventMask::NO_EVENT, event)?;
                self.conn.flush()?;
                info!("Sent WM_DELETE_WINDOW message to window {:?}", window);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "Window does not support WM_DELETE_WINDOW protocol"
        ))
    }

    /// Forcefully kills a window using XKillClient
    fn kill_window_forcefully(&self, window: x11rb::protocol::xproto::Window) -> Result<()> {
        info!("Forcefully killing window {:?}", window);
        self.conn.kill_client(window)?;
        self.conn.flush()?;
        Ok(())
    }

    /// Swaps the currently focused window with the next window in the layout
    pub fn swap_window_next(&mut self) -> Result<()> {
        self.swap_window_direction(SwapDirection::Next)
    }

    /// Swaps the currently focused window with the previous window in the layout
    pub fn swap_window_prev(&mut self) -> Result<()> {
        self.swap_window_direction(SwapDirection::Previous)
    }

    /// Helper method to swap windows in a given direction
    fn swap_window_direction(&mut self, direction: SwapDirection) -> Result<()> {
        if self.window_count() < 2 {
            return Ok(());
        }

        // Exit fullscreen if active, then perform swap
        if self.fullscreen_window.is_some() {
            info!("Exiting fullscreen for window swap");
            self.fullscreen_window = None;
        }

        if let Some(focused) = self.focused_window {
            let target_window = match direction {
                SwapDirection::Next => self.bsp_tree.next_window(focused),
                SwapDirection::Previous => self.bsp_tree.prev_window(focused),
            };

            if let Some(target_window) = target_window {
                // Swap windows in the BSP tree
                self.bsp_tree.swap_windows(focused, target_window);

                let direction_str = match direction {
                    SwapDirection::Next => "next",
                    SwapDirection::Previous => "previous",
                };

                info!(
                    "Swapped window {:?} with {} window {:?}",
                    focused, direction_str, target_window
                );

                // Apply layout
                self.apply_layout()?;
            }
        }
        Ok(())
    }

    /// Toggles fullscreen mode for the focused window
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        let focused = match self.focused_window {
            Some(window) => window,
            None => {
                info!("No window focused for fullscreen toggle");
                return Ok(());
            }
        };

        // Check if we're currently in fullscreen mode
        if let Some(fullscreen) = self.fullscreen_window {
            if fullscreen == focused {
                // Exit fullscreen mode
                info!("Exiting fullscreen mode for window {:?}", focused);
                self.fullscreen_window = None;
                self.apply_layout()?;
            } else {
                // Different window wants fullscreen, switch to it
                info!(
                    "Switching fullscreen from {:?} to {:?}",
                    fullscreen, focused
                );
                self.fullscreen_window = Some(focused);
                self.apply_fullscreen_layout()?;
            }
        } else {
            // Enter fullscreen mode
            info!("Entering fullscreen mode for window {:?}", focused);
            self.fullscreen_window = Some(focused);
            self.apply_fullscreen_layout()?;
        }

        Ok(())
    }

    /// Applies fullscreen layout - window takes entire screen
    fn apply_fullscreen_layout(&mut self) -> Result<()> {
        if let Some(fullscreen) = self.fullscreen_window {
            let setup = self.conn.setup();
            let screen = &setup.roots[self.screen_num];

            // Ensure the fullscreen window is mapped (visible)
            self.conn.map_window(fullscreen)?;

            // Configure fullscreen window to cover entire screen (no gaps, no borders)
            let config = ConfigureWindowAux::new()
                .x(0)
                .y(0)
                .width(u32::from(screen.width_in_pixels))
                .height(u32::from(screen.height_in_pixels))
                .border_width(0);

            self.conn.configure_window(fullscreen, &config)?;

            // Hide all other windows (mark as intentionally unmapped)
            for &window in &self.get_all_windows() {
                if window != fullscreen {
                    // Mark as intentionally unmapped BEFORE unmapping
                    self.intentionally_unmapped.insert(window);
                    self.conn.unmap_window(window)?;
                }
            }

            // Ensure fullscreen window is on top
            self.conn.configure_window(
                fullscreen,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;

            self.conn.flush()?;
        }

        Ok(())
    }

    /// Rotates the focused window by flipping its parent split direction
    pub fn rotate_windows(&mut self) -> Result<()> {
        let focused = match self.focused_window {
            Some(window) => window,
            None => {
                info!("No window focused for rotation");
                return Ok(());
            }
        };

        // Cannot rotate in fullscreen mode
        if self.fullscreen_window.is_some() {
            info!("Cannot rotate in fullscreen mode");
            return Ok(());
        }

        // Need at least 2 windows to rotate
        if self.window_count() < 2 {
            info!("Not enough windows to rotate (need at least 2)");
            return Ok(());
        }

        info!(
            "Rotating parent split direction for focused window {:?}",
            focused
        );

        // Debug: Print tree structure before rotation
        tracing::info!("BSP tree before rotation: {:?}", self.bsp_tree);

        // Rotate the focused window in the BSP tree
        let rotated = self.bsp_tree.rotate_window(focused);

        if rotated {
            // Debug: Print tree structure after rotation
            tracing::info!("BSP tree after rotation: {:?}", self.bsp_tree);

            // Apply the existing rotated tree layout (without rebuilding)
            self.apply_layout()?;
            info!("Window rotation completed for window {:?}", focused);
        } else {
            info!(
                "No rotation performed - window {:?} may be root or not found",
                focused
            );
        }

        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use x11rb::protocol::xproto::Window;

    // Test helper functions for window management logic
    // These test the core algorithms without requiring X11 connections

    /// Helper to test focus_next logic
    fn test_focus_next_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }

        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let next_idx = (current_idx + 1) % windows.len();
                Some(windows[next_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }

    /// Helper to test focus_prev logic
    fn test_focus_prev_logic(windows: &[Window], current: Option<Window>) -> Option<Window> {
        if windows.is_empty() {
            return None;
        }

        if let Some(current) = current {
            if let Some(current_idx) = windows.iter().position(|&w| w == current) {
                let prev_idx = if current_idx == 0 {
                    windows.len() - 1
                } else {
                    current_idx - 1
                };
                Some(windows[prev_idx])
            } else {
                Some(windows[0])
            }
        } else {
            Some(windows[0])
        }
    }

    #[test]
    fn test_focus_next_empty_windows() {
        let windows = vec![];
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_focus_next_single_window() {
        let windows = vec![10];

        // Should focus the only window when none focused
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));

        // Should stay on same window when already focused
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_focus_next_multiple_windows() {
        let windows = vec![10, 20, 30];

        // From no focus, should focus first window
        let result = test_focus_next_logic(&windows, None);
        assert_eq!(result, Some(10));

        // From first window, should focus second
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(20));

        // From second window, should focus third
        let result = test_focus_next_logic(&windows, Some(20));
        assert_eq!(result, Some(30));

        // From last window, should wrap to first
        let result = test_focus_next_logic(&windows, Some(30));
        assert_eq!(result, Some(10));

        // From non-existent window, should focus first
        let result = test_focus_next_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_focus_prev_empty_windows() {
        let windows = vec![];
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_focus_prev_single_window() {
        let windows = vec![10];

        // Should focus the only window when none focused
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));

        // Should stay on same window when already focused
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_focus_prev_multiple_windows() {
        let windows = vec![10, 20, 30];

        // From no focus, should focus first window
        let result = test_focus_prev_logic(&windows, None);
        assert_eq!(result, Some(10));

        // From first window, should wrap to last
        let result = test_focus_prev_logic(&windows, Some(10));
        assert_eq!(result, Some(30));

        // From second window, should focus first
        let result = test_focus_prev_logic(&windows, Some(20));
        assert_eq!(result, Some(10));

        // From last window, should focus second
        let result = test_focus_prev_logic(&windows, Some(30));
        assert_eq!(result, Some(20));

        // From non-existent window, should focus first
        let result = test_focus_prev_logic(&windows, Some(999));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_focus_cycling_edge_cases() {
        // Test with duplicate windows (should still work)
        let windows = vec![10, 10, 20];

        // Should find first occurrence and move correctly
        let result = test_focus_next_logic(&windows, Some(10));
        assert_eq!(result, Some(10)); // Next occurrence of 10

        // Test wrapping behavior
        let result = test_focus_next_logic(&windows, Some(20));
        assert_eq!(result, Some(10)); // Wrap to first
    }

    #[test]
    fn test_window_order_preservation() {
        // Test that window order is preserved correctly during swaps
        let mut windows = vec![1, 2, 3, 4, 5];

        // Test swap next logic (swap with next element)
        if let Some(pos) = windows.iter().position(|&w| w == 2) {
            let next_pos = (pos + 1) % windows.len();
            windows.swap(pos, next_pos);
        }
        assert_eq!(windows, vec![1, 3, 2, 4, 5]);

        // Swap back
        if let Some(pos) = windows.iter().position(|&w| w == 2) {
            let prev_pos = if pos == 0 { windows.len() - 1 } else { pos - 1 };
            windows.swap(pos, prev_pos);
        }
        assert_eq!(windows, vec![1, 2, 3, 4, 5]);
    }

    /// Helper to test fullscreen toggle logic
    fn test_toggle_fullscreen_logic(
        fullscreen_window: Option<Window>,
        focused_window: Option<Window>,
    ) -> Option<Window> {
        match (fullscreen_window, focused_window) {
            (None, Some(focused)) => {
                // Enter fullscreen mode
                Some(focused)
            }
            (Some(current_fs), Some(focused)) if current_fs == focused => {
                // Exit fullscreen mode (same window)
                None
            }
            (Some(_), Some(focused)) => {
                // Switch fullscreen to different window
                Some(focused)
            }
            (Some(_), None) => {
                // No focused window, can't toggle
                fullscreen_window
            }
            (None, None) => {
                // No focused window, can't enter fullscreen
                None
            }
        }
    }

    /// Helper to test auto-exit fullscreen when focusing different window
    fn test_focus_exit_fullscreen_logic(
        fullscreen_window: Option<Window>,
        target_window: Window,
    ) -> Option<Window> {
        if fullscreen_window.is_some() && fullscreen_window != Some(target_window) {
            // Exit fullscreen when focusing different window
            None
        } else {
            fullscreen_window
        }
    }

    #[test]
    fn test_toggle_fullscreen_enter_mode() {
        // Test entering fullscreen mode
        let result = test_toggle_fullscreen_logic(None, Some(10));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_toggle_fullscreen_exit_mode() {
        // Test exiting fullscreen mode (same window)
        let result = test_toggle_fullscreen_logic(Some(10), Some(10));
        assert_eq!(result, None);
    }

    #[test]
    fn test_toggle_fullscreen_switch_window() {
        // Test switching fullscreen to different window
        let result = test_toggle_fullscreen_logic(Some(10), Some(20));
        assert_eq!(result, Some(20));
    }

    #[test]
    fn test_toggle_fullscreen_no_focused_window() {
        // Test toggle with no focused window
        let result = test_toggle_fullscreen_logic(None, None);
        assert_eq!(result, None);

        // Test with fullscreen active but no focused window
        let result = test_toggle_fullscreen_logic(Some(10), None);
        assert_eq!(result, Some(10)); // Should remain in fullscreen
    }

    #[test]
    fn test_focus_auto_exit_fullscreen() {
        // Test auto-exit when focusing different window
        let result = test_focus_exit_fullscreen_logic(Some(10), 20);
        assert_eq!(result, None);

        // Test no auto-exit when focusing same window
        let result = test_focus_exit_fullscreen_logic(Some(10), 10);
        assert_eq!(result, Some(10));

        // Test no change when not in fullscreen
        let result = test_focus_exit_fullscreen_logic(None, 10);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fullscreen_state_consistency() {
        // Test multiple state transitions
        let mut fs_state = None;

        // Enter fullscreen
        fs_state = test_toggle_fullscreen_logic(fs_state, Some(10));
        assert_eq!(fs_state, Some(10));

        // Try to focus different window (should exit fullscreen)
        fs_state = test_focus_exit_fullscreen_logic(fs_state, 20);
        assert_eq!(fs_state, None);

        // Enter fullscreen for different window
        fs_state = test_toggle_fullscreen_logic(fs_state, Some(20));
        assert_eq!(fs_state, Some(20));

        // Switch fullscreen to third window
        fs_state = test_toggle_fullscreen_logic(fs_state, Some(30));
        assert_eq!(fs_state, Some(30));

        // Exit fullscreen
        fs_state = test_toggle_fullscreen_logic(fs_state, Some(30));
        assert_eq!(fs_state, None);
    }

    /// Helper to test window swap with fullscreen auto-exit
    fn test_swap_exit_fullscreen_logic(
        fullscreen_window: Option<Window>,
        will_swap: bool,
    ) -> Option<Window> {
        if will_swap && fullscreen_window.is_some() {
            // Exit fullscreen before performing swap
            None
        } else {
            fullscreen_window
        }
    }

    #[test]
    fn test_swap_operations_exit_fullscreen() {
        // Test that window swaps exit fullscreen mode
        let result = test_swap_exit_fullscreen_logic(Some(10), true);
        assert_eq!(result, None);

        // Test no change when swap doesn't occur
        let result = test_swap_exit_fullscreen_logic(Some(10), false);
        assert_eq!(result, Some(10));

        // Test no change when not in fullscreen
        let result = test_swap_exit_fullscreen_logic(None, true);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fullscreen_edge_cases() {
        // Test various edge cases for fullscreen functionality

        // Multiple consecutive toggles
        let mut fs_state = None;
        for _ in 0..3 {
            fs_state = test_toggle_fullscreen_logic(fs_state, Some(10));
            fs_state = test_toggle_fullscreen_logic(fs_state, Some(10));
        }
        assert_eq!(fs_state, None); // Should end up not in fullscreen

        // Rapid window switching
        fs_state = test_toggle_fullscreen_logic(None, Some(10));
        assert_eq!(fs_state, Some(10));

        for window in 20..25 {
            fs_state = test_toggle_fullscreen_logic(fs_state, Some(window));
            assert_eq!(fs_state, Some(window));
        }

        // Focus different window should exit
        fs_state = test_focus_exit_fullscreen_logic(fs_state, 100);
        assert_eq!(fs_state, None);
    }

    /// Helper to test destroy window logic
    fn test_destroy_window_logic(windows: &mut Vec<Window>, focused: Option<Window>) -> bool {
        if let Some(focused) = focused {
            if let Some(focused_idx) = windows.iter().position(|&w| w == focused) {
                windows.remove(focused_idx);
                return true;
            }
        }
        false
    }

    #[test]
    fn test_destroy_window_empty_list() {
        let mut windows = vec![];
        let result = test_destroy_window_logic(&mut windows, Some(10));
        assert!(!result); // No destruction should occur
        assert!(windows.is_empty());
    }

    #[test]
    fn test_destroy_window_no_focus() {
        let mut windows = vec![10, 20, 30];
        let result = test_destroy_window_logic(&mut windows, None);
        assert!(!result); // No destruction should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }

    #[test]
    fn test_destroy_window_focused_exists() {
        let mut windows = vec![10, 20, 30];

        // Destroy focused window (middle)
        let result = test_destroy_window_logic(&mut windows, Some(20));
        assert!(result); // Destruction should occur
        assert_eq!(windows, vec![10, 30]);

        // Destroy focused window (first)
        let result = test_destroy_window_logic(&mut windows, Some(10));
        assert!(result); // Destruction should occur
        assert_eq!(windows, vec![30]);

        // Destroy last window
        let result = test_destroy_window_logic(&mut windows, Some(30));
        assert!(result); // Destruction should occur
        assert!(windows.is_empty());
    }

    #[test]
    fn test_destroy_window_focused_not_exists() {
        let mut windows = vec![10, 20, 30];
        let result = test_destroy_window_logic(&mut windows, Some(999));
        assert!(!result); // No destruction should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }

    #[test]
    fn test_destroy_window_order_preservation() {
        // Test that remaining windows preserve order after destruction
        let mut windows = vec![1, 2, 3, 4, 5];

        // Destroy middle window
        test_destroy_window_logic(&mut windows, Some(3));
        assert_eq!(windows, vec![1, 2, 4, 5]);

        // Destroy first window
        test_destroy_window_logic(&mut windows, Some(1));
        assert_eq!(windows, vec![2, 4, 5]);

        // Destroy last window
        test_destroy_window_logic(&mut windows, Some(5));
        assert_eq!(windows, vec![2, 4]);
    }

    /// Direction for test swap operations
    #[derive(Debug, Clone, Copy)]
    enum TestSwapDirection {
        Next,
        Previous,
    }

    /// Helper to test window swapping logic in either direction
    fn test_swap_window_logic(
        windows: &mut [Window],
        focused: Option<Window>,
        direction: TestSwapDirection,
    ) -> bool {
        if windows.len() < 2 {
            return false;
        }

        if let Some(focused) = focused {
            if let Some(focused_idx) = windows.iter().position(|&w| w == focused) {
                let target_idx = match direction {
                    TestSwapDirection::Next => (focused_idx + 1) % windows.len(),
                    TestSwapDirection::Previous => {
                        if focused_idx == 0 {
                            windows.len() - 1
                        } else {
                            focused_idx - 1
                        }
                    }
                };
                windows.swap(focused_idx, target_idx);
                return true;
            }
        }
        false
    }

    /// Helper to test swap_window_next logic
    fn test_swap_window_next_logic(windows: &mut [Window], focused: Option<Window>) -> bool {
        test_swap_window_logic(windows, focused, TestSwapDirection::Next)
    }

    /// Helper to test swap_window_prev logic
    fn test_swap_window_prev_logic(windows: &mut [Window], focused: Option<Window>) -> bool {
        test_swap_window_logic(windows, focused, TestSwapDirection::Previous)
    }

    #[test]
    fn test_swap_window_next_empty_windows() {
        let mut windows = vec![];
        let result = test_swap_window_next_logic(&mut windows, None);
        assert!(!result); // No swap should occur
        assert!(windows.is_empty());
    }

    #[test]
    fn test_swap_window_next_single_window() {
        let mut windows = vec![10];
        let result = test_swap_window_next_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10]);
    }

    #[test]
    fn test_swap_window_next_multiple_windows() {
        // Test swapping first with second
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_next_logic(&mut windows, Some(10));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![20, 10, 30]);

        // Test swapping middle with next
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_next_logic(&mut windows, Some(20));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![10, 30, 20]);

        // Test swapping last with first (wrap around)
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_next_logic(&mut windows, Some(30));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![30, 20, 10]);

        // Test non-existent focused window
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_next_logic(&mut windows, Some(999));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }

    #[test]
    fn test_swap_window_prev_empty_windows() {
        let mut windows = vec![];
        let result = test_swap_window_prev_logic(&mut windows, None);
        assert!(!result); // No swap should occur
        assert!(windows.is_empty());
    }

    #[test]
    fn test_swap_window_prev_single_window() {
        let mut windows = vec![10];
        let result = test_swap_window_prev_logic(&mut windows, Some(10));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10]);
    }

    #[test]
    fn test_swap_window_prev_multiple_windows() {
        // Test swapping first with last (wrap around)
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_prev_logic(&mut windows, Some(10));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![30, 20, 10]);

        // Test swapping middle with previous
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_prev_logic(&mut windows, Some(20));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![20, 10, 30]);

        // Test swapping last with previous
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_prev_logic(&mut windows, Some(30));
        assert!(result); // Swap should occur
        assert_eq!(windows, vec![10, 30, 20]);

        // Test non-existent focused window
        let mut windows = vec![10, 20, 30];
        let result = test_swap_window_prev_logic(&mut windows, Some(999));
        assert!(!result); // No swap should occur
        assert_eq!(windows, vec![10, 20, 30]);
    }

    #[test]
    fn test_swap_window_order_preservation() {
        // Test that swapping preserves correct order relationships
        let mut windows = vec![1, 2, 3, 4, 5];

        // Swap middle window with next
        test_swap_window_next_logic(&mut windows, Some(3));
        assert_eq!(windows, vec![1, 2, 4, 3, 5]);

        // Swap back
        test_swap_window_prev_logic(&mut windows, Some(3));
        assert_eq!(windows, vec![1, 2, 3, 4, 5]);

        // Test wrapping behavior
        test_swap_window_next_logic(&mut windows, Some(5));
        assert_eq!(windows, vec![5, 2, 3, 4, 1]);
    }
}

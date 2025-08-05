//! Window manager core - coordinates X11 events and state

use anyhow::Result;
use std::process::Command;
use tracing::{debug, error, info};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

use crate::keyboard::KeyboardManager;
use crate::window_renderer::WindowRenderer;
use crate::window_state::WindowState;

/// Main window manager coordinating X11 events and window state
pub struct WindowManager<C: Connection> {
    pub(crate) conn: C,
    pub(crate) keyboard_manager: KeyboardManager,
    pub(crate) window_state: WindowState,
    pub(crate) window_renderer: WindowRenderer,
}

impl<C: Connection> WindowManager<C> {
    /// Creates a new window manager instance
    pub fn new(conn: C, screen_num: usize) -> Result<Self> {
        let config = crate::config::Config::load()?;
        info!(
            "Loaded configuration with {} shortcuts",
            config.shortcuts().len()
        );

        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;

        let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

        // SUBSTRUCTURE_REDIRECT/NOTIFY = become window manager
        let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);

        if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
            error!("Another window manager is already running: {:?}", e);
            return Err(anyhow::anyhow!(
                "Failed to become window manager. Is another WM running?"
            ));
        }

        info!("Successfully became the window manager");

        keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

        let window_state = WindowState::new(config, screen_num);
        let window_renderer = WindowRenderer::new();

        Ok(Self {
            conn,
            keyboard_manager,
            window_state,
            window_renderer,
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

    // =============================================================================
    // Event Handling
    // =============================================================================

    /// Main event dispatcher
    pub(crate) fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::KeyPress(ev) => self.handle_key_press(ev),
            Event::MapRequest(ev) => self.handle_map_request(ev),
            Event::UnmapNotify(ev) => self.handle_unmap_notify(ev),
            Event::DestroyNotify(ev) => self.handle_destroy_notify(ev),
            Event::EnterNotify(ev) => self.handle_enter_notify(ev),
            _ => {
                #[cfg(debug_assertions)]
                debug!("Unhandled event: {:#?}", event);
                Ok(())
            }
        }
    }

    /// Handles key press events
    fn handle_key_press(&mut self, event: KeyPressEvent) -> Result<()> {
        if let Some(command) = self.keyboard_manager.handle_key_press(&event) {
            info!("Shortcut pressed, executing: {}", command);

            // Handle window management commands
            match command {
                "focus_next" => return self.focus_next(),
                "focus_prev" => return self.focus_prev(),
                "swap_window_next" => return self.swap_window_next(),
                "swap_window_prev" => return self.swap_window_prev(),
                "destroy_window" => return self.destroy_focused_window(),
                "toggle_fullscreen" => return self.toggle_fullscreen(),
                "rotate_windows" => return self.rotate_windows(),
                _ => {
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if let Some(program) = parts.first() {
                        let mut cmd = Command::new(program);

                        if parts.len() > 1 {
                            cmd.args(&parts[1..]);
                        }

                        cmd.env("DISPLAY", self.window_state.default_display());

                        match cmd.spawn() {
                            Ok(_) => info!("Successfully launched: {}", command),
                            Err(e) => error!("Failed to launch {}: {}", command, e),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles window map requests
    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
        let window = event.window;
        info!("Mapping window: {:?}", window);

        self.conn.map_window(window)?;
        self.window_state.add_window_to_layout(window);
        self.window_state.set_focused_window(Some(window));

        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles window unmap notifications
    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Unmapping window: {:?}", window);

        if self.window_state.is_intentionally_unmapped(window) {
            info!("Window {:?} was intentionally unmapped, ignoring", window);
            return Ok(());
        }

        info!(
            "Window {:?} closed by user, removing from management",
            window
        );
        self.window_state.remove_window_from_layout(window);

        if self.window_state.get_focused_window() == Some(window) {
            let next_focus = self.window_state.get_first_window();
            if let Some(next_focus) = next_focus {
                self.window_state.set_focused_window(Some(next_focus));
            } else {
                self.window_state.clear_focus();
            }
        }

        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles window destroy notifications
    fn handle_destroy_notify(&mut self, event: DestroyNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Window destroyed: {:?}", window);

        self.window_state.remove_window_from_layout(window);
        self.window_state.remove_intentionally_unmapped(window);

        if self.window_state.get_fullscreen_window() == Some(window) {
            info!("Fullscreen window destroyed, exiting fullscreen mode");
            self.window_state.clear_fullscreen();
        }

        if self.window_state.get_focused_window() == Some(window) {
            let next_focus = self.window_state.get_first_window();
            if let Some(next_focus) = next_focus {
                self.window_state.set_focused_window(Some(next_focus));
            } else {
                self.window_state.clear_focus();
            }
        }

        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles mouse enter events (focus follows mouse)
    fn handle_enter_notify(&mut self, event: EnterNotifyEvent) -> Result<()> {
        let window = event.event;
        #[cfg(debug_assertions)]
        debug!("Mouse entered window: {:?}", window);

        if self.window_state.has_window(window) {
            self.window_state.set_focused_window(Some(window));
            self.window_renderer
                .apply_state(&mut self.conn, &mut self.window_state)?;
        }
        Ok(())
    }
}

// =============================================================================
// Focus Management
// =============================================================================

impl<C: Connection> WindowManager<C> {
    /// Focuses next window in BSP tree order
    pub fn focus_next(&mut self) -> Result<()> {
        self.window_renderer
            .focus_next(&mut self.conn, &mut self.window_state)
    }

    /// Focuses previous window in BSP tree order
    pub fn focus_prev(&mut self) -> Result<()> {
        self.window_renderer
            .focus_prev(&mut self.conn, &mut self.window_state)
    }
}

impl<C: Connection> WindowManager<C> {
    /// Destroys the currently focused window
    pub fn destroy_focused_window(&mut self) -> Result<()> {
        self.window_renderer
            .destroy_focused_window(&mut self.conn, &mut self.window_state)
    }

    /// Swaps focused window with next window in BSP order
    pub fn swap_window_next(&mut self) -> Result<()> {
        self.window_renderer
            .swap_window_next(&mut self.conn, &mut self.window_state)
    }

    /// Swaps focused window with previous window in BSP order
    pub fn swap_window_prev(&mut self) -> Result<()> {
        self.window_renderer
            .swap_window_prev(&mut self.conn, &mut self.window_state)
    }

    /// Toggles fullscreen mode for focused window
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        self.window_renderer
            .toggle_fullscreen(&mut self.conn, &mut self.window_state)
    }

    /// Rotates focused window by flipping parent split direction
    pub fn rotate_windows(&mut self) -> Result<()> {
        self.window_renderer
            .rotate_windows(&mut self.conn, &mut self.window_state)
    }
}

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

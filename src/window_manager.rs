//! # Window Manager Module
//!
//! This is the heart of rustile - it coordinates all window management operations.
//! ## The Three Main Responsibilities
//!
//! 1. **Event Handling**: React to X11 events (keyboard, new windows, etc.)
//! 2. **State Coordination**: Keep WindowState and WindowRenderer in sync
//! 3. **User Interface**: Provide public methods for window operations

use anyhow::Result;
use std::process::Command;
use tracing::{debug, error, info};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

use crate::keyboard::KeyboardManager;
use crate::window_renderer::WindowRenderer;
use crate::window_state::WindowState;

/// The main window manager structure that coordinates everything
pub struct WindowManager<C: Connection> {
    /// The connection to the X11 server
    pub(crate) conn: C,

    /// Manages keyboard shortcuts (Alt+j, Alt+k, etc.)
    pub(crate) keyboard_manager: KeyboardManager,

    /// Stores the current state of all windows
    pub(crate) window_state: WindowState,

    /// Handles all X11 drawing operations
    pub(crate) window_renderer: WindowRenderer,
}

impl<C: Connection> WindowManager<C> {
    /// Creates a new window manager instance
    pub fn new(conn: C, screen_num: usize) -> Result<Self> {
        // Load configuration from disk
        let config = crate::config::Config::load()?;
        info!(
            "Loaded configuration with {} shortcuts",
            config.shortcuts().len()
        );

        // Get X11 setup information
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root; // The "desktop" window

        // Initialize keyboard manager
        let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

        // Register as window manager
        // SUBSTRUCTURE_REDIRECT = "Tell me when windows want to appear"
        // SUBSTRUCTURE_NOTIFY = "Tell me when windows change"
        let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);

        // Try to become the window manager
        if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
            error!("Another window manager is already running: {:?}", e);
            return Err(anyhow::anyhow!(
                "Failed to become window manager. Is another WM running?"
            ));
        }

        info!("Successfully became the window manager");

        // Register keyboard shortcuts from config
        keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

        // Create our subsystems
        let window_state = WindowState::new(config, screen_num);
        let window_renderer = WindowRenderer::new();

        // Return the complete WindowManager
        Ok(Self {
            conn,
            keyboard_manager,
            window_state,
            window_renderer,
        })
    }

    /// Runs the main event loop - THE HEART OF THE WINDOW MANAGER!
    pub fn run(mut self) -> Result<()> {
        info!("Starting window manager event loop");

        // THE INFINITE LOOP - This runs until rustile is killed
        loop {
            // 1. Send any pending X11 commands to the server
            self.conn.flush()?;

            // 2. Wait for the next event (BLOCKS HERE!)
            // The program stops here until something happens
            let event = self.conn.wait_for_event()?;

            // 3. Handle the event
            if let Err(e) = self.handle_event(event) {
                error!("Error handling event: {:?}", e);
                // Important: We continue running!
                // One bad window shouldn't crash the whole WM
            }

            // 4. Loop back to step 1
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
            Event::ConfigureRequest(ev) => self.handle_configure_request(ev),
            Event::DestroyNotify(ev) => self.handle_destroy_notify(ev),
            Event::FocusIn(ev) => self.handle_focus_in(ev),
            Event::FocusOut(ev) => self.handle_focus_out(ev),
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
                    // Handle regular application commands
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if let Some(program) = parts.first() {
                        let mut cmd = Command::new(program);

                        // Add arguments if any
                        if parts.len() > 1 {
                            cmd.args(&parts[1..]);
                        }

                        // Set display environment
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

        // Map the window
        self.conn.map_window(window)?;

        // Add to managed windows and set focus
        self.window_state.add_window_to_layout(window);
        self.window_state.set_focused_window(Some(window));

        // Apply complete state to screen
        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles window unmap notifications
    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Unmapping window: {:?}", window);

        // Check if this was intentionally unmapped (during fullscreen)
        if self.window_state.is_intentionally_unmapped(window) {
            info!("Window {:?} was intentionally unmapped, ignoring", window);
            return Ok(());
        }

        // Only remove from managed windows if NOT intentionally unmapped
        info!(
            "Window {:?} closed by user, removing from management",
            window
        );
        self.window_state.remove_window_from_layout(window);

        // Update focus if focused window was unmapped
        if self.window_state.get_focused_window() == Some(window) {
            // Focus first remaining window in BSP tree order
            let next_focus = self.window_state.get_first_window();
            if let Some(next_focus) = next_focus {
                self.window_state.set_focused_window(Some(next_focus));
            } else {
                self.window_state.clear_focus();
            }
        }

        // Apply complete state to screen
        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles window configure requests
    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!(
            "Configure request for window: {:?} - Event: {:#?}",
            event.window, event
        );

        // For now, just honor the request
        // In the future, we might want to be more selective
        let values = ConfigureWindowAux::from_configure_request(&event);
        self.conn.configure_window(event.window, &values)?;

        Ok(())
    }

    /// Handles window destroy notifications
    fn handle_destroy_notify(&mut self, event: DestroyNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Window destroyed: {:?}", window);

        // Remove from managed windows
        self.window_state.remove_window_from_layout(window);

        // Clean up intentionally unmapped set to prevent memory leaks
        self.window_state.remove_intentionally_unmapped(window);

        // Clear fullscreen if fullscreen window was destroyed
        if self.window_state.get_fullscreen_window() == Some(window) {
            info!("Fullscreen window destroyed, exiting fullscreen mode");
            self.window_state.clear_fullscreen();
        }

        // Update focus if focused window was destroyed
        if self.window_state.get_focused_window() == Some(window) {
            // Focus first remaining window in BSP tree order
            let next_focus = self.window_state.get_first_window();
            if let Some(next_focus) = next_focus {
                self.window_state.set_focused_window(Some(next_focus));
            } else {
                self.window_state.clear_focus();
            }
        }

        // Apply complete state to screen
        self.window_renderer
            .apply_state(&mut self.conn, &mut self.window_state)?;

        Ok(())
    }

    /// Handles focus in events (window receives keyboard focus)
    ///
    /// ## When This Happens
    ///
    /// X11 tells us when a window becomes the "active" window:
    /// - User clicks on a window
    /// - We programmatically focus a window
    /// - Window manager changes focus due to window closure
    ///
    /// ## Why We Don't Act Here
    ///
    /// We typically don't need to do anything because:
    /// - We already set borders when WE change focus
    /// - These events are mostly for X11's internal bookkeeping
    /// - Our focus management happens in the renderer
    ///
    /// But we log it in debug mode to help with development.
    fn handle_focus_in(&mut self, _event: FocusInEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!(
            "Focus in event for window: {:?} - Event: {:#?}",
            _event.event, _event
        );
        Ok(())
    }

    /// Handles focus out events (window loses keyboard focus)
    ///
    /// ## When This Happens
    ///
    /// X11 tells us when a window stops being the "active" window:
    /// - User clicks on a different window
    /// - We programmatically focus a different window
    /// - Window gets destroyed or minimized
    ///
    /// ## Why We Don't Act Here
    ///
    /// Same as FocusIn - we handle focus changes in our own code,
    /// these events are mostly X11's way of keeping us informed.
    ///
    /// The real focus management happens in `window_renderer.rs`
    /// where we set border colors and update our internal state.
    fn handle_focus_out(&mut self, _event: FocusOutEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!(
            "Focus out event for window: {:?} - Event: {:#?}",
            _event.event, _event
        );
        Ok(())
    }

    /// Handles mouse pointer entering a window ("focus follows mouse")
    ///
    /// ## When This Happens
    ///
    /// Every time your mouse cursor moves into a window:
    /// - Moving mouse from desktop to a window
    /// - Moving mouse between different windows
    /// - Moving mouse back from another application
    ///
    /// ## Behavior: Focus Follows Mouse
    ///
    /// This implements "sloppy focus" - windows get focus when you
    /// move your mouse over them (no clicking required):
    ///
    /// ```text
    /// Mouse moves into xterm
    ///         │
    ///         ▼
    /// Is xterm managed by us? ─── No ──► Ignore
    ///         │
    ///        Yes
    ///         │
    ///         ▼
    /// Focus xterm (red border)
    /// Unfocus previous window (gray border)
    /// ```
    ///
    /// ## Safety Check
    ///
    /// We only focus windows that we manage. This prevents focusing
    /// system windows, panels, or other WM components.
    fn handle_enter_notify(&mut self, event: EnterNotifyEvent) -> Result<()> {
        let window = event.event;
        #[cfg(debug_assertions)]
        debug!("Mouse entered window: {:?}", window);

        // Only focus if it's a managed window
        if self.window_state.has_window(window) {
            self.window_state.set_focused_window(Some(window));
            self.window_renderer
                .apply_state(&mut self.conn, &mut self.window_state)?;
        }
        Ok(())
    }
}

// =============================================================================
// Focus Management - Making Windows "Active"
// =============================================================================
//
// Focus determines which window receives keyboard input.
// Only one window can be focused at a time.

impl<C: Connection> WindowManager<C> {
    /// Focuses the next window in the BSP tree order (Alt+j)
    ///
    /// ## How It Works
    ///
    /// 1. Get all windows from BSP tree in left-to-right order
    /// 2. Find current focused window in the list
    /// 3. Move to next window (wraps around to first if at end)
    /// 4. Set visual focus (red border) and keyboard focus
    ///
    /// ## Example
    /// ```text
    /// Before: [xterm] firefox  [gedit]  (firefox focused)
    /// After:  [xterm] firefox   gedit   (gedit focused)
    /// ```
    pub fn focus_next(&mut self) -> Result<()> {
        self.window_renderer
            .focus_next(&mut self.conn, &mut self.window_state)
    }

    /// Focuses the previous window in the BSP tree order (Alt+k)
    ///
    /// ## How It Works
    ///
    /// Same as `focus_next()` but moves backwards through the window list.
    /// If currently on first window, wraps around to last window.
    ///
    /// ## Example
    /// ```text
    /// Before: [xterm] firefox  [gedit]  (gedit focused)
    /// After:  [xterm] firefox   gedit   (firefox focused)
    /// ```
    pub fn focus_prev(&mut self) -> Result<()> {
        self.window_renderer
            .focus_prev(&mut self.conn, &mut self.window_state)
    }
}

// =============================================================================
// Window Operations and Layout Integration - Moving and Manipulating Windows
// =============================================================================
//
// These functions modify window positions, sizes, and states.
// They all delegate to the WindowRenderer for the actual X11 operations.

impl<C: Connection> WindowManager<C> {
    /// Destroys (closes) the currently focused window (Shift+Alt+q)
    ///
    /// ## What This Does
    ///
    /// Forcefully closes the focused window (like clicking the X button):
    /// 1. Sends X11 "destroy" command to the window
    /// 2. Window receives signal and should exit gracefully
    /// 3. We'll get a `DestroyNotify` event confirming closure
    /// 4. Cleanup happens in `handle_destroy_notify()`
    ///
    /// ## Warning
    ///
    /// This is a "polite" close request. Some applications might
    /// ask "Do you want to save?" or ignore the request entirely.
    pub fn destroy_focused_window(&mut self) -> Result<()> {
        self.window_renderer
            .destroy_focused_window(&mut self.conn, &mut self.window_state)
    }

    /// Swaps the currently focused window with the next window (Shift+Alt+j)
    ///
    /// ## How Window Swapping Works
    ///
    /// Changes window positions in the BSP tree without changing focus:
    ///
    /// ```text
    /// Before: [A] B   C   (A is focused)
    /// After:   B [A]  C   (A still focused, but moved)
    /// ```
    ///
    /// The BSP tree structure remains the same, but window assignments
    /// to tree nodes change. This creates a "shuffle" effect.
    pub fn swap_window_next(&mut self) -> Result<()> {
        self.window_renderer
            .swap_window_next(&mut self.conn, &mut self.window_state)
    }

    /// Swaps the currently focused window with the previous window (Shift+Alt+k)
    ///
    /// ## Opposite of swap_window_next()
    ///
    /// ```text
    /// Before:  A  [B] C   (B is focused)
    /// After:  [B]  A  C   (B still focused, but moved)
    /// ```
    pub fn swap_window_prev(&mut self) -> Result<()> {
        self.window_renderer
            .swap_window_prev(&mut self.conn, &mut self.window_state)
    }

    /// Toggles fullscreen mode for the focused window (Alt+f)
    ///
    /// ## Fullscreen Behavior
    ///
    /// - **Enter fullscreen**: Hide all other windows, expand current to screen
    /// - **Exit fullscreen**: Restore all windows to tiled layout
    ///
    /// ```text
    /// Normal Mode:         Fullscreen Mode:
    /// ┌──────┬──────┐      ┌─────────────────────────┐
    /// │ [A]  │  B   │  →   │                         │
    /// ├──────┼──────┤      │          [A]            │
    /// │  C   │  D   │      │                         │
    /// └──────┴──────┘      └─────────────────────────┘
    /// ```
    ///
    /// Windows B, C, D are temporarily hidden (unmapped).
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        self.window_renderer
            .toggle_fullscreen(&mut self.conn, &mut self.window_state)
    }

    /// Rotates the focused window by flipping its parent split direction (Alt+r)
    ///
    /// ## BSP Rotation Explained
    ///
    /// Changes how windows are split without changing their relative positions:
    ///
    /// ```text
    /// Horizontal Split:     Vertical Split:
    /// ┌──────┬──────┐      ┌──────────────────┐
    /// │ [A]  │  B   │  →   │       [A]       │
    /// └──────┴──────┘      ├──────────────────┤
    ///                      │        B        │
    ///                      └──────────────────┘
    /// ```
    ///
    /// This is useful for getting better proportions for different content.
    pub fn rotate_windows(&mut self) -> Result<()> {
        self.window_renderer
            .rotate_windows(&mut self.conn, &mut self.window_state)
    }
}

// =============================================================================
// Tests - Ensuring Our Logic Works Correctly
// =============================================================================
//
// These tests verify our window management algorithms work correctly
// without needing a real X11 server. They test pure logic functions.
//
// ## For Beginners: Testing in Rust
//
// - `#[cfg(test)]` = "only compile this when running tests"
// - `#[test]` = "this function is a test case"
// - `assert_eq!(a, b)` = "panic if a != b"
// - `cargo test` runs all tests
//
// ## Why Test Without X11?
//
// Testing with a real X server is slow and complex. Instead, we test
// the core algorithms with simple data structures.

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

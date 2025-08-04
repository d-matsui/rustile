//! Window rendering and X11 operations
//!
//! This module handles all X11 visual operations including positioning, borders,
//! mapping/unmapping, and layout application. It works with WindowState to render
//! the current state to the screen.

use anyhow::Result;
use tracing::{debug, info};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::window_state::WindowState;

/// Handles all X11 rendering operations
pub struct WindowRenderer {
    // No fields - pure operations using injected dependencies
}

impl WindowRenderer {
    /// Creates a new window renderer
    pub fn new() -> Self {
        Self {}
    }

    /// Sets X11 input focus to a specific window
    pub fn set_x11_focus<C: Connection>(&self, conn: &C, window: Window) -> Result<()> {
        conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;
        Ok(())
    }

    /// Updates window borders based on current focus state
    pub fn update_focus_borders<C: Connection>(&self, conn: &C, state: &WindowState) -> Result<()> {
        for &window in &state.get_all_windows() {
            let border_color = state.border_color_for_window(window);
            self.configure_window_border(conn, window, border_color, state.border_width())?;
        }
        Ok(())
    }

    /// Configures window border color and width
    pub fn configure_window_border<C: Connection>(
        &self,
        conn: &C,
        window: Window,
        border_color: u32,
        border_width: u32,
    ) -> Result<()> {
        let border_aux = ChangeWindowAttributesAux::new().border_pixel(border_color);
        conn.change_window_attributes(window, &border_aux)?;

        let config_aux = ConfigureWindowAux::new().border_width(border_width);
        conn.configure_window(window, &config_aux)?;

        Ok(())
    }

    /// Sets focus to a specific window with full rendering
    pub fn set_focus<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
        window: Window,
    ) -> Result<()> {
        if !state.has_window(window) {
            return Ok(());
        }

        // Set X11 input focus
        self.set_x11_focus(conn, window)?;

        // Update state
        state.set_focused_window(Some(window));

        // Update window borders
        self.update_focus_borders(conn, state)?;

        #[cfg(debug_assertions)]
        debug!("Focus set to window: {:?}", window);
        Ok(())
    }

    /// Focuses the next window in the stack
    pub fn focus_next<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        let next_window = if let Some(current) = state.get_focused_window() {
            // Use BSP tree navigation
            state.next_window(current).unwrap_or(current)
        } else {
            // Focus first window if none focused
            match state.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        // Exit fullscreen if trying to focus a different window
        if state.is_in_fullscreen_mode() && state.get_fullscreen_window() != Some(next_window) {
            info!("Exiting fullscreen mode to focus different window");
            state.clear_fullscreen();
            self.apply_layout(conn, state)?;
        }

        self.set_focus(conn, state, next_window)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses the previous window in the stack
    pub fn focus_prev<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        let prev_window = if let Some(current) = state.get_focused_window() {
            // Use BSP tree navigation
            state.prev_window(current).unwrap_or(current)
        } else {
            // Focus first window if none focused
            match state.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        // Exit fullscreen if trying to focus a different window
        if state.is_in_fullscreen_mode() && state.get_fullscreen_window() != Some(prev_window) {
            info!("Exiting fullscreen mode to focus different window");
            state.clear_fullscreen();
            self.apply_layout(conn, state)?;
        }

        self.set_focus(conn, state, prev_window)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }

    /// Applies the current BSP tree layout without rebuilding the tree
    pub fn apply_layout<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        // If we're in fullscreen mode, apply fullscreen layout instead
        if state.is_in_fullscreen_mode() {
            return self.apply_fullscreen_layout(conn, state);
        }

        let setup = conn.setup();
        let screen = &setup.roots[state.screen_num()];

        // Ensure all windows are mapped (visible) and have borders when not in fullscreen
        let border_width = state.border_width();
        for &window in &state.get_all_windows() {
            conn.map_window(window)?;
            // Remove from intentionally unmapped set when restoring
            state.remove_intentionally_unmapped(window);
            // Restore border width
            conn.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;
        }

        // Calculate window geometries from existing BSP tree (preserves tree structure)
        let geometries =
            state.calculate_window_geometries(screen.width_in_pixels, screen.height_in_pixels);

        // Apply calculated geometries and update borders
        for geometry in &geometries {
            let border_color = state.border_color_for_window(geometry.window);

            // Set border color
            conn.change_window_attributes(
                geometry.window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;

            // Set geometry and border width
            conn.configure_window(
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
        if let Some(focused) = state.get_focused_window() {
            conn.configure_window(
                focused,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        conn.flush()?;

        #[cfg(debug_assertions)]
        debug!(
            "Applied existing BSP tree layout to {} windows",
            geometries.len()
        );

        Ok(())
    }

    /// Applies fullscreen layout - window takes entire screen
    fn apply_fullscreen_layout<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if let Some(fullscreen) = state.get_fullscreen_window() {
            let setup = conn.setup();
            let screen = &setup.roots[state.screen_num()];

            // Ensure the fullscreen window is mapped (visible)
            conn.map_window(fullscreen)?;

            // Configure fullscreen window to cover entire screen (no gaps, no borders)
            let config = ConfigureWindowAux::new()
                .x(0)
                .y(0)
                .width(u32::from(screen.width_in_pixels))
                .height(u32::from(screen.height_in_pixels))
                .border_width(0);

            conn.configure_window(fullscreen, &config)?;

            // Hide all other windows (mark as intentionally unmapped)
            for &window in &state.get_all_windows() {
                if window != fullscreen {
                    // Mark as intentionally unmapped BEFORE unmapping
                    state.mark_intentionally_unmapped(window);
                    conn.unmap_window(window)?;
                }
            }

            // Ensure fullscreen window is on top
            conn.configure_window(
                fullscreen,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;

            conn.flush()?;
        }

        Ok(())
    }

    /// Destroys (closes) the currently focused window
    pub fn destroy_focused_window<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if let Some(focused) = state.get_focused_window() {
            info!("Destroying focused window: {:?}", focused);

            // Try to close the window gracefully first using WM_DELETE_WINDOW
            // If that fails, kill it forcefully
            self.close_window_gracefully(conn, focused)
                .or_else(|_| self.kill_window_forcefully(conn, focused))?;
        } else {
            info!("No focused window to destroy");
        }
        Ok(())
    }

    /// Attempts to close a window gracefully using WM_DELETE_WINDOW protocol
    fn close_window_gracefully<C: Connection>(&self, conn: &C, window: Window) -> Result<()> {
        // Get WM_DELETE_WINDOW and WM_PROTOCOLS atoms
        let wm_protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;
        let wm_delete_window = conn.intern_atom(false, b"WM_DELETE_WINDOW")?.reply()?.atom;

        // Check if the window supports WM_DELETE_WINDOW
        let protocols = conn
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
                    data: ClientMessageData::from([wm_delete_window, CURRENT_TIME, 0, 0, 0]),
                };

                conn.send_event(false, window, EventMask::NO_EVENT, event)?;
                conn.flush()?;
                info!("Sent WM_DELETE_WINDOW message to window {:?}", window);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "Window does not support WM_DELETE_WINDOW protocol"
        ))
    }

    /// Forcefully kills a window using XKillClient
    fn kill_window_forcefully<C: Connection>(&self, conn: &C, window: Window) -> Result<()> {
        info!("Forcefully killing window {:?}", window);
        conn.kill_client(window)?;
        conn.flush()?;
        Ok(())
    }

    /// Swaps the currently focused window with the next window in the layout
    pub fn swap_window_next<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        self.swap_window_direction(conn, state, SwapDirection::Next)
    }

    /// Swaps the currently focused window with the previous window in the layout
    pub fn swap_window_prev<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        self.swap_window_direction(conn, state, SwapDirection::Previous)
    }

    /// Helper method to swap windows in a given direction
    fn swap_window_direction<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
        direction: SwapDirection,
    ) -> Result<()> {
        if state.window_count() < 2 {
            return Ok(());
        }

        // Exit fullscreen if active, then perform swap
        if state.is_in_fullscreen_mode() {
            info!("Exiting fullscreen for window swap");
            state.clear_fullscreen();
        }

        if let Some(focused) = state.get_focused_window() {
            let target_window = match direction {
                SwapDirection::Next => state.next_window(focused),
                SwapDirection::Previous => state.prev_window(focused),
            };

            if let Some(target_window) = target_window {
                // Swap windows in the BSP tree
                state.swap_windows(focused, target_window);

                let direction_str = match direction {
                    SwapDirection::Next => "next",
                    SwapDirection::Previous => "previous",
                };

                info!(
                    "Swapped window {:?} with {} window {:?}",
                    focused, direction_str, target_window
                );

                // Apply layout
                self.apply_layout(conn, state)?;
            }
        }
        Ok(())
    }

    /// Toggles fullscreen mode for the focused window
    pub fn toggle_fullscreen<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        let focused = match state.get_focused_window() {
            Some(window) => window,
            None => {
                info!("No window focused for fullscreen toggle");
                return Ok(());
            }
        };

        // Check if we're currently in fullscreen mode
        if let Some(fullscreen) = state.get_fullscreen_window() {
            if fullscreen == focused {
                // Exit fullscreen mode
                info!("Exiting fullscreen mode for window {:?}", focused);
                state.clear_fullscreen();
                self.apply_layout(conn, state)?;
            } else {
                // Different window wants fullscreen, switch to it
                info!(
                    "Switching fullscreen from {:?} to {:?}",
                    fullscreen, focused
                );
                state.set_fullscreen_window(Some(focused));
                self.apply_fullscreen_layout(conn, state)?;
            }
        } else {
            // Enter fullscreen mode
            info!("Entering fullscreen mode for window {:?}", focused);
            state.set_fullscreen_window(Some(focused));
            self.apply_fullscreen_layout(conn, state)?;
        }

        Ok(())
    }

    /// Rotates the focused window by flipping its parent split direction
    pub fn rotate_windows<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        let focused = match state.get_focused_window() {
            Some(window) => window,
            None => {
                info!("No window focused for rotation");
                return Ok(());
            }
        };

        // Cannot rotate in fullscreen mode
        if state.is_in_fullscreen_mode() {
            info!("Cannot rotate in fullscreen mode");
            return Ok(());
        }

        // Need at least 2 windows to rotate
        if state.window_count() < 2 {
            info!("Not enough windows to rotate (need at least 2)");
            return Ok(());
        }

        info!(
            "Rotating parent split direction for focused window {:?}",
            focused
        );

        // Rotate the focused window in the BSP tree
        let rotated = state.rotate_window(focused);

        if rotated {
            // Apply the existing rotated tree layout (without rebuilding)
            self.apply_layout(conn, state)?;
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

/// Direction for window swapping operations
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,
    Previous,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_renderer_creation() {
        let renderer = WindowRenderer::new();
        // WindowRenderer has no fields, so just test it can be created
        let _ = renderer;
    }

    #[test]
    fn test_swap_direction_enum() {
        let next = SwapDirection::Next;
        let prev = SwapDirection::Previous;

        // Test that enum values can be created and compared
        assert!(matches!(next, SwapDirection::Next));
        assert!(matches!(prev, SwapDirection::Previous));
    }

    // Note: Most WindowRenderer methods require X11 connection and are tested
    // through integration tests rather than unit tests
}

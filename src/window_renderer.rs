//! Window rendering and X11 operations

use anyhow::Result;
#[cfg(debug_assertions)]
use tracing::debug;
use tracing::info;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::window_state::WindowState;

/// Handles X11 rendering operations
pub struct WindowRenderer {}

impl WindowRenderer {
    /// Creates a new window renderer
    pub fn new() -> Self {
        Self {}
    }

    /// Focuses next window in BSP order
    pub fn focus_next<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        let next_window = if let Some(current) = state.get_focused_window() {
            state.next_window(current).unwrap_or(current)
        } else {
            match state.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        if state.is_in_fullscreen_mode() && state.get_fullscreen_window() != Some(next_window) {
            info!("Exiting fullscreen mode to focus different window");
            state.clear_fullscreen();
        }

        state.set_focused_window(Some(next_window));
        self.apply_state(conn, state)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses previous window in BSP order
    pub fn focus_prev<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        let prev_window = if let Some(current) = state.get_focused_window() {
            state.prev_window(current).unwrap_or(current)
        } else {
            match state.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        if state.is_in_fullscreen_mode() && state.get_fullscreen_window() != Some(prev_window) {
            info!("Exiting fullscreen mode to focus different window");
            state.clear_fullscreen();
        }

        state.set_focused_window(Some(prev_window));
        self.apply_state(conn, state)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }

    /// Applies current window state to screen (unified rendering method)
    pub fn apply_state<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if state.window_count() == 0 {
            return Ok(());
        }

        if state.is_in_fullscreen_mode() {
            self.apply_fullscreen_layout(conn, state)?;
        } else {
            self.apply_normal_layout(conn, state)?;
        }

        if let Some(focused) = state.get_focused_window() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)?;
            conn.configure_window(
                focused,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        conn.flush()?;

        #[cfg(debug_assertions)]
        debug!(
            "Applied complete state to screen: {} windows",
            state.window_count()
        );

        Ok(())
    }

    /// Applies normal tiled layout
    fn apply_normal_layout<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        let setup = conn.setup();
        let screen = &setup.roots[state.screen_num()];

        let border_width = state.border_width();
        for &window in &state.get_all_windows() {
            conn.map_window(window)?;
            state.remove_intentionally_unmapped(window);
            conn.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;
        }

        let mut geometries =
            state.calculate_window_geometries(screen.width_in_pixels, screen.height_in_pixels);

        // Apply zoom if a window is zoomed
        if let Some(zoomed_window) = state.get_zoomed_window() {
            let screen_rect = crate::bsp::BspRect {
                x: state.layout_params().gap as i32,
                y: state.layout_params().gap as i32,
                width: (screen.width_in_pixels as i32 - 2 * state.layout_params().gap as i32)
                    .max(state.layout_params().min_window_width as i32),
                height: (screen.height_in_pixels as i32 - 2 * state.layout_params().gap as i32)
                    .max(state.layout_params().min_window_height as i32),
            };

            // Find parent bounds for the zoomed window
            if let Some(parent_bounds) = state
                .bsp_tree()
                .find_parent_bounds(zoomed_window, screen_rect)
            {
                // Override the zoomed window's geometry with parent bounds
                for geometry in &mut geometries {
                    if geometry.window == zoomed_window {
                        geometry.x = parent_bounds.x;
                        geometry.y = parent_bounds.y;
                        geometry.width = parent_bounds
                            .width
                            .max(state.layout_params().min_window_width as i32)
                            as u32;
                        geometry.height = parent_bounds
                            .height
                            .max(state.layout_params().min_window_height as i32)
                            as u32;
                        break;
                    }
                }
            }
        }

        for geometry in &geometries {
            let border_color = state.border_color_for_window(geometry.window);

            conn.change_window_attributes(
                geometry.window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;

            let mut config = ConfigureWindowAux::new()
                .x(geometry.x)
                .y(geometry.y)
                .width(geometry.width)
                .height(geometry.height)
                .border_width(border_width);

            // If this is the zoomed window, ensure it's on top
            if Some(geometry.window) == state.get_zoomed_window() {
                config = config.stack_mode(StackMode::ABOVE);
            }

            conn.configure_window(geometry.window, &config)?;
        }

        Ok(())
    }

    /// Applies fullscreen layout
    fn apply_fullscreen_layout<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if let Some(fullscreen) = state.get_fullscreen_window() {
            let setup = conn.setup();
            let screen = &setup.roots[state.screen_num()];

            conn.map_window(fullscreen)?;

            let config = ConfigureWindowAux::new()
                .x(0)
                .y(0)
                .width(u32::from(screen.width_in_pixels))
                .height(u32::from(screen.height_in_pixels))
                .border_width(0);

            conn.configure_window(fullscreen, &config)?;

            for &window in &state.get_all_windows() {
                if window != fullscreen {
                    state.mark_intentionally_unmapped(window);
                    conn.unmap_window(window)?;
                }
            }

            conn.configure_window(
                fullscreen,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;

            conn.flush()?;
        }

        Ok(())
    }

    /// Destroys the currently focused window
    pub fn destroy_focused_window<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        if let Some(focused) = state.get_focused_window() {
            info!("Destroying focused window: {:?}", focused);
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

    /// Toggles zoom for the focused window
    pub fn toggle_zoom<C: Connection>(
        &mut self,
        conn: &mut C,
        state: &mut WindowState,
    ) -> Result<()> {
        // Don't allow zoom in fullscreen mode
        if state.is_in_fullscreen_mode() {
            info!("Cannot zoom while in fullscreen mode");
            return Ok(());
        }

        // Get the focused window
        let focused = match state.get_focused_window() {
            Some(window) => window,
            None => {
                info!("No focused window to zoom");
                return Ok(());
            }
        };

        // Get screen dimensions for parent bounds calculation
        let setup = conn.setup();
        let screen = &setup.roots[state.screen_num()];
        let screen_rect = crate::bsp::BspRect {
            x: state.layout_params().gap as i32,
            y: state.layout_params().gap as i32,
            width: (screen.width_in_pixels as i32 - 2 * state.layout_params().gap as i32)
                .max(state.layout_params().min_window_width as i32),
            height: (screen.height_in_pixels as i32 - 2 * state.layout_params().gap as i32)
                .max(state.layout_params().min_window_height as i32),
        };

        // Toggle zoom state
        if state.get_zoomed_window() == Some(focused) {
            // Already zoomed - unzoom
            state.clear_zoom();
            info!("Unzoomed window: {:?}", focused);
        } else {
            // Check if this window can be zoomed (has a parent)
            if state
                .bsp_tree()
                .find_parent_bounds(focused, screen_rect)
                .is_none()
            {
                // Single window or root - cannot zoom
                info!("Window {:?} has no parent to zoom to", focused);
                return Ok(());
            }

            // Zoom the focused window
            state.set_zoomed_window(Some(focused));
            info!("Zoomed window: {:?}", focused);
        }

        // Apply the new state
        self.apply_state(conn, state)?;
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

                // Apply complete state to screen
                self.apply_state(conn, state)?;
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
                self.apply_state(conn, state)?;
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
            // Apply complete state to screen
            self.apply_state(conn, state)?;
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

//! Window state management and operations
//!
//! This module manages all window state including focus, layout, and fullscreen mode.
//! It handles window manipulation operations like focus changes, layout application,
//! and window lifecycle management.

use anyhow::Result;
use std::collections::HashSet;
use tracing::{debug, info};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::bsp::{BspTree, LayoutParams};
use crate::config::Config;

/// Manages all window state and operations
pub struct WindowOperations {
    /// Currently focused window
    pub(crate) focused_window: Option<Window>,
    /// BSP tree for window arrangement (single source of truth for window layout)
    pub(crate) bsp_tree: BspTree,
    /// Currently fullscreen window (if any)
    pub(crate) fullscreen_window: Option<Window>,
    /// Windows we intentionally unmapped (to distinguish from user-closed windows)
    pub(crate) intentionally_unmapped: HashSet<Window>,
    /// Configuration
    pub(crate) config: Config,
    /// Screen number to use
    pub(crate) screen_num: usize,
}

impl WindowOperations {
    /// Creates a new window operations manager
    pub fn new(config: Config, screen_num: usize) -> Self {
        Self {
            focused_window: None,
            bsp_tree: BspTree::new(),
            fullscreen_window: None,
            intentionally_unmapped: HashSet::new(),
            config,
            screen_num,
        }
    }

    /// Gets the currently focused window
    pub fn get_focused_window(&self) -> Option<Window> {
        self.focused_window
    }

    /// Clears the focused window
    pub fn clear_focus(&mut self) {
        self.focused_window = None;
    }

    /// Gets the current fullscreen window
    pub fn get_fullscreen_window(&self) -> Option<Window> {
        self.fullscreen_window
    }

    /// Clears fullscreen state
    pub fn clear_fullscreen(&mut self) {
        self.fullscreen_window = None;
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

    /// Checks if a window is intentionally unmapped
    pub fn is_intentionally_unmapped(&self, window: Window) -> bool {
        self.intentionally_unmapped.contains(&window)
    }

    /// Removes a window from the intentionally unmapped set
    pub fn remove_intentionally_unmapped(&mut self, window: Window) {
        self.intentionally_unmapped.remove(&window);
    }

    /// Gets the unfocused border color from config
    pub fn unfocused_border_color(&self) -> u32 {
        self.config.unfocused_border_color()
    }

    /// Gets the first window in the layout, or None if empty
    pub fn get_first_window(&self) -> Option<Window> {
        self.get_all_windows().first().copied()
    }

    /// Sets focus to a specific window
    pub fn set_focus<C: Connection>(&mut self, conn: &mut C, window: Window) -> Result<()> {
        if !self.has_window(window) {
            return Ok(());
        }

        // Set X11 input focus
        conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;

        // Update focus state
        self.focused_window = Some(window);

        // Update window borders
        self.update_window_borders(conn)?;

        #[cfg(debug_assertions)]
        debug!("Focus set to window: {:?}", window);
        Ok(())
    }

    /// Updates window borders based on focus state
    fn update_window_borders<C: Connection>(&self, conn: &C) -> Result<()> {
        for &window in &self.get_all_windows() {
            let border_color = self.border_color_for_window(window);
            self.configure_window_border(conn, window, border_color)?;
        }
        Ok(())
    }

    /// Focuses the next window in the stack
    pub fn focus_next<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
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
                None => return Ok(()),
            }
        };

        // Exit fullscreen if trying to focus a different window
        if self.fullscreen_window.is_some() && self.fullscreen_window != Some(next_window) {
            info!("Exiting fullscreen mode to focus different window");
            self.fullscreen_window = None;
            self.apply_layout(conn)?;
        }

        self.set_focus(conn, next_window)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses the previous window in the stack
    pub fn focus_prev<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
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
                None => return Ok(()),
            }
        };

        // Exit fullscreen if trying to focus a different window
        if self.fullscreen_window.is_some() && self.fullscreen_window != Some(prev_window) {
            info!("Exiting fullscreen mode to focus different window");
            self.fullscreen_window = None;
            self.apply_layout(conn)?;
        }

        self.set_focus(conn, prev_window)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }

    /// Configures window border color and width - helper to reduce duplication
    pub fn configure_window_border<C: Connection>(
        &self,
        conn: &C,
        window: Window,
        border_color: u32,
    ) -> Result<()> {
        let border_aux = ChangeWindowAttributesAux::new().border_pixel(border_color);
        conn.change_window_attributes(window, &border_aux)?;

        let config_aux = ConfigureWindowAux::new().border_width(self.config.border_width());
        conn.configure_window(window, &config_aux)?;

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

    /// Adds a window to the layout manager
    pub fn add_window_to_layout(&mut self, window: Window) {
        self.bsp_tree
            .add_window(window, self.focused_window, self.config.bsp_split_ratio());
    }

    /// Removes a window from the layout manager
    pub fn remove_window_from_layout(&mut self, window: Window) {
        self.bsp_tree.remove_window(window);
    }

    /// Applies the current BSP tree layout without rebuilding the tree
    pub fn apply_layout<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
        if self.window_count() == 0 {
            return Ok(());
        }

        // If we're in fullscreen mode, apply fullscreen layout instead
        if self.fullscreen_window.is_some() {
            return self.apply_fullscreen_layout(conn);
        }

        let setup = conn.setup();
        let screen = &setup.roots[self.screen_num];

        // Ensure all windows are mapped (visible) and have borders when not in fullscreen
        let border_width = self.config.border_width();
        for &window in &self.get_all_windows() {
            conn.map_window(window)?;
            // Remove from intentionally unmapped set when restoring
            self.intentionally_unmapped.remove(&window);
            // Restore border width
            conn.configure_window(
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
        if let Some(focused) = self.focused_window {
            conn.configure_window(
                focused,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        conn.flush()?;

        #[cfg(debug_assertions)]
        tracing::debug!(
            "Applied existing BSP tree layout to {} windows",
            geometries.len()
        );

        Ok(())
    }

    /// Destroys (closes) the currently focused window
    pub fn destroy_focused_window<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
        if let Some(focused) = self.focused_window {
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
        let wm_delete_window = conn
            .intern_atom(false, b"WM_DELETE_WINDOW")?
            .reply()?
            .atom;

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
    pub fn swap_window_next<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
        self.swap_window_direction(conn, SwapDirection::Next)
    }

    /// Swaps the currently focused window with the previous window in the layout
    pub fn swap_window_prev<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
        self.swap_window_direction(conn, SwapDirection::Previous)
    }

    /// Helper method to swap windows in a given direction
    fn swap_window_direction<C: Connection>(
        &mut self,
        conn: &mut C,
        direction: SwapDirection,
    ) -> Result<()> {
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
                self.apply_layout(conn)?;
            }
        }
        Ok(())
    }

    /// Toggles fullscreen mode for the focused window
    pub fn toggle_fullscreen<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
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
                self.apply_layout(conn)?;
            } else {
                // Different window wants fullscreen, switch to it
                info!(
                    "Switching fullscreen from {:?} to {:?}",
                    fullscreen, focused
                );
                self.fullscreen_window = Some(focused);
                self.apply_fullscreen_layout(conn)?;
            }
        } else {
            // Enter fullscreen mode
            info!("Entering fullscreen mode for window {:?}", focused);
            self.fullscreen_window = Some(focused);
            self.apply_fullscreen_layout(conn)?;
        }

        Ok(())
    }

    /// Applies fullscreen layout - window takes entire screen
    fn apply_fullscreen_layout<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
        if let Some(fullscreen) = self.fullscreen_window {
            let setup = conn.setup();
            let screen = &setup.roots[self.screen_num];

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
            for &window in &self.get_all_windows() {
                if window != fullscreen {
                    // Mark as intentionally unmapped BEFORE unmapping
                    self.intentionally_unmapped.insert(window);
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

    /// Rotates the focused window by flipping its parent split direction
    pub fn rotate_windows<C: Connection>(&mut self, conn: &mut C) -> Result<()> {
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
            self.apply_layout(conn)?;
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
    // Window operations tests would go here
    // We'll move existing tests as we complete the refactoring
}
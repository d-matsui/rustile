//! Window operations and layout integration

use anyhow::Result;
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, StackMode, Window,
};

use super::core::WindowManager;
use crate::layout::bsp;

/// Direction for window swapping operations
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,
    Previous,
}

impl<C: Connection> WindowManager<C> {
    /// Adds a window to the layout manager
    pub(super) fn add_window_to_layout(&mut self, window: Window) {
        self.bsp_tree
            .add_window(window, self.focused_window, self.config.bsp_split_ratio());
    }

    /// Removes a window from the layout manager
    pub(super) fn remove_window_from_layout(&mut self, window: Window) {
        self.bsp_tree.remove_window(window);
    }

    /// Gets all windows currently managed by the layout
    pub(super) fn get_all_windows(&self) -> Vec<Window> {
        self.bsp_tree.all_windows()
    }

    /// Gets the total number of windows in the layout
    pub(super) fn window_count(&self) -> usize {
        self.bsp_tree.window_count()
    }

    /// Checks if a window is managed by the layout
    pub(super) fn has_window(&self, window: Window) -> bool {
        self.bsp_tree.has_window(window)
    }

    /// Applies the current BSP tree layout without rebuilding the tree
    pub(super) fn apply_layout(&mut self) -> Result<()> {
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
        let geometries = bsp::calculate_bsp_geometries(
            &self.bsp_tree,
            screen.width_in_pixels,
            screen.height_in_pixels,
            self.config.min_window_width(),
            self.config.min_window_height(),
            self.config.gap(),
        );

        // Update window borders based on focus
        let focused_color = self.config.focused_border_color();
        let unfocused_color = self.config.unfocused_border_color();

        // Apply calculated geometries and update borders
        for geometry in &geometries {
            let is_focused = Some(geometry.window) == self.focused_window;
            let border_color = if is_focused {
                focused_color
            } else {
                unfocused_color
            };

            self.conn.change_window_attributes(
                geometry.window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;

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

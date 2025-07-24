//! Window operations and layout integration

use anyhow::Result;
use tracing::info;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

use super::core::WindowManager;

/// Direction for window swapping operations
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,
    Previous,
}

impl<C: Connection> WindowManager<C> {
    /// Applies the current layout to arrange windows
    pub(super) fn apply_layout(&mut self) -> Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }

        let setup = self.conn.setup();
        let screen = &setup.roots[self.screen_num];

        self.layout_manager.apply_layout(
            &self.conn,
            &self.windows,
            self.focused_window,
            screen.width_in_pixels,
            screen.height_in_pixels,
            self.config.master_ratio(),
            self.config.bsp_split_ratio(),
            self.config.min_window_width(),
            self.config.min_window_height(),
            self.config.gap(),
        )?;

        #[cfg(debug_assertions)]
        tracing::debug!("Applied layout to {} windows", self.windows.len());
        Ok(())
    }

    /// Swaps the currently focused window with the master window
    pub fn swap_with_master(&mut self) -> Result<()> {
        if self.windows.len() < 2 {
            return Ok(());
        }

        if let Some(focused) = self.focused_window {
            if let Some(focused_idx) = self.windows.iter().position(|&w| w == focused) {
                if focused_idx != 0 {
                    // Swap with master (index 0)
                    self.windows.swap(0, focused_idx);
                    info!("Swapped window {:?} with master", focused);

                    // Reapply layout to update window positions on screen
                    self.apply_layout()?;
                }
            }
        }
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
        if self.windows.len() < 2 {
            return Ok(());
        }

        if let Some(focused) = self.focused_window {
            if let Some(focused_idx) = self.windows.iter().position(|&w| w == focused) {
                let target_idx = match direction {
                    SwapDirection::Next => (focused_idx + 1) % self.windows.len(),
                    SwapDirection::Previous => {
                        if focused_idx == 0 {
                            self.windows.len() - 1
                        } else {
                            focused_idx - 1
                        }
                    }
                };

                let target_window = self.windows[target_idx];
                self.windows.swap(focused_idx, target_idx);

                let direction_str = match direction {
                    SwapDirection::Next => "next",
                    SwapDirection::Previous => "previous",
                };

                info!(
                    "Swapped window {:?} with {} window {:?}",
                    focused, direction_str, target_window
                );

                self.apply_layout()?;
            }
        }
        Ok(())
    }
}

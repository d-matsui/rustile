//! Window operations and layout integration

use anyhow::Result;
use tracing::info;
use x11rb::connection::Connection;

use super::core::WindowManager;

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
                }
            }
        }
        Ok(())
    }
}

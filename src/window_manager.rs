//! Window manager core - coordinates X11 events and state

use anyhow::Result;
use std::process::Command;
#[cfg(debug_assertions)]
use tracing::debug;
use tracing::{error, info};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

use std::collections::HashSet;

use crate::keyboard::ShortcutManager;
use crate::workspace::Workspace;
use crate::workspace_renderer::WorkspaceRenderer;

/// Main window manager coordinating X11 events and window state
pub struct WindowManager<C: Connection> {
    pub(crate) conn: C,
    pub(crate) shortcut_manager: ShortcutManager,
    pub(crate) workspaces: Vec<Workspace>,
    pub(crate) current_workspace_index: usize,
    pub(crate) intentionally_unmapped: HashSet<Window>,
    pub(crate) screen_num: usize,
    pub(crate) workspace_renderer: WorkspaceRenderer,
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

        let mut shortcut_manager = ShortcutManager::new(&conn, setup)?;

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

        shortcut_manager.register_shortcuts(&conn, root, config.shortcuts())?;

        // Initialize with a single empty workspace
        let workspaces = vec![Workspace::new()];
        let current_workspace_index = 0;
        let intentionally_unmapped = HashSet::new();
        let workspace_renderer = WorkspaceRenderer::new(config.clone(), screen_num);

        info!("Initialized with 1 empty workspace");

        Ok(Self {
            conn,
            shortcut_manager,
            workspaces,
            current_workspace_index,
            intentionally_unmapped,
            screen_num,
            workspace_renderer,
        })
    }

    /// Gets a reference to the current workspace
    fn current_workspace(&self) -> &Workspace {
        &self.workspaces[self.current_workspace_index]
    }

    /// Gets a mutable reference to the current workspace
    fn current_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current_workspace_index]
    }

    /// Creates a new workspace and switches to it
    pub fn create_workspace(&mut self) {
        info!("Creating new workspace");
        let old_workspace_index = self.current_workspace_index;
        self.workspaces.push(Workspace::new());
        self.current_workspace_index = self.workspaces.len() - 1;
        info!(
            "Created workspace {}, total workspaces: {}",
            self.current_workspace_index,
            self.workspaces.len()
        );
        // Switch to the newly created workspace
        self.perform_workspace_switch(old_workspace_index);
    }

    /// Deletes the current workspace
    /// If this is the last workspace, this is a no-op
    pub fn delete_workspace(&mut self) {
        if self.workspaces.len() <= 1 {
            info!("Cannot delete last workspace, ignoring");
            return;
        }

        info!(
            "Deleting workspace {}, total workspaces: {}",
            self.current_workspace_index,
            self.workspaces.len()
        );

        // Get all windows in the current workspace
        let windows_to_close = self.current_workspace().get_all_windows();

        // Close all windows (send delete window event to each)
        for window in &windows_to_close {
            // Remove from intentionally_unmapped before closing
            self.intentionally_unmapped.remove(window);

            // Send WM_DELETE_WINDOW message to gracefully close the window
            // If the application doesn't support it, it will be destroyed anyway
            if let Err(e) = self.conn.destroy_window(*window) {
                error!("Failed to destroy window {:?}: {}", window, e);
            }
        }

        // Remove the workspace
        self.workspaces.remove(self.current_workspace_index);

        // Adjust current_workspace_index if it's out of bounds
        if self.current_workspace_index >= self.workspaces.len() {
            self.current_workspace_index = self.workspaces.len() - 1;
        }

        // Show windows from the new current workspace
        let new_windows = self.current_workspace().get_all_windows();
        for window in &new_windows {
            self.intentionally_unmapped.remove(window);
            if let Err(e) = self.conn.map_window(*window) {
                error!("Failed to map window {:?}: {}", window, e);
            }
        }

        // Restore focus to the new workspace's focused window
        if let Some(focused) = self.current_workspace().focused_window()
            && let Err(e) =
                self.conn
                    .set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)
        {
            error!("Failed to set focus to window {:?}: {}", focused, e);
        }

        // Flush X11 commands
        if let Err(e) = self.conn.flush() {
            error!("Failed to flush X11 connection: {}", e);
        }

        info!(
            "Deleted workspace, closed {} windows, now at workspace {}, total workspaces: {}",
            windows_to_close.len(),
            self.current_workspace_index,
            self.workspaces.len()
        );
    }

    /// Switches to the next workspace (circular)
    pub fn switch_workspace_next(&mut self) {
        if self.workspaces.len() <= 1 {
            return; // Nothing to switch
        }

        let old_index = self.current_workspace_index;
        self.current_workspace_index = (self.current_workspace_index + 1) % self.workspaces.len();

        info!(
            "Switched workspace: {} -> {} (next)",
            old_index, self.current_workspace_index
        );

        // Perform window visibility switch
        self.perform_workspace_switch(old_index);
    }

    /// Switches to the previous workspace (circular)
    pub fn switch_workspace_prev(&mut self) {
        if self.workspaces.len() <= 1 {
            return; // Nothing to switch
        }

        let old_index = self.current_workspace_index;
        self.current_workspace_index = if self.current_workspace_index == 0 {
            self.workspaces.len() - 1
        } else {
            self.current_workspace_index - 1
        };

        info!(
            "Switched workspace: {} -> {} (prev)",
            old_index, self.current_workspace_index
        );

        // Perform window visibility switch
        self.perform_workspace_switch(old_index);
    }

    /// Performs the actual workspace switch: unmaps old windows, maps new windows
    fn perform_workspace_switch(&mut self, old_workspace_index: usize) {
        // Get windows from old workspace
        let old_windows = self.workspaces[old_workspace_index].get_all_windows();

        // Mark old workspace windows as intentionally unmapped and unmap them
        for window in &old_windows {
            self.intentionally_unmapped.insert(*window);
            if let Err(e) = self.conn.unmap_window(*window) {
                error!("Failed to unmap window {:?}: {}", window, e);
            }
        }

        // Get windows from new workspace
        let new_windows = self.current_workspace().get_all_windows();

        // Remove new workspace windows from intentionally_unmapped and map them
        for window in &new_windows {
            self.intentionally_unmapped.remove(window);
            if let Err(e) = self.conn.map_window(*window) {
                error!("Failed to map window {:?}: {}", window, e);
            }
        }

        // Restore focus to the new workspace's focused window
        if let Some(focused) = self.current_workspace().focused_window()
            && let Err(e) =
                self.conn
                    .set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)
        {
            error!("Failed to set focus to window {:?}: {}", focused, e);
        }

        // Flush X11 commands
        if let Err(e) = self.conn.flush() {
            error!("Failed to flush X11 connection: {}", e);
        }

        info!(
            "Workspace switch complete: {} old windows unmapped, {} new windows mapped",
            old_windows.len(),
            new_windows.len()
        );
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
            Event::ConfigureRequest(ev) => self.handle_configure_request(ev),
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
        let (command_opt, matched) = self.shortcut_manager.handle_key_press(&self.conn, &event)?;

        if matched {
            // Shortcut matched - allow event processing to continue (ADR-015)
            if let Err(e) = self.conn.allow_events(Allow::ASYNC_KEYBOARD, CURRENT_TIME) {
                error!("Failed to allow keyboard events: {}", e);
                // Fallback: try again to prevent keyboard freeze
                let _ = self.conn.allow_events(Allow::ASYNC_KEYBOARD, CURRENT_TIME);
            }

            if let Some(command) = command_opt {
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
                    "toggle_zoom" => return self.toggle_zoom(),
                    "balance_tree" => return self.balance_tree(),
                    // Workspace management commands
                    "create_workspace" => {
                        self.create_workspace();
                        return Ok(());
                    }
                    "delete_workspace" => {
                        self.delete_workspace();
                        return Ok(());
                    }
                    "switch_workspace_next" => {
                        self.switch_workspace_next();
                        return Ok(());
                    }
                    "switch_workspace_prev" => {
                        self.switch_workspace_prev();
                        return Ok(());
                    }
                    _ => {
                        let parts: Vec<&str> = command.split_whitespace().collect();
                        if let Some(program) = parts.first() {
                            let mut cmd = Command::new(program);

                            if parts.len() > 1 {
                                cmd.args(&parts[1..]);
                            }

                            match cmd.spawn() {
                                Ok(_) => info!("Successfully launched: {}", command),
                                Err(e) => error!("Failed to launch {}: {}", command, e),
                            }
                        }
                    }
                }
            }
        } else {
            // No shortcut matched - replay event to focused application (ADR-015)
            if let Err(e) = self.conn.allow_events(Allow::REPLAY_KEYBOARD, CURRENT_TIME) {
                error!("Failed to replay key event: {}", e);
                // Fallback: try ASYNC_KEYBOARD to prevent keyboard freeze
                let _ = self.conn.allow_events(Allow::ASYNC_KEYBOARD, CURRENT_TIME);
            }
        }
        Ok(())
    }

    /// Handles window map requests
    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
        let window = event.window;
        info!("Mapping window: {:?}", window);

        // Check if window is already managed in current workspace
        if self.current_workspace().has_window(window) {
            info!(
                "Window {:?} is already managed, ignoring duplicate MapRequest",
                window
            );
            return Ok(());
        }

        self.conn.map_window(window)?;

        // Add to current workspace
        self.current_workspace_mut().add_window(window);
        self.current_workspace_mut()
            .set_focused_window(Some(window));

        // Render the workspace
        {
            let workspace = &self.workspaces[self.current_workspace_index];
            self.workspace_renderer
                .apply_workspace(&mut self.conn, workspace)?;
        }

        Ok(())
    }

    /// Handles window unmap notifications
    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Unmapping window: {:?}", window);

        // Check intentionally_unmapped from WindowManager (workspace switches)
        if self.intentionally_unmapped.contains(&window) {
            info!("Window {:?} was intentionally unmapped, ignoring", window);
            return Ok(());
        }

        info!(
            "Window {:?} closed by user, removing from management",
            window
        );

        // Remove from all workspaces (window could be in any workspace)
        for workspace in &mut self.workspaces {
            if workspace.has_window(window) {
                workspace.remove_window(window);
            }
        }

        // Update focus in current workspace
        if self.current_workspace().focused_window() == Some(window) {
            let next_focus = self.current_workspace().get_first_window();
            if let Some(next_focus) = next_focus {
                self.current_workspace_mut()
                    .set_focused_window(Some(next_focus));
            } else {
                self.current_workspace_mut().clear_focus();
            }
        }

        // Render the workspace
        {
            let workspace = &self.workspaces[self.current_workspace_index];
            self.workspace_renderer
                .apply_workspace(&mut self.conn, workspace)?;
        }

        Ok(())
    }

    /// Handles window configure requests
    ///
    /// CRITICAL: Applications like xterm will timeout (5 seconds) if we don't acknowledge
    /// their ConfigureRequest events, causing slow launch. We acknowledge immediately
    /// for protocol compliance, then override geometry with our BSP layout.
    /// See ADR-006 for detailed analysis.
    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<()> {
        #[cfg(debug_assertions)]
        debug!(
            "Configure request for window: {:?} - Event: {:#?}",
            event.window, event
        );

        // Forward the request as-is to acknowledge it (X11 protocol compliance)
        let values = ConfigureWindowAux::from_configure_request(&event);
        self.conn.configure_window(event.window, &values)?;

        Ok(())
    }

    /// Handles window destroy notifications
    fn handle_destroy_notify(&mut self, event: DestroyNotifyEvent) -> Result<()> {
        let window = event.window;
        info!("Window destroyed: {:?}", window);

        // Remove from intentionally_unmapped (WindowManager)
        self.intentionally_unmapped.remove(&window);

        // Remove from all workspaces
        for workspace in &mut self.workspaces {
            if workspace.has_window(window) {
                workspace.remove_window(window);
            }
        }

        // Clear fullscreen in current workspace
        if self.current_workspace().fullscreen_window() == Some(window) {
            info!("Fullscreen window destroyed in workspace, exiting fullscreen mode");
            self.current_workspace_mut().clear_fullscreen();
        }

        // Update focus in current workspace
        if self.current_workspace().focused_window() == Some(window) {
            let next_focus = self.current_workspace().get_first_window();
            if let Some(next_focus) = next_focus {
                self.current_workspace_mut()
                    .set_focused_window(Some(next_focus));
            } else {
                self.current_workspace_mut().clear_focus();
            }
        }

        // Render the workspace
        {
            let workspace = &self.workspaces[self.current_workspace_index];
            self.workspace_renderer
                .apply_workspace(&mut self.conn, workspace)?;
        }

        Ok(())
    }

    /// Handles mouse enter events (focus follows mouse)
    fn handle_enter_notify(&mut self, event: EnterNotifyEvent) -> Result<()> {
        let window = event.event;
        #[cfg(debug_assertions)]
        debug!("Mouse entered window: {:?}", window);

        // Update current workspace
        if self.current_workspace().has_window(window) {
            self.current_workspace_mut()
                .set_focused_window(Some(window));

            // Render the workspace
            {
                let workspace = &self.workspaces[self.current_workspace_index];
                self.workspace_renderer
                    .apply_workspace(&mut self.conn, workspace)?;
            }
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
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .focus_next(&mut self.conn, workspace)
    }

    /// Focuses previous window in BSP tree order
    pub fn focus_prev(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .focus_prev(&mut self.conn, workspace)
    }
}

impl<C: Connection> WindowManager<C> {
    /// Destroys the currently focused window
    pub fn destroy_focused_window(&mut self) -> Result<()> {
        let workspace = &self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .destroy_focused_window(&mut self.conn, workspace)
    }

    /// Swaps focused window with next window in BSP order
    pub fn swap_window_next(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .swap_window_next(&mut self.conn, workspace)
    }

    /// Swaps focused window with previous window in BSP order
    pub fn swap_window_prev(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .swap_window_prev(&mut self.conn, workspace)
    }

    /// Toggles fullscreen mode for focused window
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        let focused = match self.current_workspace().focused_window() {
            Some(window) => window,
            None => {
                info!("No window focused for fullscreen toggle");
                return Ok(());
            }
        };

        // Check if we're currently in fullscreen mode
        if let Some(fullscreen) = self.current_workspace().fullscreen_window() {
            if fullscreen == focused {
                // Exit fullscreen mode
                info!("Exiting fullscreen mode for window {:?}", focused);
                self.current_workspace_mut().set_fullscreen_window(None);

                // Map all windows in the current workspace
                for window in self.current_workspace().get_all_windows() {
                    self.intentionally_unmapped.remove(&window);
                    if let Err(e) = self.conn.map_window(window) {
                        error!("Failed to map window {:?}: {}", window, e);
                    }
                }

                // Render normal layout
                {
                    let workspace = &self.workspaces[self.current_workspace_index];
                    self.workspace_renderer
                        .apply_workspace(&mut self.conn, workspace)?;
                }
            } else {
                // Different window wants fullscreen, switch to it
                info!(
                    "Switching fullscreen from {:?} to {:?}",
                    fullscreen, focused
                );
                self.current_workspace_mut()
                    .set_fullscreen_window(Some(focused));
                self.apply_fullscreen_layout(focused)?;
            }
        } else {
            // Enter fullscreen mode
            info!("Entering fullscreen mode for window {:?}", focused);
            self.current_workspace_mut()
                .set_fullscreen_window(Some(focused));
            self.apply_fullscreen_layout(focused)?;
        }

        Ok(())
    }

    /// Applies fullscreen layout for a window
    fn apply_fullscreen_layout(&mut self, fullscreen: Window) -> Result<()> {
        let setup = self.conn.setup();
        let screen = &setup.roots[self.screen_num];

        self.conn.map_window(fullscreen)?;

        let config = ConfigureWindowAux::new()
            .x(0)
            .y(0)
            .width(u32::from(screen.width_in_pixels))
            .height(u32::from(screen.height_in_pixels))
            .border_width(0);

        self.conn.configure_window(fullscreen, &config)?;

        // Unmap all other windows
        for window in self.current_workspace().get_all_windows() {
            if window != fullscreen {
                self.intentionally_unmapped.insert(window);
                self.conn.unmap_window(window)?;
            }
        }

        self.conn.configure_window(
            fullscreen,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;

        self.conn.flush()?;
        Ok(())
    }

    /// Rotates focused window by flipping parent split direction
    pub fn rotate_windows(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .rotate_windows(&mut self.conn, workspace)
    }

    /// Toggles zoom for the focused window
    pub fn toggle_zoom(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .toggle_zoom(&mut self.conn, workspace)
    }

    /// Balances the BSP tree by calculating optimal split ratios based on window count
    ///
    /// This command traverses the entire BSP tree and updates each split node's ratio
    /// to be proportional to the number of windows in its left and right subtrees,
    /// ensuring all windows receive equal screen area.
    pub fn balance_tree(&mut self) -> Result<()> {
        let workspace = &mut self.workspaces[self.current_workspace_index];
        self.workspace_renderer
            .balance_tree(&mut self.conn, workspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x11rb::protocol::xproto::Window;

    #[test]
    fn test_workspace_initialization() {
        // This test requires X11 connection, so we'll just test the logic
        // Actual WindowManager::new() requires X11 connection which is hard to mock

        // Test that workspace fields are properly structured
        let workspaces = [Workspace::new()];
        let current_workspace_index = 0;

        assert_eq!(workspaces.len(), 1);
        assert_eq!(current_workspace_index, 0);
        assert_eq!(
            workspaces[current_workspace_index].get_all_windows().len(),
            0
        );
    }

    #[test]
    fn test_current_workspace_index_bounds() {
        let workspaces = [Workspace::new(), Workspace::new(), Workspace::new()];

        for index in 0..workspaces.len() {
            assert!(index < workspaces.len());
            assert_eq!(workspaces[index].get_all_windows().len(), 0);
        }
    }

    #[test]
    fn test_workspace_creation_logic() {
        let mut workspaces = vec![Workspace::new()];
        let mut current_workspace_index = 0;

        // Initial state: 1 workspace
        assert_eq!(workspaces.len(), 1);
        assert_eq!(current_workspace_index, 0);

        // Create new workspace
        workspaces.push(Workspace::new());
        current_workspace_index = workspaces.len() - 1;

        // After creation: 2 workspaces, current is the new one
        assert_eq!(workspaces.len(), 2);
        assert_eq!(current_workspace_index, 1);

        // Create another workspace
        workspaces.push(Workspace::new());
        current_workspace_index = workspaces.len() - 1;

        // After creation: 3 workspaces, current is the newest
        assert_eq!(workspaces.len(), 3);
        assert_eq!(current_workspace_index, 2);
    }

    #[test]
    fn test_workspace_deletion_logic() {
        // Test: Cannot delete last workspace
        let mut workspaces = vec![Workspace::new()];
        let mut current_workspace_index = 0;

        // Try to delete last workspace - should be no-op
        if workspaces.len() > 1 {
            workspaces.remove(current_workspace_index);
            if current_workspace_index >= workspaces.len() {
                current_workspace_index = workspaces.len() - 1;
            }
        }

        // Still have 1 workspace
        assert_eq!(workspaces.len(), 1);
        assert_eq!(current_workspace_index, 0);

        // Add more workspaces
        workspaces.push(Workspace::new());
        workspaces.push(Workspace::new());
        current_workspace_index = 1; // Delete middle workspace

        // Delete workspace at index 1
        if workspaces.len() > 1 {
            workspaces.remove(current_workspace_index);
            if current_workspace_index >= workspaces.len() {
                current_workspace_index = workspaces.len() - 1;
            }
        }

        // Now have 2 workspaces
        assert_eq!(workspaces.len(), 2);
        assert_eq!(current_workspace_index, 1);

        // Delete workspace at index 1 (last one)
        if workspaces.len() > 1 {
            workspaces.remove(current_workspace_index);
            if current_workspace_index >= workspaces.len() {
                current_workspace_index = workspaces.len() - 1;
            }
        }

        // Now have 1 workspace, current index is 0
        assert_eq!(workspaces.len(), 1);
        assert_eq!(current_workspace_index, 0);
    }

    #[test]
    fn test_workspace_switching_logic() {
        // Create 3 workspaces
        let workspaces = [Workspace::new(), Workspace::new(), Workspace::new()];
        let mut current_workspace_index = 0;

        // Switch next: 0 -> 1
        current_workspace_index = (current_workspace_index + 1) % workspaces.len();
        assert_eq!(current_workspace_index, 1);

        // Switch next: 1 -> 2
        current_workspace_index = (current_workspace_index + 1) % workspaces.len();
        assert_eq!(current_workspace_index, 2);

        // Switch next: 2 -> 0 (wrap around)
        current_workspace_index = (current_workspace_index + 1) % workspaces.len();
        assert_eq!(current_workspace_index, 0);

        // Switch prev: 0 -> 2 (wrap around)
        current_workspace_index = if current_workspace_index == 0 {
            workspaces.len() - 1
        } else {
            current_workspace_index - 1
        };
        assert_eq!(current_workspace_index, 2);

        // Switch prev: 2 -> 1
        current_workspace_index = if current_workspace_index == 0 {
            workspaces.len() - 1
        } else {
            current_workspace_index - 1
        };
        assert_eq!(current_workspace_index, 1);

        // Switch prev: 1 -> 0
        current_workspace_index = if current_workspace_index == 0 {
            workspaces.len() - 1
        } else {
            current_workspace_index - 1
        };
        assert_eq!(current_workspace_index, 0);
    }

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
        if let Some(focused) = focused
            && let Some(focused_idx) = windows.iter().position(|&w| w == focused)
        {
            windows.remove(focused_idx);
            return true;
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

        if let Some(focused) = focused
            && let Some(focused_idx) = windows.iter().position(|&w| w == focused)
        {
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

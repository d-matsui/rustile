//! Workspace rendering and X11 operations

use anyhow::Result;
#[cfg(debug_assertions)]
use tracing::debug;
use tracing::info;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::bsp::{BspNode, BspTree, SplitDirection, dimensions};
use crate::config::Config;
use crate::workspace::Workspace;

// === Geometry Types ===

/// Rectangle for BSP layout calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BspRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Represents a calculated window position and size
#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    pub window: Window,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Layout parameters bundle to reduce parameter passing
#[derive(Debug, Clone, Copy)]
pub struct LayoutParams {
    pub min_window_width: u32,
    pub min_window_height: u32,
    pub gap: u32,
}

/// Handles X11 rendering operations for workspaces
pub struct WorkspaceRenderer {
    config: Config,
    screen_num: usize,
}

impl WorkspaceRenderer {
    /// Creates a new workspace renderer
    pub fn new(config: Config, screen_num: usize) -> Self {
        Self { config, screen_num }
    }

    /// Focuses next window in BSP order
    pub fn focus_next<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        if workspace.get_all_windows().is_empty() {
            return Ok(());
        }

        let next_window = if let Some(current) = workspace.focused_window() {
            workspace.bsp_tree().next_window(current).unwrap_or(current)
        } else {
            match workspace.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        if workspace.fullscreen_window().is_some()
            && workspace.fullscreen_window() != Some(next_window)
        {
            info!("Exiting fullscreen mode to focus different window");
            workspace.clear_fullscreen();
        }

        workspace.set_focused_window(Some(next_window));
        self.apply_workspace(conn, workspace)?;
        info!("Focused next window: {:?}", next_window);
        Ok(())
    }

    /// Focuses previous window in BSP order
    pub fn focus_prev<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        if workspace.get_all_windows().is_empty() {
            return Ok(());
        }

        let prev_window = if let Some(current) = workspace.focused_window() {
            workspace.bsp_tree().prev_window(current).unwrap_or(current)
        } else {
            match workspace.get_first_window() {
                Some(window) => window,
                None => return Ok(()),
            }
        };

        if workspace.fullscreen_window().is_some()
            && workspace.fullscreen_window() != Some(prev_window)
        {
            info!("Exiting fullscreen mode to focus different window");
            workspace.clear_fullscreen();
        }

        workspace.set_focused_window(Some(prev_window));
        self.apply_workspace(conn, workspace)?;
        info!("Focused previous window: {:?}", prev_window);
        Ok(())
    }

    /// Applies current workspace state to screen (unified rendering method)
    pub fn apply_workspace<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &Workspace,
    ) -> Result<()> {
        if workspace.get_all_windows().is_empty() {
            return Ok(());
        }

        // Don't apply normal layout if in fullscreen mode
        // Fullscreen layout is managed by WindowManager
        if workspace.fullscreen_window().is_some() {
            #[cfg(debug_assertions)]
            debug!("Skipping layout application (in fullscreen mode)");
            // Still set focus if needed
            if let Some(focused) = workspace.focused_window() {
                conn.set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)?;
            }
            conn.flush()?;
            return Ok(());
        }

        // Apply normal layout
        self.apply_normal_layout(conn, workspace)?;

        if let Some(focused) = workspace.focused_window() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, focused, CURRENT_TIME)?;
            conn.configure_window(
                focused,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        conn.flush()?;

        #[cfg(debug_assertions)]
        debug!(
            "Applied complete workspace state to screen: {} windows",
            workspace.get_all_windows().len()
        );

        Ok(())
    }

    /// Applies normal tiled layout
    fn apply_normal_layout<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &Workspace,
    ) -> Result<()> {
        let setup = conn.setup();
        let screen = &setup.roots[self.screen_num];

        let border_width = self.config.border_width();
        for &window in &workspace.get_all_windows() {
            conn.map_window(window)?;
            conn.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;
        }

        let mut geometries = self.calculate_window_geometries(
            workspace,
            screen.width_in_pixels,
            screen.height_in_pixels,
        );

        // Apply zoom if a window is zoomed
        if let Some(zoomed_window) = workspace.zoomed_window() {
            let screen_rect =
                self.calculate_screen_rect(screen.width_in_pixels, screen.height_in_pixels);

            // Find parent bounds for the zoomed window
            if let Some(parent_bounds) = workspace
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
                            .max(self.layout_params().min_window_width as i32)
                            as u32;
                        geometry.height = parent_bounds
                            .height
                            .max(self.layout_params().min_window_height as i32)
                            as u32;
                        break;
                    }
                }
            }
        }

        for geometry in &geometries {
            let border_color = self.border_color_for_window(workspace, geometry.window);

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
            if Some(geometry.window) == workspace.zoomed_window() {
                config = config.stack_mode(StackMode::ABOVE);
            }

            conn.configure_window(geometry.window, &config)?;
        }

        Ok(())
    }

    /// Destroys the currently focused window
    pub fn destroy_focused_window<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &Workspace,
    ) -> Result<()> {
        if let Some(focused) = workspace.focused_window() {
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
        workspace: &mut Workspace,
    ) -> Result<()> {
        // Don't allow zoom in fullscreen mode
        if workspace.fullscreen_window().is_some() {
            info!("Cannot zoom while in fullscreen mode");
            return Ok(());
        }

        // Get the focused window
        let focused = match workspace.focused_window() {
            Some(window) => window,
            None => {
                info!("No focused window to zoom");
                return Ok(());
            }
        };

        // Get screen dimensions for parent bounds calculation
        let setup = conn.setup();
        let screen = &setup.roots[self.screen_num];
        let screen_rect =
            self.calculate_screen_rect(screen.width_in_pixels, screen.height_in_pixels);

        // Toggle zoom state
        if workspace.zoomed_window() == Some(focused) {
            // Already zoomed - unzoom
            workspace.set_zoomed_window(None);
            info!("Unzoomed window: {:?}", focused);
        } else {
            // Check if this window can be zoomed (has a parent)
            if workspace
                .bsp_tree()
                .find_parent_bounds(focused, screen_rect)
                .is_none()
            {
                // Single window or root - cannot zoom
                info!("Window {:?} has no parent to zoom to", focused);
                return Ok(());
            }

            // Zoom the focused window
            workspace.set_zoomed_window(Some(focused));
            info!("Zoomed window: {:?}", focused);
        }

        // Apply the new state
        self.apply_workspace(conn, workspace)?;
        Ok(())
    }

    /// Swaps the currently focused window with the next window in the layout
    pub fn swap_window_next<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        self.swap_window_direction(conn, workspace, SwapDirection::Next)
    }

    /// Swaps the currently focused window with the previous window in the layout
    pub fn swap_window_prev<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        self.swap_window_direction(conn, workspace, SwapDirection::Previous)
    }

    /// Helper method to swap windows in a given direction
    fn swap_window_direction<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
        direction: SwapDirection,
    ) -> Result<()> {
        if workspace.get_all_windows().len() < 2 {
            return Ok(());
        }

        // Exit fullscreen if active, then perform swap
        if workspace.fullscreen_window().is_some() {
            info!("Exiting fullscreen for window swap");
            workspace.clear_fullscreen();
        }

        if let Some(focused) = workspace.focused_window() {
            let target_window = match direction {
                SwapDirection::Next => workspace.bsp_tree().next_window(focused),
                SwapDirection::Previous => workspace.bsp_tree().prev_window(focused),
            };

            if let Some(target_window) = target_window {
                // Swap windows in the BSP tree
                workspace
                    .bsp_tree_mut()
                    .swap_windows(focused, target_window);

                let direction_str = match direction {
                    SwapDirection::Next => "next",
                    SwapDirection::Previous => "previous",
                };

                info!(
                    "Swapped window {:?} with {} window {:?}",
                    focused, direction_str, target_window
                );

                // Apply complete state to screen
                self.apply_workspace(conn, workspace)?;
            }
        }
        Ok(())
    }

    /// Rotates the focused window by flipping its parent split direction
    pub fn rotate_windows<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        let focused = match workspace.focused_window() {
            Some(window) => window,
            None => {
                info!("No window focused for rotation");
                return Ok(());
            }
        };

        // Cannot rotate in fullscreen mode
        if workspace.fullscreen_window().is_some() {
            info!("Cannot rotate in fullscreen mode");
            return Ok(());
        }

        // Need at least 2 windows to rotate
        if workspace.get_all_windows().len() < 2 {
            info!("Not enough windows to rotate (need at least 2)");
            return Ok(());
        }

        info!(
            "Rotating parent split direction for focused window {:?}",
            focused
        );

        // Rotate the focused window in the BSP tree
        let rotated = workspace.bsp_tree_mut().rotate_window(focused);

        if rotated {
            // Apply complete state to screen
            self.apply_workspace(conn, workspace)?;
            info!("Window rotation completed for window {:?}", focused);
        } else {
            info!(
                "No rotation performed - window {:?} may be root or not found",
                focused
            );
        }

        Ok(())
    }

    /// Balances the BSP tree by calculating optimal split ratios based on window count
    pub fn balance_tree<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &mut Workspace,
    ) -> Result<()> {
        info!("Balancing BSP tree");

        // Balance the tree
        workspace.bsp_tree_mut().balance_tree();

        // Apply the balanced layout
        self.apply_workspace(conn, workspace)?;

        info!("BSP tree balanced and applied");
        Ok(())
    }

    // === Helper methods ===

    /// Calculates window geometries from the BSP tree (pure calculation - no X11 calls)
    fn calculate_window_geometries(
        &self,
        workspace: &Workspace,
        screen_width: u16,
        screen_height: u16,
    ) -> Vec<WindowGeometry> {
        let params = self.layout_params();
        calculate_bsp_geometries(workspace.bsp_tree(), screen_width, screen_height, params)
    }

    /// Calculates the screen rectangle with gap and minimum size constraints
    fn calculate_screen_rect(&self, screen_width: u16, screen_height: u16) -> BspRect {
        let params = self.layout_params();
        BspRect {
            x: params.gap as i32,
            y: params.gap as i32,
            width: (screen_width as i32 - 2 * params.gap as i32)
                .max(params.min_window_width as i32),
            height: (screen_height as i32 - 2 * params.gap as i32)
                .max(params.min_window_height as i32),
        }
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
    fn border_color_for_window(&self, workspace: &Workspace, window: Window) -> u32 {
        if Some(window) == workspace.focused_window() {
            self.config.focused_border_color()
        } else {
            self.config.unfocused_border_color()
        }
    }
}

/// Direction for window swapping operations
#[derive(Debug, Clone, Copy)]
enum SwapDirection {
    Next,
    Previous,
}

// === BSP Geometry Calculation Functions ===

/// Calculate window geometries without applying them (pure calculation)
pub fn calculate_bsp_geometries(
    bsp_tree: &BspTree,
    screen_width: u16,
    screen_height: u16,
    params: LayoutParams,
) -> Vec<WindowGeometry> {
    let mut geometries = Vec::new();

    if let Some(ref root) = bsp_tree.root {
        // Create screen rect with gap and minimum size constraints
        let screen_rect = BspRect {
            x: params.gap as i32,
            y: params.gap as i32,
            width: (screen_width as i32 - 2 * params.gap as i32)
                .max(params.min_window_width as i32),
            height: (screen_height as i32 - 2 * params.gap as i32)
                .max(params.min_window_height as i32),
        };

        calculate_bsp_recursive(
            root,
            screen_rect,
            params.min_window_width,
            params.min_window_height,
            params.gap,
            &mut geometries,
        );
    }

    geometries
}

/// Recursively calculate window geometries for BSP nodes
fn calculate_bsp_recursive(
    node: &BspNode,
    rect: BspRect,
    min_window_width: u32,
    min_window_height: u32,
    gap: u32,
    geometries: &mut Vec<WindowGeometry>,
) {
    match node {
        BspNode::Leaf(window) => {
            geometries.push(WindowGeometry {
                window: *window,
                x: rect.x,
                y: rect.y,
                width: rect.width.max(dimensions::MIN_WINDOW_WIDTH as i32) as u32,
                height: rect.height.max(dimensions::MIN_WINDOW_HEIGHT as i32) as u32,
            });
        }
        BspNode::Split {
            direction,
            ratio,
            left,
            right,
        } => {
            let gap_i32 = gap as i32;
            let (left_rect, right_rect) = match direction {
                SplitDirection::Horizontal => {
                    // Split left/right (horizontal arrangement)
                    let split_pos = (rect.width as f32 * ratio) as i32;
                    let left_rect = BspRect {
                        x: rect.x,
                        y: rect.y,
                        width: (split_pos - gap_i32 / 2).max(min_window_width as i32),
                        height: rect.height,
                    };
                    let right_rect = BspRect {
                        x: rect.x + split_pos + gap_i32 / 2,
                        y: rect.y,
                        width: (rect.width - split_pos - gap_i32 / 2).max(min_window_width as i32),
                        height: rect.height,
                    };
                    (left_rect, right_rect)
                }
                SplitDirection::Vertical => {
                    // Split top/bottom (vertical arrangement)
                    let split_pos = (rect.height as f32 * ratio) as i32;
                    let top_rect = BspRect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: (split_pos - gap_i32 / 2).max(min_window_height as i32),
                    };
                    let bottom_rect = BspRect {
                        x: rect.x,
                        y: rect.y + split_pos + gap_i32 / 2,
                        width: rect.width,
                        height: (rect.height - split_pos - gap_i32 / 2)
                            .max(min_window_height as i32),
                    };
                    (top_rect, bottom_rect)
                }
            };

            // Recursively calculate for children
            calculate_bsp_recursive(
                left,
                left_rect,
                min_window_width,
                min_window_height,
                gap,
                geometries,
            );
            calculate_bsp_recursive(
                right,
                right_rect,
                min_window_width,
                min_window_height,
                gap,
                geometries,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_renderer_creation() {
        let config = Config::default();
        let renderer = WorkspaceRenderer::new(config, 0);
        assert_eq!(renderer.screen_num, 0);
    }

    #[test]
    fn test_swap_direction_enum() {
        let next = SwapDirection::Next;
        let prev = SwapDirection::Previous;

        // Test that enum values can be created and compared
        assert!(matches!(next, SwapDirection::Next));
        assert!(matches!(prev, SwapDirection::Previous));
    }

    // Note: Most WorkspaceRenderer methods require X11 connection and are tested
    // through integration tests rather than unit tests
}

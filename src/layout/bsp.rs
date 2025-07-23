//! Binary Space Partitioning (BSP) layout algorithm implementation

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::types::{BspRect, SplitDirection};

/// Represents a node in the BSP tree
#[derive(Debug, Clone)]
pub enum BspNode {
    /// A split with two child nodes
    Split {
        direction: SplitDirection,
        ratio: f32,
        left: Box<BspNode>,
        right: Box<BspNode>,
    },
    /// A leaf containing a window
    Leaf(Window),
}

/// BSP tree for managing window splits
#[derive(Debug, Clone)]
pub struct BspTree {
    pub(crate) root: Option<BspNode>,
    split_count: usize, // To alternate split directions
}

impl Default for BspTree {
    fn default() -> Self {
        Self::new()
    }
}

impl BspTree {
    pub fn new() -> Self {
        Self {
            root: None,
            split_count: 0,
        }
    }

    /// Adds a window to the BSP tree using the simplest algorithm
    pub fn add_window(&mut self, window: Window, focused_window: Option<Window>, split_ratio: f32) {
        if self.root.is_none() {
            // First window - becomes root
            self.root = Some(BspNode::Leaf(window));
            return;
        }

        // Find where to insert the window (split the focused window or last leaf)
        let target_window = focused_window.unwrap_or(window);
        let split_count = self.split_count; // Capture split_count to avoid borrowing issues

        if let Some(ref mut root_node) = self.root {
            Self::insert_window_into_node_static(
                root_node,
                window,
                target_window,
                split_count,
                split_ratio,
            );
        }
        self.split_count += 1;
    }

    /// Recursively find the target window and split it (static version)
    fn insert_window_into_node_static(
        node: &mut BspNode,
        new_window: Window,
        target_window: Window,
        split_count: usize,
        split_ratio: f32,
    ) -> bool {
        match node {
            BspNode::Leaf(existing_window) => {
                if *existing_window == target_window {
                    // Found target - split this leaf
                    let direction = if split_count % 2 == 0 {
                        SplitDirection::Vertical
                    } else {
                        SplitDirection::Horizontal
                    };

                    let old_leaf = BspNode::Leaf(*existing_window);
                    let new_leaf = BspNode::Leaf(new_window);

                    *node = BspNode::Split {
                        direction,
                        ratio: split_ratio,
                        left: Box::new(old_leaf),
                        right: Box::new(new_leaf),
                    };
                    return true;
                }
                false
            }
            BspNode::Split { left, right, .. } => {
                // Try left subtree first
                if Self::contains_window_static(left, target_window) {
                    Self::insert_window_into_node_static(
                        left,
                        new_window,
                        target_window,
                        split_count,
                        split_ratio,
                    )
                } else if Self::contains_window_static(right, target_window) {
                    Self::insert_window_into_node_static(
                        right,
                        new_window,
                        target_window,
                        split_count,
                        split_ratio,
                    )
                } else {
                    false
                }
            }
        }
    }

    /// Check if a subtree contains a specific window (static version to avoid borrow issues)
    fn contains_window_static(node: &BspNode, target_window: Window) -> bool {
        match node {
            BspNode::Leaf(window) => *window == target_window,
            BspNode::Split { left, right, .. } => {
                Self::contains_window_static(left, target_window)
                    || Self::contains_window_static(right, target_window)
            }
        }
    }

    /// Remove a window from the BSP tree
    pub fn remove_window(&mut self, window: Window) {
        if let Some(root_node) = self.root.take() {
            if let Some(replacement) = Self::remove_window_from_node_static(root_node, window) {
                self.root = Some(replacement);
            } else {
                // Window was the only one, clear the tree
                self.root = None;
            }
        }
    }

    /// Remove a window from a node, returning the replacement node (or None if should be removed)
    fn remove_window_from_node_static(node: BspNode, target_window: Window) -> Option<BspNode> {
        match node {
            BspNode::Leaf(window) => {
                if window == target_window {
                    // Remove this leaf
                    None
                } else {
                    // Keep this leaf
                    Some(BspNode::Leaf(window))
                }
            }
            BspNode::Split {
                direction: _direction,
                ratio: _ratio,
                mut left,
                mut right,
            } => {
                // Check if we need to remove from left or right subtree
                let left_contains = Self::contains_window_static(&left, target_window);
                let right_contains = Self::contains_window_static(&right, target_window);

                if left_contains {
                    if let Some(new_left_node) =
                        Self::remove_window_from_node_static(*left, target_window)
                    {
                        left = Box::new(new_left_node);
                        // Keep the split with updated left child
                        Some(BspNode::Split {
                            direction: _direction,
                            ratio: _ratio,
                            left,
                            right,
                        })
                    } else {
                        // Left subtree is empty, replace this split with right subtree
                        Some(*right)
                    }
                } else if right_contains {
                    if let Some(new_right_node) =
                        Self::remove_window_from_node_static(*right, target_window)
                    {
                        right = Box::new(new_right_node);
                        // Keep the split with updated right child
                        Some(BspNode::Split {
                            direction: _direction,
                            ratio: _ratio,
                            left,
                            right,
                        })
                    } else {
                        // Right subtree is empty, replace this split with left subtree
                        Some(*left)
                    }
                } else {
                    // Window not found in this subtree, keep the node unchanged
                    Some(BspNode::Split {
                        direction: _direction,
                        ratio: _ratio,
                        left,
                        right,
                    })
                }
            }
        }
    }
}

/// Tiles windows using BSP layout algorithm
pub fn tile_bsp_windows<C: Connection>(
    conn: &C,
    bsp_tree: &BspTree,
    _windows: &[Window],
    _focused_window: Option<Window>,
    screen_width: u16,
    screen_height: u16,
    _split_ratio: f32,
    min_window_width: u32,
    min_window_height: u32,
    gap: u32,
) -> Result<()> {
    if let Some(ref root) = bsp_tree.root {
        let screen_rect = BspRect {
            x: gap as i32,
            y: gap as i32,
            width: (screen_width as i32 - 2 * gap as i32).max(min_window_width as i32),
            height: (screen_height as i32 - 2 * gap as i32).max(min_window_height as i32),
        };
        #[cfg(debug_assertions)]
        tracing::debug!(
            "BSP: Applying layout to screen {}x{} with gap {}",
            screen_width,
            screen_height,
            gap
        );
        apply_bsp_recursive(conn, root, screen_rect, min_window_width, min_window_height, gap)?;
    } else {
        #[cfg(debug_assertions)]
        tracing::debug!("BSP: No root node, skipping layout");
    }
    Ok(())
}

/// Recursively apply BSP layout to nodes
fn apply_bsp_recursive<C: Connection>(
    conn: &C,
    node: &BspNode,
    rect: BspRect,
    min_window_width: u32,
    min_window_height: u32,
    gap: u32,
) -> Result<()> {
    match node {
        BspNode::Leaf(window) => {
            // Configure the window to fill the rect
            #[cfg(debug_assertions)]
            tracing::debug!(
                "BSP: Positioning window {} at ({}, {}) with size {}x{}",
                window,
                rect.x,
                rect.y,
                rect.width,
                rect.height
            );
            let config = ConfigureWindowAux::new()
                .x(rect.x)
                .y(rect.y)
                .width(rect.width.max(1) as u32)
                .height(rect.height.max(1) as u32);
            conn.configure_window(*window, &config)?;
        }
        BspNode::Split {
            direction,
            ratio,
            left,
            right,
        } => {
            let gap_i32 = gap as i32;
            let (left_rect, right_rect) = match direction {
                SplitDirection::Vertical => {
                    // Split left/right
                    let split_pos = (rect.width as f32 * ratio) as i32;
                    let left_rect = BspRect {
                        x: rect.x,
                        y: rect.y,
                        width: split_pos.max(min_window_width as i32),
                        height: rect.height,
                    };
                    let right_rect = BspRect {
                        x: rect.x + split_pos + gap_i32,
                        y: rect.y,
                        width: (rect.width - split_pos - gap_i32).max(min_window_width as i32),
                        height: rect.height,
                    };
                    (left_rect, right_rect)
                }
                SplitDirection::Horizontal => {
                    // Split top/bottom
                    let split_pos = (rect.height as f32 * ratio) as i32;
                    let left_rect = BspRect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: split_pos.max(min_window_height as i32),
                    };
                    let right_rect = BspRect {
                        x: rect.x,
                        y: rect.y + split_pos + gap_i32,
                        width: rect.width,
                        height: (rect.height - split_pos - gap_i32).max(min_window_height as i32),
                    };
                    (left_rect, right_rect)
                }
            };

            // Recursively apply layout to children
            apply_bsp_recursive(conn, left, left_rect, min_window_width, min_window_height, gap)?;
            apply_bsp_recursive(conn, right, right_rect, min_window_width, min_window_height, gap)?
        }
    }
    Ok(())
}

/// Rebuild BSP tree from window list (simple approach for now)
pub fn rebuild_bsp_tree(
    bsp_tree: &mut BspTree,
    windows: &[Window],
    _focused_window: Option<Window>,
    bsp_split_ratio: f32,
) {
    #[cfg(debug_assertions)]
    tracing::debug!(
        "Rebuilding BSP tree with {} windows, focused: {:?}",
        windows.len(),
        _focused_window
    );
    *bsp_tree = BspTree::new();
    for (index, &window) in windows.iter().enumerate() {
        if index == 0 {
            // First window becomes root
            #[cfg(debug_assertions)]
            tracing::debug!("BSP: Adding first window {} as root", window);
            bsp_tree.add_window(window, None, bsp_split_ratio);
        } else {
            // For BSP, we want to split the most recently added window (not focused)
            // This creates the typical BSP behavior
            let target = Some(windows[index - 1]);
            #[cfg(debug_assertions)]
            tracing::debug!("BSP: Adding window {} targeting {:?}", window, target);
            bsp_tree.add_window(window, target, bsp_split_ratio);
        }
    }
    // Debug print the tree structure (only in debug builds)
    #[cfg(debug_assertions)]
    if let Some(ref root) = bsp_tree.root {
        tracing::debug!("BSP tree structure: {:?}", root);
    } else {
        tracing::debug!("BSP tree is empty");
    }
}
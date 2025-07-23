//! Binary Space Partitioning (BSP) layout algorithm implementation

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::types::{BspRect, SplitDirection};
use super::constants::{bsp, dimensions};

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
            split_count: bsp::INITIAL_SPLIT_COUNT,
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
                    let direction = if split_count % bsp::SPLIT_DIRECTION_MODULUS == 0 {
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
#[allow(clippy::too_many_arguments)]
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
        apply_bsp_recursive(
            conn,
            root,
            screen_rect,
            min_window_width,
            min_window_height,
            gap,
        )?;
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
                .width(rect.width.max(dimensions::MIN_WINDOW_WIDTH as i32) as u32)
                .height(rect.height.max(dimensions::MIN_WINDOW_HEIGHT as i32) as u32);
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
            apply_bsp_recursive(
                conn,
                left,
                left_rect,
                min_window_width,
                min_window_height,
                gap,
            )?;
            apply_bsp_recursive(
                conn,
                right,
                right_rect,
                min_window_width,
                min_window_height,
                gap,
            )?
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
        if index == bsp::INITIAL_SPLIT_COUNT {
            // First window becomes root
            #[cfg(debug_assertions)]
            tracing::debug!("BSP: Adding first window {} as root", window);
            bsp_tree.add_window(window, None, bsp_split_ratio);
        } else {
            // For BSP, we want to split the most recently added window (not focused)
            // This creates the typical BSP behavior
            let target = Some(windows[index - bsp::TARGET_WINDOW_OFFSET]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsp_tree_creation() {
        let bsp_tree = BspTree::new();
        assert!(bsp_tree.root.is_none());
        assert_eq!(bsp_tree.split_count, 0);
    }

    #[test]
    fn test_bsp_tree_default() {
        let bsp_tree = BspTree::default();
        assert!(bsp_tree.root.is_none());
        assert_eq!(bsp_tree.split_count, 0);
    }

    #[test]
    fn test_bsp_single_window() {
        let mut bsp_tree = BspTree::new();
        let window = 1; // Mock window ID

        bsp_tree.add_window(window, None, 0.5);

        assert!(bsp_tree.root.is_some());
        if let Some(BspNode::Leaf(w)) = &bsp_tree.root {
            assert_eq!(*w, window);
        } else {
            panic!("Root should be a leaf with the window");
        }
    }

    #[test]
    fn test_bsp_two_windows_vertical_split() {
        let mut bsp_tree = BspTree::new();
        let window1 = 1;
        let window2 = 2;

        bsp_tree.add_window(window1, None, 0.5);
        bsp_tree.add_window(window2, Some(window1), 0.5);

        // Should create a vertical split (first split)
        if let Some(BspNode::Split {
            direction,
            ratio,
            left,
            right,
        }) = &bsp_tree.root
        {
            assert!(matches!(direction, SplitDirection::Vertical));
            assert!((ratio - 0.5).abs() < f32::EPSILON);

            // Left should be window1, right should be window2
            if let (BspNode::Leaf(w1), BspNode::Leaf(w2)) = (left.as_ref(), right.as_ref()) {
                assert_eq!(*w1, window1);
                assert_eq!(*w2, window2);
            } else {
                panic!("Both children should be leaves");
            }
        } else {
            panic!("Root should be a split node");
        }
    }

    #[test]
    fn test_bsp_window_removal() {
        let mut bsp_tree = BspTree::new();
        let window1 = 1;
        let window2 = 2;

        // Add two windows
        bsp_tree.add_window(window1, None, 0.5);
        bsp_tree.add_window(window2, Some(window1), 0.5);

        // Remove window2 - should collapse back to just window1
        bsp_tree.remove_window(window2);

        if let Some(BspNode::Leaf(w)) = &bsp_tree.root {
            assert_eq!(*w, window1);
        } else {
            panic!("After removing one window, root should be a leaf with window1");
        }

        // Remove the last window - tree should be empty
        bsp_tree.remove_window(window1);
        assert!(bsp_tree.root.is_none());
    }

    #[test]
    fn test_bsp_contains_window() {
        let mut bsp_tree = BspTree::new();
        let window1 = 1;
        let window2 = 2;
        let window3 = 999; // Not in tree

        bsp_tree.add_window(window1, None, 0.5);
        bsp_tree.add_window(window2, Some(window1), 0.5);

        assert!(BspTree::contains_window_static(
            bsp_tree.root.as_ref().unwrap(),
            window1
        ));
        assert!(BspTree::contains_window_static(
            bsp_tree.root.as_ref().unwrap(),
            window2
        ));
        assert!(!BspTree::contains_window_static(
            bsp_tree.root.as_ref().unwrap(),
            window3
        ));
    }

    #[test]
    fn test_bsp_split_direction_alternation() {
        let mut bsp_tree = BspTree::new();

        // Test that splits alternate V→H→V→H
        bsp_tree.add_window(1, None, 0.5); // Root
        bsp_tree.add_window(2, Some(1), 0.5); // Split 0 (even) = Vertical

        if let Some(BspNode::Split { direction, .. }) = &bsp_tree.root {
            assert!(matches!(direction, SplitDirection::Vertical));
        }

        bsp_tree.add_window(3, Some(2), 0.5); // Split 1 (odd) = Horizontal

        // Navigate to the right child which should be horizontal
        if let Some(BspNode::Split { right, .. }) = &bsp_tree.root {
            if let BspNode::Split { direction, .. } = right.as_ref() {
                assert!(matches!(direction, SplitDirection::Horizontal));
            }
        }
    }

    #[test]
    fn test_bsp_empty_window_list_rebuild() {
        let mut bsp_tree = BspTree::new();

        // Test rebuild with empty window list
        rebuild_bsp_tree(&mut bsp_tree, &[], None, 0.5);
        assert!(bsp_tree.root.is_none());
    }

    #[test]
    fn test_bsp_single_window_rebuild() {
        let mut bsp_tree = BspTree::new();

        // Test rebuild with single window
        rebuild_bsp_tree(&mut bsp_tree, &[42], None, 0.5);

        if let Some(BspNode::Leaf(window)) = &bsp_tree.root {
            assert_eq!(*window, 42);
        } else {
            panic!("Single window should create a leaf node");
        }
    }
}

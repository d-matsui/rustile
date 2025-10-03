//! Binary Space Partitioning (BSP) tree data structure for window management

use tracing::info;
use x11rb::protocol::xproto::Window;

// === Constants ===

/// Minimum dimensions for ensuring windows remain usable
pub mod dimensions {
    /// Minimum window width to ensure usability (pixels)
    pub const MIN_WINDOW_WIDTH: u32 = 50;

    /// Minimum window height to ensure usability (pixels)
    pub const MIN_WINDOW_HEIGHT: u32 = 50;
}

/// BSP tree configuration
pub mod bsp_constants {
    /// Initial split count for new BSP trees
    pub const INITIAL_SPLIT_COUNT: usize = 0;

    /// Modulus for alternating split directions (even=vertical, odd=horizontal)
    pub const SPLIT_DIRECTION_MODULUS: usize = 2;
}

// === Types ===

/// Represents a split direction in BSP layout
#[derive(Debug, Clone, Copy)]
pub enum SplitDirection {
    /// Horizontal arrangement: windows placed left-to-right
    Horizontal,
    /// Vertical arrangement: windows placed top-to-bottom  
    Vertical,
}

impl SplitDirection {
    /// Returns the opposite split direction
    pub fn opposite(self) -> Self {
        match self {
            SplitDirection::Horizontal => SplitDirection::Vertical,
            SplitDirection::Vertical => SplitDirection::Horizontal,
        }
    }
}

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
            split_count: bsp_constants::INITIAL_SPLIT_COUNT,
        }
    }

    /// Returns all windows in the tree in left-to-right, depth-first order
    pub fn all_windows(&self) -> Vec<Window> {
        let mut windows = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_windows_ordered(root, &mut windows);
        }
        windows
    }

    /// Helper to collect windows in order (left-to-right, depth-first)
    fn collect_windows_ordered(node: &BspNode, windows: &mut Vec<Window>) {
        match node {
            BspNode::Leaf(window) => {
                windows.push(*window);
            }
            BspNode::Split { left, right, .. } => {
                Self::collect_windows_ordered(left, windows);
                Self::collect_windows_ordered(right, windows);
            }
        }
    }

    /// Returns the total number of windows in the tree
    pub fn window_count(&self) -> usize {
        match &self.root {
            Some(root) => Self::count_windows(root),
            None => 0,
        }
    }

    /// Helper to count windows recursively
    fn count_windows(node: &BspNode) -> usize {
        match node {
            BspNode::Leaf(_) => 1,
            BspNode::Split { left, right, .. } => {
                Self::count_windows(left) + Self::count_windows(right)
            }
        }
    }

    /// Checks if the tree contains a specific window
    pub fn has_window(&self, target_window: Window) -> bool {
        match &self.root {
            Some(root) => Self::contains_window(root, target_window),
            None => false,
        }
    }

    /// Returns the next window in order after the given window (wraps around)
    pub fn next_window(&self, current: Window) -> Option<Window> {
        let windows = self.all_windows();
        if windows.is_empty() {
            return None;
        }

        if let Some(pos) = windows.iter().position(|&w| w == current) {
            let next_pos = (pos + 1) % windows.len();
            Some(windows[next_pos])
        } else {
            // If current window not found, return first window
            Some(windows[0])
        }
    }

    /// Returns the previous window in order before the given window (wraps around)
    pub fn prev_window(&self, current: Window) -> Option<Window> {
        let windows = self.all_windows();
        if windows.is_empty() {
            return None;
        }

        if let Some(pos) = windows.iter().position(|&w| w == current) {
            let prev_pos = if pos == 0 { windows.len() - 1 } else { pos - 1 };
            Some(windows[prev_pos])
        } else {
            // If current window not found, return first window
            Some(windows[0])
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
                    let direction = if split_count % bsp_constants::SPLIT_DIRECTION_MODULUS == 0 {
                        SplitDirection::Horizontal
                    } else {
                        SplitDirection::Vertical
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

    /// Rotates the parent split direction of the specified window
    pub fn rotate_window(&mut self, window: Window) -> bool {
        match &mut self.root {
            Some(root) => {
                info!("Attempting to rotate window {:?} in BSP tree", window);
                Self::rotate_window_recursive(root, window)
            }
            None => {
                info!("Cannot rotate: BSP tree is empty");
                false
            }
        }
    }

    /// Recursively finds and rotates the parent split of the target window
    fn rotate_window_recursive(node: &mut BspNode, target_window: Window) -> bool {
        match node {
            BspNode::Leaf(window) => {
                // Found the target window as a leaf
                let found = *window == target_window;
                if found {
                    info!(
                        "Found target window {:?} as a leaf node (cannot rotate leaf)",
                        target_window
                    );
                }
                found
            }
            BspNode::Split {
                direction,
                left,
                right,
                ..
            } => {
                // Check if target window is in left subtree
                if Self::contains_window(left, target_window) {
                    // If left child is the target window (direct child), rotate this split
                    if let BspNode::Leaf(window) = left.as_ref()
                        && *window == target_window
                    {
                        // Flip this split's direction
                        let old_direction = *direction;
                        *direction = direction.opposite();
                        info!(
                            "Rotated parent split from {:?} to {:?} for window {:?}",
                            old_direction, direction, target_window
                        );
                        return true;
                    }
                    // Otherwise, recurse into left subtree
                    return Self::rotate_window_recursive(left, target_window);
                }

                // Check if target window is in right subtree
                if Self::contains_window(right, target_window) {
                    // If right child is the target window (direct child), rotate this split
                    if let BspNode::Leaf(window) = right.as_ref()
                        && *window == target_window
                    {
                        // Flip this split's direction
                        let old_direction = *direction;
                        *direction = direction.opposite();
                        info!(
                            "Rotated parent split from {:?} to {:?} for window {:?}",
                            old_direction, direction, target_window
                        );
                        return true;
                    }
                    // Otherwise, recurse into right subtree
                    return Self::rotate_window_recursive(right, target_window);
                }

                false
            }
        }
    }

    /// Helper function to check if a subtree contains a specific window
    fn contains_window(node: &BspNode, target_window: Window) -> bool {
        match node {
            BspNode::Leaf(window) => *window == target_window,
            BspNode::Split { left, right, .. } => {
                Self::contains_window(left, target_window)
                    || Self::contains_window(right, target_window)
            }
        }
    }

    /// Swaps two windows in the BSP tree while preserving the tree structure
    pub fn swap_windows(&mut self, window1: Window, window2: Window) -> bool {
        if let Some(root) = &mut self.root {
            Self::swap_windows_recursive(root, window1, window2)
        } else {
            false
        }
    }

    /// Recursively swaps two windows in the tree
    fn swap_windows_recursive(node: &mut BspNode, window1: Window, window2: Window) -> bool {
        match node {
            BspNode::Leaf(window) => {
                if *window == window1 {
                    *window = window2;
                    true
                } else if *window == window2 {
                    *window = window1;
                    true
                } else {
                    false
                }
            }
            BspNode::Split { left, right, .. } => {
                let swapped_left = Self::swap_windows_recursive(left, window1, window2);
                let swapped_right = Self::swap_windows_recursive(right, window1, window2);
                swapped_left || swapped_right
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

    /// Find the bounds of the parent split containing the target window
    pub fn find_parent_bounds(
        &self,
        target_window: Window,
        screen_rect: crate::window_renderer::BspRect,
    ) -> Option<crate::window_renderer::BspRect> {
        if let Some(ref root) = self.root {
            Self::find_parent_bounds_recursive(root, target_window, screen_rect)
        } else {
            None
        }
    }

    /// Recursively find parent bounds for a window
    fn find_parent_bounds_recursive(
        node: &BspNode,
        target_window: Window,
        rect: crate::window_renderer::BspRect,
    ) -> Option<crate::window_renderer::BspRect> {
        match node {
            BspNode::Leaf(window) => {
                if *window == target_window {
                    // Found the window - if it's the root, it has no parent
                    None
                } else {
                    None
                }
            }
            BspNode::Split {
                direction,
                ratio,
                left,
                right,
            } => {
                // Check if either child directly contains the target window
                let left_is_target = matches!(**left, BspNode::Leaf(w) if w == target_window);
                let right_is_target = matches!(**right, BspNode::Leaf(w) if w == target_window);

                if left_is_target || right_is_target {
                    // This split is the parent of the target window
                    return Some(rect);
                }

                // Calculate child rectangles and recurse
                let (left_rect, right_rect) = match direction {
                    SplitDirection::Horizontal => {
                        let split_x = rect.x + (rect.width as f32 * ratio) as i32;
                        (
                            crate::window_renderer::BspRect {
                                x: rect.x,
                                y: rect.y,
                                width: split_x - rect.x,
                                height: rect.height,
                            },
                            crate::window_renderer::BspRect {
                                x: split_x,
                                y: rect.y,
                                width: rect.x + rect.width - split_x,
                                height: rect.height,
                            },
                        )
                    }
                    SplitDirection::Vertical => {
                        let split_y = rect.y + (rect.height as f32 * ratio) as i32;
                        (
                            crate::window_renderer::BspRect {
                                x: rect.x,
                                y: rect.y,
                                width: rect.width,
                                height: split_y - rect.y,
                            },
                            crate::window_renderer::BspRect {
                                x: rect.x,
                                y: split_y,
                                width: rect.width,
                                height: rect.y + rect.height - split_y,
                            },
                        )
                    }
                };

                // Try to find in left subtree
                if Self::contains_window_static(left, target_window) {
                    Self::find_parent_bounds_recursive(left, target_window, left_rect)
                } else if Self::contains_window_static(right, target_window) {
                    Self::find_parent_bounds_recursive(right, target_window, right_rect)
                } else {
                    None
                }
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

    /// Balance the BSP tree by calculating optimal split ratios based on window count
    ///
    /// This method traverses the tree and updates each split node's ratio to be proportional
    /// to the number of windows in the left and right subtrees, ensuring all windows receive
    /// equal screen area.
    ///
    /// # Example
    /// ```ignore
    /// let mut tree = BspTree::new();
    /// tree.add_window(1, None, 0.5);
    /// tree.add_window(2, Some(1), 0.3); // Unbalanced
    /// tree.balance_tree(); // Now balanced with ratio ~0.5
    /// ```
    pub fn balance_tree(&mut self) {
        if let Some(root) = &mut self.root {
            // Only balance if we have more than one window (i.e., at least one split)
            if !matches!(root, BspNode::Leaf(_)) {
                Self::balance_tree_recursive(root);
            }
        }
    }

    /// Recursively balance a subtree and return the number of windows it contains
    ///
    /// This performs a bottom-up traversal, calculating window counts and updating
    /// split ratios in a single pass.
    ///
    /// # Returns
    /// The number of windows in this subtree
    fn balance_tree_recursive(node: &mut BspNode) -> usize {
        match node {
            BspNode::Leaf(_) => {
                // Leaf nodes contain exactly one window
                1
            }
            BspNode::Split {
                ratio,
                left,
                right,
                ..
            } => {
                // Recursively balance and count windows in both subtrees
                let left_count = Self::balance_tree_recursive(left);
                let right_count = Self::balance_tree_recursive(right);
                let total_count = left_count + right_count;

                // Calculate optimal ratio: left_count / total_count
                // This ensures windows get area proportional to their count
                *ratio = left_count as f32 / total_count as f32;

                total_count
            }
        }
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
            assert!(matches!(direction, SplitDirection::Horizontal));
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

        // Test that splits alternate H→V→H→V
        bsp_tree.add_window(1, None, 0.5); // Root
        bsp_tree.add_window(2, Some(1), 0.5); // Split 0 (even) = Horizontal

        if let Some(BspNode::Split { direction, .. }) = &bsp_tree.root {
            assert!(matches!(direction, SplitDirection::Horizontal));
        }

        bsp_tree.add_window(3, Some(2), 0.5); // Split 1 (odd) = Vertical

        // Navigate to the right child which should be vertical
        if let Some(BspNode::Split { right, .. }) = &bsp_tree.root
            && let BspNode::Split { direction, .. } = right.as_ref()
        {
            assert!(matches!(direction, SplitDirection::Vertical));
        }
    }

    #[test]
    fn test_bsp_tree_api_methods() {
        let mut bsp_tree = BspTree::new();

        // Test empty tree
        assert_eq!(bsp_tree.window_count(), 0);
        assert_eq!(bsp_tree.all_windows(), Vec::<Window>::new());
        assert!(!bsp_tree.has_window(1));
        assert_eq!(bsp_tree.next_window(1), None);
        assert_eq!(bsp_tree.prev_window(1), None);

        // Add first window
        bsp_tree.add_window(10, None, 0.5);
        assert_eq!(bsp_tree.window_count(), 1);
        assert_eq!(bsp_tree.all_windows(), vec![10]);
        assert!(bsp_tree.has_window(10));
        assert!(!bsp_tree.has_window(20));

        // Single window navigation should return self
        assert_eq!(bsp_tree.next_window(10), Some(10));
        assert_eq!(bsp_tree.prev_window(10), Some(10));

        // Add second window
        bsp_tree.add_window(20, Some(10), 0.5);
        assert_eq!(bsp_tree.window_count(), 2);
        assert_eq!(bsp_tree.all_windows(), vec![10, 20]);
        assert!(bsp_tree.has_window(10));
        assert!(bsp_tree.has_window(20));

        // Two window navigation should wrap around
        assert_eq!(bsp_tree.next_window(10), Some(20));
        assert_eq!(bsp_tree.next_window(20), Some(10));
        assert_eq!(bsp_tree.prev_window(10), Some(20));
        assert_eq!(bsp_tree.prev_window(20), Some(10));

        // Add third window
        bsp_tree.add_window(30, Some(20), 0.5);
        assert_eq!(bsp_tree.window_count(), 3);
        assert_eq!(bsp_tree.all_windows(), vec![10, 20, 30]);

        // Three window navigation
        assert_eq!(bsp_tree.next_window(10), Some(20));
        assert_eq!(bsp_tree.next_window(20), Some(30));
        assert_eq!(bsp_tree.next_window(30), Some(10)); // Wrap to first

        assert_eq!(bsp_tree.prev_window(10), Some(30)); // Wrap to last
        assert_eq!(bsp_tree.prev_window(20), Some(10));
        assert_eq!(bsp_tree.prev_window(30), Some(20));

        // Non-existent window should return first window
        assert_eq!(bsp_tree.next_window(999), Some(10));
        assert_eq!(bsp_tree.prev_window(999), Some(10));
    }

    #[test]
    fn test_find_parent_bounds() {
        use crate::window_renderer::BspRect;

        let mut bsp_tree = BspTree::new();
        let screen_rect = BspRect {
            x: 0,
            y: 0,
            width: 1000,
            height: 800,
        };

        // Single window - should have no parent
        bsp_tree.add_window(10, None, 0.5);
        assert_eq!(bsp_tree.find_parent_bounds(10, screen_rect), None);

        // Two windows - both should return the root split bounds
        bsp_tree.add_window(20, Some(10), 0.5);
        assert_eq!(
            bsp_tree.find_parent_bounds(10, screen_rect),
            Some(screen_rect)
        );
        assert_eq!(
            bsp_tree.find_parent_bounds(20, screen_rect),
            Some(screen_rect)
        );

        // Three windows - test nested split
        // Tree structure:
        //     Split(H)
        //    /        \
        //   10      Split(V)
        //          /        \
        //         20        30
        bsp_tree.add_window(30, Some(20), 0.5);

        // Window 10's parent is still the root
        assert_eq!(
            bsp_tree.find_parent_bounds(10, screen_rect),
            Some(screen_rect)
        );

        // Windows 20 and 30's parent should be the right half (vertical split)
        let right_half = BspRect {
            x: 500,
            y: 0,
            width: 500,
            height: 800,
        };
        assert_eq!(
            bsp_tree.find_parent_bounds(20, screen_rect),
            Some(right_half)
        );
        assert_eq!(
            bsp_tree.find_parent_bounds(30, screen_rect),
            Some(right_half)
        );

        // Non-existent window should return None
        assert_eq!(bsp_tree.find_parent_bounds(999, screen_rect), None);
    }

    // === Balance Tests ===

    #[test]
    fn test_balance_empty_tree() {
        let mut bsp_tree = BspTree::new();

        // Balance on empty tree should be no-op
        bsp_tree.balance_tree();

        assert!(bsp_tree.root.is_none());
    }

    #[test]
    fn test_balance_single_window() {
        let mut bsp_tree = BspTree::new();
        bsp_tree.add_window(1, None, 0.5);

        // Balance on single window should be no-op
        bsp_tree.balance_tree();

        // Should still be a single leaf
        assert!(matches!(bsp_tree.root, Some(BspNode::Leaf(1))));
    }

    #[test]
    fn test_balance_two_windows() {
        let mut bsp_tree = BspTree::new();
        bsp_tree.add_window(1, None, 0.5);
        bsp_tree.add_window(2, Some(1), 0.3); // Unbalanced ratio

        bsp_tree.balance_tree();

        // Should have ratio = 0.5 (1 window on left, 1 on right)
        if let Some(BspNode::Split { ratio, .. }) = &bsp_tree.root {
            assert!((ratio - 0.5).abs() < 0.01);
        } else {
            panic!("Root should be a split node");
        }
    }

    #[test]
    fn test_balance_three_windows_vertical() {
        let mut bsp_tree = BspTree::new();
        bsp_tree.add_window(1, None, 0.5);
        bsp_tree.add_window(2, Some(1), 0.5);
        bsp_tree.add_window(3, Some(2), 0.5);

        // Tree: Split(H) { left: Leaf(1), right: Split(V) { left: Leaf(2), right: Leaf(3) } }
        // After balance: root ratio should be 1/(1+2) = 0.33, right ratio should be 1/(1+1) = 0.5

        bsp_tree.balance_tree();

        if let Some(BspNode::Split { ratio, right, .. }) = &bsp_tree.root {
            // Root ratio: 1 window on left, 2 on right = 1/3 ≈ 0.33
            assert!((ratio - 0.33).abs() < 0.01, "Root ratio should be ~0.33, got {}", ratio);

            // Right subtree should have equal ratio (1 window each)
            if let BspNode::Split { ratio: right_ratio, .. } = right.as_ref() {
                assert!((right_ratio - 0.5).abs() < 0.01, "Right ratio should be ~0.5, got {}", right_ratio);
            }
        } else {
            panic!("Root should be a split node");
        }
    }

    #[test]
    fn test_balance_three_windows_horizontal() {
        let mut bsp_tree = BspTree::new();
        // Create horizontal split at root by adding windows in specific order
        bsp_tree.add_window(1, None, 0.5);
        bsp_tree.add_window(2, Some(1), 0.5); // Creates horizontal split (split_count=0)
        bsp_tree.add_window(3, Some(1), 0.5); // Splits left side vertically (split_count=1)

        // Tree: Split(H) { left: Split(V) { left: Leaf(3), right: Leaf(1) }, right: Leaf(2) }
        // After balance: root ratio should be 2/(2+1) = 0.67, left ratio should be 0.5

        bsp_tree.balance_tree();

        if let Some(BspNode::Split { ratio, .. }) = &bsp_tree.root {
            // Root ratio: 2 windows on left, 1 on right = 2/3 ≈ 0.67
            assert!((ratio - 0.67).abs() < 0.01, "Root ratio should be ~0.67, got {}", ratio);
        } else {
            panic!("Root should be a split node");
        }
    }

    #[test]
    fn test_balance_preserves_structure() {
        let mut bsp_tree = BspTree::new();
        bsp_tree.add_window(1, None, 0.5);
        bsp_tree.add_window(2, Some(1), 0.5);
        bsp_tree.add_window(3, Some(2), 0.5);

        let windows_before = bsp_tree.all_windows();
        let count_before = bsp_tree.window_count();

        bsp_tree.balance_tree();

        let windows_after = bsp_tree.all_windows();
        let count_after = bsp_tree.window_count();

        // Window count and order should be preserved
        assert_eq!(count_before, count_after);
        assert_eq!(windows_before, windows_after);
    }
}

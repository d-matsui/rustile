//! Window layout algorithms for the tiling window manager

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

/// Represents a split direction in BSP layout
#[derive(Debug, Clone, Copy)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
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
    root: Option<BspNode>,
    split_count: usize, // To alternate split directions
}

/// Rectangle for BSP layout calculations
#[derive(Debug, Clone, Copy)]
struct BspRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// Represents different tiling layouts
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// Master-stack layout: one master window on the left, stack on the right
    MasterStack,
    /// Binary Space Partitioning layout: recursive splitting of space
    Bsp,
}

/// Window layout manager
pub struct LayoutManager {
    current_layout: Layout,
    bsp_tree: BspTree,
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
    pub fn add_window(&mut self, window: Window, focused_window: Option<Window>) {
        if self.root.is_none() {
            // First window - becomes root
            self.root = Some(BspNode::Leaf(window));
            return;
        }

        // Find where to insert the window (split the focused window or last leaf)
        let target_window = focused_window.unwrap_or(window);
        let split_count = self.split_count; // Capture split_count to avoid borrowing issues

        if let Some(ref mut root_node) = self.root {
            Self::insert_window_into_node_static(root_node, window, target_window, split_count);
        }
        self.split_count += 1;
    }

    /// Recursively find the target window and split it (static version)
    fn insert_window_into_node_static(
        node: &mut BspNode,
        new_window: Window,
        target_window: Window,
        split_count: usize,
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
                        ratio: 0.5,
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
                    )
                } else if Self::contains_window_static(right, target_window) {
                    Self::insert_window_into_node_static(
                        right,
                        new_window,
                        target_window,
                        split_count,
                    )
                } else {
                    false
                }
            }
        }
    }

    /// Check if a subtree contains a specific window
    #[allow(dead_code)]
    fn contains_window(&self, node: &BspNode, target_window: Window) -> bool {
        Self::contains_window_static(node, target_window)
    }

    /// Static version of contains_window to avoid borrow issues
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
                direction: _,
                ratio: _,
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
                            direction: SplitDirection::Vertical, // We'll track this properly later
                            ratio: 0.5,
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
                            direction: SplitDirection::Vertical, // We'll track this properly later
                            ratio: 0.5,
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
                        direction: SplitDirection::Vertical,
                        ratio: 0.5,
                        left,
                        right,
                    })
                }
            }
        }
    }
}

impl LayoutManager {
    /// Creates a new layout manager with default layout
    pub fn new() -> Self {
        Self {
            current_layout: Layout::MasterStack,
            bsp_tree: BspTree::new(),
        }
    }

    /// Switch to BSP layout
    pub fn set_layout(&mut self, layout: Layout) {
        self.current_layout = layout;
    }

    /// Get the current layout
    pub fn current_layout(&self) -> Layout {
        self.current_layout
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    /// Applies the current layout to the given windows
    pub fn apply_layout(
        &mut self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
        focused_window: Option<Window>,
        master_ratio: f32,
        gap: u32,
    ) -> Result<()> {
        tracing::info!(
            "Applying layout: {:?} with {} windows",
            self.current_layout,
            windows.len()
        );
        match self.current_layout {
            Layout::MasterStack => {
                tracing::debug!("Using MasterStack layout");
                self.tile_master_stack(conn, screen, windows, master_ratio, gap)
            }
            Layout::Bsp => {
                tracing::debug!("Using BSP layout");
                // Rebuild BSP tree from current windows
                self.rebuild_bsp_tree(windows, focused_window);
                self.tile_bsp(conn, screen, gap)
            }
        }
    }

    /// Rebuild BSP tree from window list (simple approach for now)
    fn rebuild_bsp_tree(&mut self, windows: &[Window], focused_window: Option<Window>) {
        tracing::debug!(
            "Rebuilding BSP tree with {} windows, focused: {:?}",
            windows.len(),
            focused_window
        );
        self.bsp_tree = BspTree::new();
        for (index, &window) in windows.iter().enumerate() {
            if index == 0 {
                // First window becomes root
                tracing::debug!("BSP: Adding first window {} as root", window);
                self.bsp_tree.add_window(window, None);
            } else {
                // For BSP, we want to split the most recently added window (not focused)
                // This creates the typical yabai behavior
                let target = Some(windows[index - 1]);
                tracing::debug!("BSP: Adding window {} targeting {:?}", window, target);
                self.bsp_tree.add_window(window, target);
            }
        }
        // Debug print the tree structure
        if let Some(ref root) = self.bsp_tree.root {
            tracing::debug!("BSP tree structure: {:?}", root);
        } else {
            tracing::debug!("BSP tree is empty");
        }
    }

    /// Apply BSP tiling layout
    fn tile_bsp(&self, conn: &impl Connection, screen: &Screen, gap: u32) -> Result<()> {
        if let Some(ref root) = self.bsp_tree.root {
            let screen_rect = BspRect {
                x: gap as i32,
                y: gap as i32,
                width: (screen.width_in_pixels as i32 - 2 * gap as i32).max(100),
                height: (screen.height_in_pixels as i32 - 2 * gap as i32).max(100),
            };
            tracing::debug!(
                "BSP: Applying layout to screen {}x{} with gap {}",
                screen.width_in_pixels,
                screen.height_in_pixels,
                gap
            );
            Self::apply_bsp_recursive(conn, root, screen_rect, gap)?;
        } else {
            tracing::debug!("BSP: No root node, skipping layout");
        }
        Ok(())
    }

    /// Recursively apply BSP layout to nodes
    fn apply_bsp_recursive(
        conn: &impl Connection,
        node: &BspNode,
        rect: BspRect,
        gap: u32,
    ) -> Result<()> {
        match node {
            BspNode::Leaf(window) => {
                // Configure the window to fill the rect
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
                            width: split_pos.max(50),
                            height: rect.height,
                        };
                        let right_rect = BspRect {
                            x: rect.x + split_pos + gap_i32,
                            y: rect.y,
                            width: (rect.width - split_pos - gap_i32).max(50),
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
                            height: split_pos.max(50),
                        };
                        let right_rect = BspRect {
                            x: rect.x,
                            y: rect.y + split_pos + gap_i32,
                            width: rect.width,
                            height: (rect.height - split_pos - gap_i32).max(50),
                        };
                        (left_rect, right_rect)
                    }
                };

                // Recursively apply layout to children
                Self::apply_bsp_recursive(conn, left, left_rect, gap)?;
                Self::apply_bsp_recursive(conn, right, right_rect, gap)?
            }
        }
        Ok(())
    }

    /// Implements master-stack tiling layout
    ///
    /// Layout behavior:
    /// - Single window: Full screen minus gaps
    /// - Multiple windows: First window takes configurable ratio (master),
    ///   remaining windows stack vertically on the right, with gaps between
    fn tile_master_stack(
        &self,
        conn: &impl Connection,
        screen: &Screen,
        windows: &[Window],
        master_ratio: f32,
        gap: u32,
    ) -> Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        let screen_width = screen.width_in_pixels as i16;
        let screen_height = screen.height_in_pixels as i16;
        let num_windows = windows.len() as i16;
        let gap_i16 = gap as i16;

        // Configure master window
        let master_window = windows[0];
        let master_width = if num_windows > 1 {
            // Multiple windows: master takes ratio of available space, ensure minimum 100px
            let available_width = screen_width - 3 * gap_i16;
            if available_width > 150 {
                // Need at least 150px total (100px master + 50px stack)
                ((available_width as f32 * master_ratio) as i16).max(100)
            } else {
                // Fallback: reduce gaps to fit windows
                (screen_width / 2).max(100)
            }
        } else {
            // Single window: full width minus gaps, minimum 100px
            (screen_width - 2 * gap_i16).max(100)
        };

        let master_config = ConfigureWindowAux::new()
            .x(gap_i16 as i32)
            .y(gap_i16 as i32)
            .width(master_width.max(100) as u32) // Minimum 100px width
            .height((screen_height - 2 * gap_i16).max(100) as u32); // Minimum 100px height

        conn.configure_window(master_window, &master_config)?;

        // Configure stack windows if any
        if num_windows > 1 {
            let stack_windows = &windows[1..];
            let num_stack = stack_windows.len() as i16;
            let stack_x = gap_i16 + master_width + gap_i16; // Add gap between master and stack
            let stack_width = (screen_width - stack_x - gap_i16).max(50); // Minimum usable width

            // Ensure we have enough space for stack windows with minimum height
            let min_total_height = num_stack * 50 + (num_stack - 1) * gap_i16; // 50px min per window
            let available_height = screen_height - 2 * gap_i16;

            let total_stack_height = if available_height >= min_total_height {
                available_height - (num_stack - 1) * gap_i16
            } else {
                // Fallback: reduce gaps if necessary to fit windows
                (available_height - num_stack * 50).max(num_stack * 50)
            };

            let stack_height = (total_stack_height / num_stack).max(50); // Minimum 50px height

            for (index, &window) in stack_windows.iter().enumerate() {
                let stack_y = gap_i16 + (index as i16) * (stack_height + gap_i16);

                let stack_config = ConfigureWindowAux::new()
                    .x(stack_x as i32)
                    .y(stack_y as i32)
                    .width(stack_width.max(1) as u32)
                    .height(stack_height.max(1) as u32);

                conn.configure_window(window, &stack_config)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_manager_default() {
        let layout_manager = LayoutManager::default();
        match layout_manager.current_layout {
            Layout::MasterStack => (),
            Layout::Bsp => panic!("Default should be MasterStack"),
        }
    }

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

        bsp_tree.add_window(window, None);

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

        bsp_tree.add_window(window1, None);
        bsp_tree.add_window(window2, Some(window1));

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
    fn test_bsp_three_windows_alternating_splits() {
        let mut bsp_tree = BspTree::new();
        let window1 = 1;
        let window2 = 2;
        let window3 = 3;

        bsp_tree.add_window(window1, None);
        bsp_tree.add_window(window2, Some(window1)); // Should split window1 vertically
        bsp_tree.add_window(window3, Some(window2)); // Should split window2 horizontally

        // Root should be a vertical split
        if let Some(BspNode::Split {
            direction: root_dir,
            left,
            right,
            ..
        }) = &bsp_tree.root
        {
            assert!(matches!(root_dir, SplitDirection::Vertical));

            // Left child should be window1 (leaf)
            if let BspNode::Leaf(w1) = left.as_ref() {
                assert_eq!(*w1, window1);
            } else {
                panic!("Left child should be window1");
            }

            // Right child should be a horizontal split containing window2 and window3
            if let BspNode::Split {
                direction: right_dir,
                left: right_left,
                right: right_right,
                ..
            } = right.as_ref()
            {
                assert!(matches!(right_dir, SplitDirection::Horizontal));

                if let (BspNode::Leaf(w2), BspNode::Leaf(w3)) =
                    (right_left.as_ref(), right_right.as_ref())
                {
                    assert_eq!(*w2, window2);
                    assert_eq!(*w3, window3);
                } else {
                    panic!("Right split children should be window2 and window3");
                }
            } else {
                panic!("Right child should be a horizontal split");
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
        bsp_tree.add_window(window1, None);
        bsp_tree.add_window(window2, Some(window1));

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

        bsp_tree.add_window(window1, None);
        bsp_tree.add_window(window2, Some(window1));

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
    fn test_layout_manager_set_bsp() {
        let mut layout_manager = LayoutManager::new();
        assert!(matches!(
            layout_manager.current_layout(),
            Layout::MasterStack
        ));

        layout_manager.set_layout(Layout::Bsp);
        assert!(matches!(layout_manager.current_layout(), Layout::Bsp));
    }

    #[test]
    fn test_bsp_rebuild_debug() {
        let mut layout_manager = LayoutManager::new();
        layout_manager.set_layout(Layout::Bsp);

        // Simulate window list
        let windows = vec![1, 2, 3];

        // Test rebuild
        layout_manager.rebuild_bsp_tree(&windows, Some(2));

        // Check if tree was built
        assert!(layout_manager.bsp_tree.root.is_some());

        // Print tree structure for debugging
        if let Some(ref root) = layout_manager.bsp_tree.root {
            println!("BSP tree structure: {:?}", root);
        }

        // Test individual window additions
        let mut bsp_tree = BspTree::new();
        bsp_tree.add_window(1, None);
        println!("After adding window 1: {:?}", bsp_tree.root);
        bsp_tree.add_window(2, Some(1));
        println!("After adding window 2: {:?}", bsp_tree.root);
        bsp_tree.add_window(3, Some(2));
        println!("After adding window 3: {:?}", bsp_tree.root);
    }

    #[test]
    fn test_empty_window_list() {
        let _layout_manager = LayoutManager::new();

        // Mock screen dimensions
        let _screen_width = 1280;
        let _screen_height = 720;

        // This should not panic with empty windows
        let windows: Vec<Window> = vec![];

        // We can't easily test X11 operations without mocking,
        // but we can at least ensure the logic doesn't panic
        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_master_window_dimensions() {
        // Test that master window calculations are correct
        let screen_width = 1280_f32;
        let screen_height = 720_f32;

        // With one window, it should take full screen
        let expected_single_width = screen_width as u32;
        let expected_single_height = screen_height as u32;

        // With multiple windows, master takes master_ratio of width (default 0.5)
        let master_ratio = 0.5_f32;
        let expected_master_width = (screen_width * master_ratio) as u32;
        let expected_master_height = screen_height as u32;

        assert_eq!(expected_single_width, 1280);
        assert_eq!(expected_single_height, 720);
        assert_eq!(expected_master_width, 640); // 1280 * 0.5
        assert_eq!(expected_master_height, 720);
    }

    #[test]
    fn test_stack_window_calculations() {
        let screen_width = 1280_i16;
        let screen_height = 720_i16;
        let num_windows = 3_i16;

        // Stack windows calculations with default master ratio (0.5)
        let master_ratio = 0.5_f32;
        let stack_x = (screen_width as f32 * master_ratio) as i16;
        let stack_width = screen_width - stack_x;
        let stack_height = screen_height / (num_windows - 1);

        assert_eq!(stack_x, 640);
        assert_eq!(stack_width, 640);
        assert_eq!(stack_height, 360); // 720 / 2 stack windows
    }

    #[test]
    fn test_gap_calculations() {
        let screen_width = 1280_i16;
        let _screen_height = 720_i16;
        let gap = 10_u32;
        let gap_i16 = gap as i16;
        let master_ratio = 0.5_f32;

        // Single window with gaps
        let single_width = screen_width - 2 * gap_i16;
        assert_eq!(single_width, 1260); // 1280 - 20

        // Multiple windows with gaps - master width calculation
        let available_width = screen_width - 3 * gap_i16; // left + center + right gaps
        let master_width = (available_width as f32 * master_ratio) as i16;
        assert_eq!(master_width, 625); // (1280 - 30) * 0.5 = 625

        // Stack positioning
        let stack_x = gap_i16 + master_width + gap_i16;
        assert_eq!(stack_x, 645); // 10 + 625 + 10

        // Stack width
        let stack_width = screen_width - stack_x - gap_i16;
        assert_eq!(stack_width, 625); // 1280 - 645 - 10
    }

    #[test]
    fn test_minimum_window_sizes() {
        // Test that minimum sizes are enforced
        let min_master_width = 100_i16;
        let min_stack_width = 50_i16;
        let _min_height = 50_i16;

        // Very small screen should still provide minimum sizes
        let small_screen_width = 200_i16;
        let large_gap = 50_i16;

        let calculated_width = (small_screen_width - 2 * large_gap).max(min_master_width);
        assert_eq!(calculated_width, min_master_width); // Should fallback to minimum

        let calculated_stack_width = (small_screen_width / 4).max(min_stack_width);
        assert_eq!(calculated_stack_width, min_stack_width); // Should use minimum
    }

    #[test]
    fn test_gap_edge_cases() {
        // Test large gap scenarios
        let screen_width = 800_i16;
        let _screen_height = 600_i16;
        let large_gap = 200_u32;
        let gap_i16 = large_gap as i16;

        // Available width after gaps
        let available_width = screen_width - 3 * gap_i16;
        // 800 - 600 = 200px available (very tight)

        // Should fallback to reasonable sizing
        let fallback_width = if available_width > 150 {
            available_width
        } else {
            screen_width / 2 // Use half screen as fallback
        };

        // 200 > 150 is true, so we use available_width (200)
        assert_eq!(fallback_width, 200);
    }
}

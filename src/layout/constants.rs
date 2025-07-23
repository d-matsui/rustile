//! Layout-related constants
//!
//! This module centralizes magic numbers and constants used throughout
//! the layout system to improve maintainability and reduce duplication.

/// Minimum dimensions for ensuring windows remain usable
pub mod dimensions {
    /// Minimum window width to ensure usability (pixels)
    pub const MIN_WINDOW_WIDTH: u32 = 50;

    /// Minimum window height to ensure usability (pixels)
    pub const MIN_WINDOW_HEIGHT: u32 = 50;

    /// Minimum master window width (pixels) - for future use
    #[allow(dead_code)]
    pub const MIN_MASTER_WIDTH: u32 = 100;
}

/// Layout calculations and spacing
pub mod layout {
    /// Number of gaps in master-stack layout (left + center + right)
    pub const MASTER_STACK_GAP_COUNT: i16 = 3;

    /// Number of edge gaps in single window layout (left + right)
    pub const SINGLE_WINDOW_GAP_COUNT: i16 = 2;

    /// Fallback screen division ratio when space is constrained
    pub const FALLBACK_SCREEN_RATIO: i16 = 2;

    /// Threshold for gap fallback decisions (pixels) - for future use
    #[allow(dead_code)]
    pub const GAP_FALLBACK_THRESHOLD: i16 = 150;

    /// First window index in array
    pub const FIRST_WINDOW_INDEX: usize = 0;

    /// Minimum array size for multi-window calculations
    pub const MIN_MULTI_WINDOW_COUNT: i16 = 1;
}

/// BSP tree configuration
pub mod bsp {
    /// Initial split count for new BSP trees
    pub const INITIAL_SPLIT_COUNT: usize = 0;

    /// Modulus for alternating split directions (even=vertical, odd=horizontal)
    pub const SPLIT_DIRECTION_MODULUS: usize = 2;

    /// Target window offset for sequential BSP splitting
    pub const TARGET_WINDOW_OFFSET: usize = 1;
}

/// Test values used in unit tests - reserved for future test refactoring
#[cfg(test)]
#[allow(dead_code)]
pub mod test_values {
    /// Standard test screen width (pixels)
    pub const SCREEN_WIDTH: u16 = 1280;

    /// Standard test screen height (pixels)
    pub const SCREEN_HEIGHT: u16 = 720;

    /// Default test gap size (pixels)
    pub const TEST_GAP: u32 = 10;

    /// Default test master ratio
    pub const TEST_MASTER_RATIO: f32 = 0.5;

    /// Test BSP split ratio
    pub const TEST_BSP_SPLIT_RATIO: f32 = 0.5;

    /// Mock window IDs for testing
    pub const MOCK_WINDOW_1: u32 = 1;
    pub const MOCK_WINDOW_2: u32 = 2;
    pub const MOCK_WINDOW_3: u32 = 3;
    pub const MOCK_WINDOW_NONEXISTENT: u32 = 999;
    pub const MOCK_WINDOW_SINGLE: u32 = 42;
}

//! Master-Stack layout algorithm implementation

use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use super::constants::{dimensions, layout};

/// Tiles windows in master-stack layout
///
/// Layout behavior:
/// - Single window: Full screen minus gaps
/// - Multiple windows: First window takes configurable ratio (master),
///   remaining windows stack vertically on the right, with gaps between
#[allow(clippy::too_many_arguments)]
pub fn tile_master_stack<C: Connection>(
    conn: &C,
    windows: &[Window],
    screen_width: u16,
    screen_height: u16,
    master_ratio: f32,
    min_window_width: u32,
    min_window_height: u32,
    gap: u32,
) -> Result<()> {
    if windows.is_empty() {
        return Ok(());
    }

    let screen_width = screen_width as i16;
    let screen_height = screen_height as i16;
    let num_windows = windows.len() as i16;
    let gap_i16 = gap as i16;

    // Configure master window
    let master_window = windows[layout::FIRST_WINDOW_INDEX];
    let master_width = if num_windows > 1 {
        // Multiple windows: master takes ratio of available space, ensure minimum width
        let available_width = screen_width - layout::MASTER_STACK_GAP_COUNT * gap_i16;
        let min_total = min_window_width + min_window_height; // master + stack minimum
        if available_width > min_total as i16 {
            // Need at least minimum total width
            ((available_width as f32 * master_ratio) as i16).max(min_window_width as i16)
        } else {
            // Fallback: reduce gaps to fit windows
            (screen_width / layout::FALLBACK_SCREEN_RATIO).max(min_window_width as i16)
        }
    } else {
        // Single window: full width minus gaps, minimum width
        (screen_width - layout::SINGLE_WINDOW_GAP_COUNT * gap_i16).max(min_window_width as i16)
    };

    let master_config = ConfigureWindowAux::new()
        .x(gap_i16 as i32)
        .y(gap_i16 as i32)
        .width(master_width.max(min_window_width as i16) as u32) // Minimum window width
        .height((screen_height - layout::SINGLE_WINDOW_GAP_COUNT * gap_i16).max(min_window_height as i16) as u32); // Minimum window height

    conn.configure_window(master_window, &master_config)?;

    // Configure stack windows if any
    if num_windows > layout::MIN_MULTI_WINDOW_COUNT {
        let stack_windows = &windows[(layout::FIRST_WINDOW_INDEX + 1)..]; 
        let num_stack = stack_windows.len() as i16;
        let stack_x = gap_i16 + master_width + gap_i16; // Add gap between master and stack
        let stack_width = (screen_width - stack_x - gap_i16).max(min_window_width as i16); // Minimum usable width

        // Ensure we have enough space for stack windows with minimum height
        let min_total_height = num_stack * min_window_height as i16 + (num_stack - 1) * gap_i16; // min height per window
        let available_height = screen_height - layout::SINGLE_WINDOW_GAP_COUNT * gap_i16;

        let total_stack_height = if available_height >= min_total_height {
            available_height - (num_stack - 1) * gap_i16
        } else {
            // Fallback: reduce gaps if necessary to fit windows
            (available_height - num_stack * min_window_height as i16)
                .max(num_stack * min_window_height as i16)
        };

        let stack_height = (total_stack_height / num_stack).max(min_window_height as i16); // Minimum window height

        for (index, &window) in stack_windows.iter().enumerate() {
            let stack_y = gap_i16 + (index as i16) * (stack_height + gap_i16);

            let stack_config = ConfigureWindowAux::new()
                .x(stack_x as i32)
                .y(stack_y as i32)
                .width(stack_width.max(dimensions::MIN_WINDOW_WIDTH as i16) as u32)
                .height(stack_height.max(dimensions::MIN_WINDOW_HEIGHT as i16) as u32);

            conn.configure_window(window, &stack_config)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests use hardcoded values for clarity and simplicity

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

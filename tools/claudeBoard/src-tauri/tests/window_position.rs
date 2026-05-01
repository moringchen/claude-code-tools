use claude_board::window_position::{
    default_overlay_position, monitor_contains_point, selected_monitor_index,
};

#[test]
fn centers_overlay_with_logical_monitor_bounds() {
    assert_eq!(default_overlay_position(0, 0, 1440, 420, 24), (510, 24));
}

#[test]
fn computes_logical_position_for_retina_monitor() {
    let scale_factor: f64 = 2.0;
    let logical_monitor_width = (2880.0_f64 / scale_factor).round() as u32;
    let logical_top_inset = (48.0_f64 / scale_factor).round() as i32;

    assert_eq!(
        default_overlay_position(0, 0, logical_monitor_width, 420, logical_top_inset),
        (510, 24)
    );
}

#[test]
fn centers_overlay_near_top_of_monitor_containing_pointer() {
    assert_eq!(default_overlay_position(100, 40, 1440, 420, 48), (610, 88));
}

#[test]
fn detects_when_pointer_is_inside_monitor_bounds() {
    assert!(monitor_contains_point(
        100, 40, 1440, 900, 800.0, 200.0, 1.0
    ));
    assert!(!monitor_contains_point(
        100, 40, 1440, 900, 50.0, 200.0, 1.0
    ));
}

#[test]
fn selects_monitor_containing_cursor_on_three_display_layout() {
    let monitors = [
        (0, 0, 2880, 1800, 1.0),
        (-1197, -1080, 1920, 1080, 1.0),
        (723, -1080, 1920, 1080, 1.0),
    ];

    assert_eq!(
        selected_monitor_index(&monitors, Some((1553.0, 776.0)), Some(0)),
        Some(0)
    );
    assert_eq!(
        selected_monitor_index(&monitors, Some((1800.0, -751.0)), Some(0)),
        Some(2)
    );
}

#[test]
fn falls_back_to_primary_when_cursor_is_unavailable() {
    let monitors = [(0, 0, 1440, 900, 1.0), (1440, 0, 1440, 900, 1.0)];

    assert_eq!(selected_monitor_index(&monitors, None, Some(1)), Some(1));
}

#[test]
fn falls_back_to_first_monitor_when_primary_is_unavailable() {
    let monitors = [(100, 40, 1440, 900, 1.0), (1540, 40, 1440, 900, 1.0)];

    assert_eq!(selected_monitor_index(&monitors, None, None), Some(0));
}

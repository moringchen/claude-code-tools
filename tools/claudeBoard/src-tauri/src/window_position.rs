pub type MonitorBounds = (i32, i32, u32, u32, f64);

pub const OVERLAY_WIDTH: i32 = 260;
pub const DEBUG_WINDOW_WIDTH: i32 = 420;
pub const DEBUG_WINDOW_HEIGHT: i32 = 480;
pub const WINDOW_GAP: i32 = 16;

pub fn logical_monitor_bounds(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    scale_factor: f64,
) -> (i32, i32, u32, u32) {
    let logical_monitor_x = (monitor_x as f64 / scale_factor).round() as i32;
    let logical_monitor_y = (monitor_y as f64 / scale_factor).round() as i32;
    let logical_monitor_width = (monitor_width as f64 / scale_factor).round() as u32;
    let logical_monitor_height = (monitor_height as f64 / scale_factor).round() as u32;
    (
        logical_monitor_x,
        logical_monitor_y,
        logical_monitor_width,
        logical_monitor_height,
    )
}

pub fn default_overlay_position(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    window_width: i32,
    top_inset: i32,
) -> (i32, i32) {
    let centered_x = monitor_x + ((monitor_width as i32 - window_width) / 2);
    let top_y = monitor_y + top_inset;
    (centered_x, top_y)
}

pub fn debug_window_position(overlay_x: i32, overlay_y: i32, overlay_width: i32) -> (i32, i32) {
    (overlay_x + overlay_width + WINDOW_GAP, overlay_y)
}

pub fn monitor_contains_point(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    pointer_x: f64,
    pointer_y: f64,
    _scale_factor: f64,
) -> bool {
    let monitor_right = monitor_x as f64 + monitor_width as f64;
    let monitor_bottom = monitor_y as f64 + monitor_height as f64;

    pointer_x >= monitor_x as f64
        && pointer_x < monitor_right
        && pointer_y >= monitor_y as f64
        && pointer_y < monitor_bottom
}

pub fn selected_monitor_index(
    monitors: &[MonitorBounds],
    pointer: Option<(f64, f64)>,
    primary_monitor_index: Option<usize>,
) -> Option<usize> {
    pointer
        .and_then(|(pointer_x, pointer_y)| {
            monitors.iter().position(
                |(monitor_x, monitor_y, monitor_width, monitor_height, scale_factor)| {
                    monitor_contains_point(
                        *monitor_x,
                        *monitor_y,
                        *monitor_width,
                        *monitor_height,
                        pointer_x,
                        pointer_y,
                        *scale_factor,
                    )
                },
            )
        })
        .or(primary_monitor_index)
        .or_else(|| (!monitors.is_empty()).then_some(0))
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use claude_board::window_position::{default_overlay_position, selected_monitor_index};
use tauri::{LogicalPosition, Manager};

fn apply_overlay_position(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let pointer = app
            .cursor_position()
            .ok()
            .map(|position| (position.x, position.y));
        eprintln!("[claudeBoard] cursor_position={pointer:?}");

        let monitors = app.available_monitors()?;
        let monitor_bounds = monitors
            .iter()
            .enumerate()
            .map(|(index, monitor)| {
                let bounds = (
                    monitor.position().x,
                    monitor.position().y,
                    monitor.size().width,
                    monitor.size().height,
                    monitor.scale_factor(),
                );
                eprintln!("[claudeBoard] monitor[{index}] bounds={bounds:?}");
                bounds
            })
            .collect::<Vec<_>>();
        let primary_monitor_index =
            app.primary_monitor()
                .ok()
                .flatten()
                .and_then(|primary_monitor| {
                    let primary_bounds = (
                        primary_monitor.position().x,
                        primary_monitor.position().y,
                        primary_monitor.size().width,
                        primary_monitor.size().height,
                        primary_monitor.scale_factor(),
                    );
                    eprintln!("[claudeBoard] primary_monitor bounds={primary_bounds:?}");
                    monitor_bounds.iter().position(|bounds| *bounds == primary_bounds)
                });
        eprintln!("[claudeBoard] primary_monitor_index={primary_monitor_index:?}");

        let monitor_index = selected_monitor_index(&monitor_bounds, pointer, primary_monitor_index);
        eprintln!("[claudeBoard] selected_monitor_index={monitor_index:?}");

        if let Some(monitor_index) = monitor_index {
            let monitor = &monitors[monitor_index];
            let scale_factor = monitor.scale_factor();
            let logical_monitor_x = (monitor.position().x as f64 / scale_factor).round() as i32;
            let logical_monitor_y = (monitor.position().y as f64 / scale_factor).round() as i32;
            let logical_monitor_width = (monitor.size().width as f64 / scale_factor).round() as u32;
            let logical_monitor_height = (monitor.size().height as f64 / scale_factor).round() as u32;
            let logical_top_inset = (48.0 / scale_factor).round() as i32;
            let (x, y) = default_overlay_position(
                logical_monitor_x,
                logical_monitor_y,
                logical_monitor_width,
                420,
                logical_top_inset,
            );
            eprintln!(
                "[claudeBoard] placing overlay monitor_index={monitor_index} physical_monitor_origin=({}, {}) physical_monitor_size={}x{} logical_monitor_origin=({}, {}) logical_monitor_size={}x{} scale_factor={} window_width=420 logical_top_inset={} final_position=({}, {})",
                monitor.position().x,
                monitor.position().y,
                monitor.size().width,
                monitor.size().height,
                logical_monitor_x,
                logical_monitor_y,
                logical_monitor_width,
                logical_monitor_height,
                scale_factor,
                logical_top_inset,
                x,
                y,
            );
            eprintln!(
                "[claudeBoard] logical_position=({}, {}) scale_factor={}",
                x, y, scale_factor,
            );
            eprintln!("[claudeBoard] is_visible_before={:?}", window.is_visible().ok());
            window.set_position(LogicalPosition::new(x, y))?;
            eprintln!(
                "[claudeBoard] pre_show_outer_position={:?}",
                window.outer_position().ok()
            );
            eprintln!(
                "[claudeBoard] pre_show_outer_position_logical={:?}",
                window
                    .outer_position()
                    .ok()
                    .map(|position| position.to_logical::<i32>(scale_factor))
            );
            window.show()?;
            eprintln!("[claudeBoard] is_visible_after_show={:?}", window.is_visible().ok());
            window.set_position(LogicalPosition::new(x, y))?;
            eprintln!("[claudeBoard] actual_outer_position={:?}", window.outer_position().ok());
            eprintln!("[claudeBoard] actual_inner_position={:?}", window.inner_position().ok());
            eprintln!(
                "[claudeBoard] actual_outer_position_logical={:?}",
                window
                    .outer_position()
                    .ok()
                    .map(|position| position.to_logical::<i32>(scale_factor))
            );
            eprintln!(
                "[claudeBoard] actual_inner_position_logical={:?}",
                window
                    .inner_position()
                    .ok()
                    .map(|position| position.to_logical::<i32>(scale_factor))
            );
            eprintln!("[claudeBoard] actual_outer_size={:?}", window.outer_size().ok());
            eprintln!("[claudeBoard] actual_inner_size={:?}", window.inner_size().ok());
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.run_on_main_thread(move || {
                if let Err(error) = apply_overlay_position(&app_handle) {
                    eprintln!("[claudeBoard] apply_overlay_position failed: {error}");
                }
            })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run claudeBoard window");
}

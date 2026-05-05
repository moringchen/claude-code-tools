#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use claude_board::window_position::{default_overlay_position, selected_monitor_index};
use tauri::{LogicalPosition, Manager, WindowEvent};

#[cfg(target_os = "macos")]
fn set_window_level(window: &tauri::WebviewWindow, level: i32) -> tauri::Result<()> {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    let ns_window = window.ns_window()?;
    unsafe {
        let _: () = msg_send![ns_window as *mut Object, setLevel: level];
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_window_hides_on_deactivate(window: &tauri::WebviewWindow, hides: bool) -> tauri::Result<()> {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    let ns_window = window.ns_window()?;
    unsafe {
        let _: () = msg_send![ns_window as *mut Object, setHidesOnDeactivate: hides];
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_always_visible_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    // Ensure window never hides on deactivate
    set_window_hides_on_deactivate(window, false)?;
    // Set initial level to floating so it's visible
    set_window_level(window, 5)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn set_window_level(_window: &tauri::WebviewWindow, _level: i32) -> tauri::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn setup_always_visible_window(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    Ok(())
}

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
        .invoke_handler(tauri::generate_handler![claude_board::sound::read_sound_file, claude_board::sound::log_from_frontend])
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.run_on_main_thread(move || {
                if let Err(error) = apply_overlay_position(&app_handle) {
                    eprintln!("[claudeBoard] apply_overlay_position failed: {error}");
                }
            })?;

            // Set up window event handlers
            // Window stays fixed in position and always visible, only z-index changes
            if let Some(window) = app.get_webview_window("main") {
                // Ensure window never hides/minimizes when deactivated
                let _ = setup_always_visible_window(&window);

                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    #[cfg(target_os = "macos")]
                    match event {
                        WindowEvent::Focused(true) => {
                            eprintln!("[claudeBoard] window focused - raising z-index");
                            // Raise window level to floating when focused
                            let _ = set_window_level(&window_clone, 5);
                        }
                        WindowEvent::Focused(false) => {
                            eprintln!("[claudeBoard] window unfocused - keeping visible at background level");
                            // Keep window at a level where it remains visible but behind other windows
                            // Using level 3 (torn-off menu level) keeps it visible but below normal windows
                            let _ = set_window_level(&window_clone, 3);
                            // Ensure window stays visible and doesn't hide
                            let _ = window_clone.show();
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run claudeBoard window");
}

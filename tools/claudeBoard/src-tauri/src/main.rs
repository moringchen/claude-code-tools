#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use claude_board::{
    macos_window_behavior::{
        macos_window_collection_behavior, macos_window_level_for_mode, OverlayZOrderMode,
    },
    window_position::{
        default_overlay_position, logical_monitor_bounds, selected_monitor_index, OVERLAY_WIDTH,
    },
};
use tauri::{LogicalPosition, Manager, RunEvent, WindowEvent};

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
fn set_window_can_hide(window: &tauri::WebviewWindow, can_hide: bool) -> tauri::Result<()> {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    let ns_window = window.ns_window()?;
    unsafe {
        let _: () = msg_send![ns_window as *mut Object, setCanHide: can_hide];
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_window_collection_behavior(window: &tauri::WebviewWindow, behavior: u64) -> tauri::Result<()> {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    let ns_window = window.ns_window()?;
    unsafe {
        let _: () = msg_send![ns_window as *mut Object, setCollectionBehavior: behavior];
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn keep_window_visible_when_inactive(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    set_window_hides_on_deactivate(window, false)?;
    set_window_can_hide(window, false)?;
    set_window_collection_behavior(window, macos_window_collection_behavior())?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_always_visible_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    keep_window_visible_when_inactive(window)?;
    set_window_level(window, macos_window_level_for_mode(OverlayZOrderMode::Foreground))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn keep_window_visible_when_inactive(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
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
            let (logical_monitor_x, logical_monitor_y, logical_monitor_width, _logical_monitor_height) =
                logical_monitor_bounds(
                    monitor.position().x,
                    monitor.position().y,
                    monitor.size().width,
                    monitor.size().height,
                    scale_factor,
                );
            let logical_top_inset = (48.0 / scale_factor).round() as i32;
            let (x, y) = default_overlay_position(
                logical_monitor_x,
                logical_monitor_y,
                logical_monitor_width,
                OVERLAY_WIDTH,
                logical_top_inset,
            );
            window.set_position(LogicalPosition::new(x, y))?;
            window.show()?;
            window.set_position(LogicalPosition::new(x, y))?;
        }
    }

    Ok(())
}

fn attach_window_handlers(window: &tauri::WebviewWindow) {
    let _ = setup_always_visible_window(window);

    let window_clone = window.clone();
    window.on_window_event(move |event| {
        #[cfg(target_os = "macos")]
        match event {
            WindowEvent::Focused(true) => {
                let _ = set_window_level(
                    &window_clone,
                    macos_window_level_for_mode(OverlayZOrderMode::Foreground),
                );
            }
            WindowEvent::Focused(false) => {
                let _ = set_window_level(
                    &window_clone,
                    macos_window_level_for_mode(OverlayZOrderMode::Background),
                );
                let _ = keep_window_visible_when_inactive(&window_clone);
                let _ = window_clone.show();
            }
            _ => {}
        }
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            claude_board::sound::read_sound_file,
            claude_board::sound::play_sound_file,
            claude_board::sound::log_from_frontend
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.run_on_main_thread(move || {
                if let Err(error) = apply_overlay_position(&app_handle) {
                    eprintln!("[claudeBoard] apply_overlay_position failed: {error}");
                }
            })?;

            if let Some(window) = app.get_webview_window("main") {
                attach_window_handlers(&window);
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build claudeBoard window")
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = set_window_level(
                        &window,
                        macos_window_level_for_mode(OverlayZOrderMode::Foreground),
                    );
                }
            }
        });
}

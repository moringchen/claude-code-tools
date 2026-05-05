use std::path::Path;
use tauri::command;

#[command]
pub fn read_sound_file(path: String) -> Result<Vec<u8>, String> {
    eprintln!("[sound:rust] Reading sound file: {}", path);
    let path = Path::new(&path);
    match std::fs::read(path) {
        Ok(data) => {
            eprintln!("[sound:rust] File read successfully, size: {} bytes", data.len());
            Ok(data)
        }
        Err(e) => {
            eprintln!("[sound:rust] Failed to read file: {}", e);
            Err(format!("Failed to read sound file: {}", e))
        }
    }
}

#[command]
pub fn log_from_frontend(level: String, message: String) {
    match level.as_str() {
        "error" => eprintln!("[frontend:err] {}", message),
        "warn" => eprintln!("[frontend:warn] {}", message),
        _ => eprintln!("[frontend:log] {}", message),
    }
}

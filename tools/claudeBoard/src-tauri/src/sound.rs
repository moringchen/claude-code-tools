use std::path::Path;
use std::process::Command;
use tauri::command;

const WAITING_SOUND_PATH: &str = "/Users/moringchen/Downloads/待回复.mp3";
const COMPLETED_SOUND_PATH: &str = "/Users/moringchen/Downloads/任务完成.mp3";

pub fn sound_path_for_type(sound_type: &str) -> Option<&'static str> {
    match sound_type {
        "waiting" => Some(WAITING_SOUND_PATH),
        "completed" => Some(COMPLETED_SOUND_PATH),
        _ => None,
    }
}

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
pub fn play_sound_file(sound_type: String) -> Result<(), String> {
    let path = sound_path_for_type(&sound_type)
        .ok_or_else(|| format!("Unknown sound type: {sound_type}"))?;

    eprintln!("[sound:rust] Starting playback type={} path={}", sound_type, path);

    Command::new("afplay")
        .arg(path)
        .spawn()
        .map(|child| {
            eprintln!("[sound:rust] Spawned afplay pid={} type={}", child.id(), sound_type);
        })
        .map_err(|error| format!("Failed to play sound file: {error}"))
}

#[command]
pub fn log_from_frontend(level: String, message: String) {
    match level.as_str() {
        "error" => eprintln!("[frontend:err] {}", message),
        "warn" => eprintln!("[frontend:warn] {}", message),
        _ => eprintln!("[frontend:log] {}", message),
    }
}

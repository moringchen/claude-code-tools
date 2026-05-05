use std::{
    env, fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{event_buffer::drain_events, hooks_config::upsert_hooks, store::TaskStore};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookPaths {
    pub settings_path: PathBuf,
    pub dispatch_script_path: PathBuf,
    pub buffer_path: PathBuf,
}

#[cfg(target_os = "windows")]
const DISPATCH_SCRIPT_NAME: &str = "hook-dispatch.ps1";
#[cfg(not(target_os = "windows"))]
const DISPATCH_SCRIPT_NAME: &str = "hook-dispatch.sh";

#[cfg(target_os = "windows")]
const DISPATCH_SCRIPT_CONTENTS: &str = include_str!("../../scripts/hook-dispatch.ps1");
#[cfg(not(target_os = "windows"))]
const DISPATCH_SCRIPT_CONTENTS: &str = include_str!("../../scripts/hook-dispatch.sh");

pub fn ensure_hook_setup() -> io::Result<HookPaths> {
    let user_home = resolve_user_home()?;
    ensure_hook_setup_in_home(&user_home)
}

pub fn ensure_hook_setup_in_home(user_home: &Path) -> io::Result<HookPaths> {
    let claude_dir = user_home.join(".claude");
    let board_dir = user_home.join(".claude-board");
    let settings_path = claude_dir.join("settings.json");
    let dispatch_script_path = board_dir.join(DISPATCH_SCRIPT_NAME);
    let buffer_path = board_dir.join("events.jsonl");

    fs::create_dir_all(&claude_dir)?;
    fs::create_dir_all(&board_dir)?;
    fs::write(&dispatch_script_path, DISPATCH_SCRIPT_CONTENTS)?;
    make_executable_if_needed(&dispatch_script_path)?;

    let existing_settings = fs::read_to_string(&settings_path).unwrap_or_else(|_| "{}".to_string());
    let updated_settings = upsert_hooks(&existing_settings, &dispatch_script_path.to_string_lossy())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(&settings_path, updated_settings)?;

    eprintln!(
        "[claudeBoard] startup hook setup completed settings_path={} dispatch_script_path={}",
        settings_path.display(),
        dispatch_script_path.display()
    );

    Ok(HookPaths {
        settings_path,
        dispatch_script_path,
        buffer_path,
    })
}

pub fn drain_buffered_events_into_store(
    store: &Arc<Mutex<TaskStore>>,
    buffer_path: &Path,
) -> io::Result<usize> {
    let events = drain_events(buffer_path)?;
    let replayed = events.len();

    if replayed == 0 {
        return Ok(0);
    }

    let mut store = store.lock().unwrap();
    for event in events {
        store.apply(event);
    }

    eprintln!(
        "[claudeBoard] startup buffered events replayed count={} buffer_path={}",
        replayed,
        buffer_path.display()
    );

    Ok(replayed)
}

fn resolve_user_home() -> io::Result<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "user home not found"))
}

#[cfg(unix)]
fn make_executable_if_needed(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable_if_needed(_path: &Path) -> io::Result<()> {
    Ok(())
}

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use claude_board::{
    event_log::{append_event_log, reset_event_log},
    model::{HookEvent, HookEventType},
    scan::scan_and_filter_rows,
    server::{build_router_with_state_path, refresh_scan},
    session_state::load_snapshot,
    startup_hooks::ensure_hook_setup,
    store::TaskStore,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async_main());
}

async fn async_main() {
    if let Err(error) = reset_event_log() {
        eprintln!("[claudeBoard] event log reset failed: {error}");
    }
    append_event_log("----- daemon_start -----");
    if let Err(error) = ensure_hook_setup() {
        eprintln!("[claudeBoard] daemon hook setup failed: {error}");
    }

    let state_path = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".claude-board")
        .join("session-state.json");
    let store = Arc::new(Mutex::new(TaskStore::default()));
    match load_snapshot(&state_path) {
        Ok(snapshot) => store.lock().unwrap().restore_snapshot(snapshot),
        Err(error) => eprintln!("[claudeBoard] daemon session state load failed: {error}"),
    }

    // Load and replay buffered events first
    let buffer_path = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".claude-board")
        .join("events.jsonl");
    eprintln!("[claudeBoard] daemon loading buffered events from {:?}", buffer_path);
    match drain_buffered_events(&buffer_path) {
        Ok(events) => {
            eprintln!("[claudeBoard] daemon replaying {} buffered events", events.len());
            for event in events {
                store.lock().unwrap().apply(event);
            }
            if let Err(error) = claude_board::session_state::save_snapshot(
                &state_path,
                &store.lock().unwrap().persisted_snapshot(),
            ) {
                eprintln!("[claudeBoard] daemon session state save after replay failed: {error}");
            }
        }
        Err(error) => {
            eprintln!("[claudeBoard] daemon no buffered events to replay: {error}");
        }
    }

    eprintln!("[claudeBoard] daemon initial_refresh start");
    match refresh_scan(&store, scan_rows, current_timestamp) {
        Ok(()) => eprintln!("[claudeBoard] daemon initial_refresh completed"),
        Err(error) => eprintln!("[claudeBoard] daemon initial_refresh failed: {error}"),
    }
    let app = build_router_with_state_path(store, scan_rows, current_timestamp, Some(state_path));
    let addr = SocketAddr::from(([127, 0, 0, 1], 46123));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

// Buffered event format from hook dispatch script
#[derive(Debug, serde::Deserialize)]
struct BufferedEvent {
    #[serde(rename = "hook_event_name")]
    hook_event_name: String,
    session_id: String,
    #[serde(rename = "claude_board_pid")]
    pid: u32,
    #[serde(rename = "claude_board_occurred_at")]
    occurred_at: String,
    cwd: Option<String>,
    prompt: Option<String>,
    permission_mode: Option<String>,
}

fn drain_buffered_events(path: &std::path::Path) -> std::io::Result<Vec<HookEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<BufferedEvent>(&line) {
            Ok(buffered) => {
                // Convert BufferedEvent to HookEvent
                let event_type = match buffered.hook_event_name.as_str() {
                    "SessionStart" | "UserPromptSubmit" | "PreToolUse" | "SubagentStart"
                    | "TaskCreated" | "PreCompact" | "PermissionDenied" | "ElicitationResult" => {
                        HookEventType::TaskCreated
                    }
                    "PermissionRequest" | "Elicitation" => HookEventType::PermissionRequest,
                    "PostToolUse" | "PostToolUseFailure" | "TaskCompleted" | "Stop"
                    | "StopFailure" => HookEventType::TaskCompleted,
                    "SubagentStop" | "SessionEnd" => HookEventType::SessionEnd,
                    "PostConversationTurn" => match buffered.permission_mode.as_deref() {
                        Some("acceptEdits") => HookEventType::PermissionRequest,
                        Some("bypassPermissions")
                            if buffered.prompt.as_ref().map(|prompt| prompt.is_empty()).unwrap_or(true) =>
                        {
                            HookEventType::TaskCompleted
                        }
                        _ => continue,
                    },
                    _ => continue, // Skip unknown events
                };

                // Extract title from prompt or cwd (UTF-8 safe truncation)
                let title = if let Some(prompt) = &buffered.prompt {
                    if !prompt.is_empty() {
                        let trimmed = prompt.trim();
                        if trimmed.chars().count() > 50 {
                            let truncated: String = trimmed.chars().take(50).collect();
                            format!("{}...", truncated)
                        } else {
                            trimmed.to_string()
                        }
                    } else {
                        buffered
                            .cwd
                            .as_ref()
                            .and_then(|p| p.split('/').filter(|s| !s.is_empty()).last())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    }
                } else {
                    buffered
                        .cwd
                        .as_ref()
                        .and_then(|p| p.split('/').filter(|s| !s.is_empty()).last())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                };

                events.push(HookEvent {
                    event_type,
                    session_id: buffered.session_id,
                    agent_id: None,
                    pid: buffered.pid,
                    title,
                    conversation_content: buffered.prompt.clone().filter(|prompt| !prompt.trim().is_empty()),
                    occurred_at: buffered.occurred_at,
                });
            }
            Err(e) => {
                eprintln!("[claudeBoard] failed to parse buffered event: {}", e);
            }
        }
    }

    // Remove the file after reading
    std::fs::remove_file(path)?;
    Ok(events)
}

fn scan_rows() -> std::io::Result<Vec<String>> {
    eprintln!("[claudeBoard] scan_rows start");
    // 使用 state 列过滤停止的进程
    let output = Command::new("ps")
        .args(["-axo", "pid=,ppid=,state=,command="])
        .output()?;

    eprintln!("[claudeBoard] scan_rows ps_status={}", output.status);
    if !output.status.success() {
        eprintln!("[claudeBoard] scan_rows ps failed status={}", output.status);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows = scan_and_filter_rows(&stdout);
    eprintln!(
        "[claudeBoard] scan_rows completed accepted_row_count={}",
        rows.len()
    );

    Ok(rows)
}

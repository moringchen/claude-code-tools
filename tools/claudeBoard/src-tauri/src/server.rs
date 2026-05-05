use std::io;
use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{header, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    model::{HookEvent, HookEventType, TaskSnapshot},
    scan::rebuild_tasks_from_rows,
    session_meta::{get_task_title_by_pid, get_task_title_by_session_id},
    sound::play_sound_file,
    store::TaskStore,
};

#[cfg(target_os = "macos")]
fn is_observer_process(pid: u32) -> bool {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let cmd = String::from_utf8_lossy(&output.stdout);
            cmd.contains("--disallowedTools") && cmd.contains("Bash,Read,Write,Edit")
        }
        _ => false,
    }
}

#[cfg(not(target_os = "macos"))]
fn is_observer_process(pid: u32) -> bool {
    // On Linux, read from /proc/PID/cmdline
    let path = format!("/proc/{}/cmdline", pid);
    if let Ok(cmd) = std::fs::read_to_string(&path) {
        cmd.contains("--disallowedTools") && cmd.contains("Bash,Read,Write,Edit")
    } else {
        false
    }
}

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<TaskStore>>,
    pub scan_refresh: Arc<dyn Fn() -> io::Result<()> + Send + Sync>,
}

// Claude Code native hook event format
#[derive(Debug, Deserialize)]
struct ClaudeCodeHookEvent {
    #[serde(rename = "hook_event_name")]
    hook_event_name: String,
    session_id: String,
    #[serde(rename = "claude_board_pid")]
    pid: u32,
    #[serde(rename = "claude_board_title")]
    title: String,
    #[serde(rename = "claude_board_occurred_at")]
    occurred_at: String,
    cwd: Option<String>,
    // Optional fields for different event types
    permission_mode: Option<String>,
    // UserPromptSubmit contains the conversation topic
    prompt: Option<String>,
    // Tool name for PreToolUse/PermissionRequest
    tool_name: Option<String>,
    // agent_id indicates a subagent task - filter these out
    agent_id: Option<String>,
}

fn extract_task_title(event: &ClaudeCodeHookEvent) -> String {
    let session_title = get_task_title_by_session_id(&event.session_id, event.cwd.as_deref(), &event.title);
    if !session_title.is_empty() && session_title != event.title {
        return session_title;
    }

    let pid_title = get_task_title_by_pid(event.pid, event.cwd.as_deref(), &event.title);
    if !pid_title.is_empty() && pid_title != event.title {
        return pid_title;
    }

    // Fallback to prompt-based title extraction
    if let Some(prompt) = &event.prompt {
        if !prompt.is_empty() {
            let trimmed = prompt.trim();
            if trimmed.chars().count() > 50 {
                let truncated: String = trimmed.chars().take(50).collect();
                return format!("{}...", truncated);
            }
            return trimmed.to_string();
        }
    }

    // Final fallback to cwd or default
    event.cwd
        .as_deref()
        .and_then(|path| path.split('/').filter(|s| !s.is_empty()).last())
        .map(|s| s.to_string())
        .unwrap_or_else(|| event.title.clone())
}

fn convert_claude_code_event(event: ClaudeCodeHookEvent) -> Option<HookEvent> {
    eprintln!("[claudeBoard] converting event: {} permission_mode={:?} prompt={:?} agent_id={:?}",
        event.hook_event_name, event.permission_mode, event.prompt.as_ref().map(|p| &p[..p.len().min(30)]), event.agent_id);

    // Filter out subagent tasks (they have agent_id)
    if event.agent_id.is_some() {
        eprintln!("[claudeBoard] filtering out subagent task with agent_id={:?}", event.agent_id);
        return None;
    }

    // Filter out observer/sidecar processes (those with --disallowedTools)
    if is_observer_process(event.pid) {
        eprintln!("[claudeBoard] filtering out observer process pid={}", event.pid);
        return None;
    }

    let event_type = match event.hook_event_name.as_str() {
        // Task lifecycle events
        "TaskCreated" => HookEventType::TaskCreated,
        "TaskCompleted" => HookEventType::TaskCompleted,

        // Permission/waiting events
        "PermissionRequest" => HookEventType::PermissionRequest,
        "PermissionDenied" => HookEventType::PermissionDenied,
        "Elicitation" => HookEventType::PermissionRequest,
        "PreUserInteraction" => HookEventType::PermissionRequest,

        // Session lifecycle events
        "SessionStart" => HookEventType::TaskCreated,
        "SessionEnd" => HookEventType::SessionEnd,

        // Tool execution events
        "PreToolUse" => HookEventType::TaskCreated,
        "PostToolUse" => {
            // Tool execution completed - task continues running
            return None;
        }
        "PostToolUseFailure" => {
            // Tool failed but task continues
            return None;
        }

        // User interaction events
        "UserPromptSubmit" => {
            // User submitted input - task is running
            HookEventType::TaskCreated
        }

        // Conversation turn events
        "PostConversationTurn" => {
            match event.permission_mode.as_deref() {
                Some("acceptEdits") => HookEventType::PermissionRequest,
                Some("bypassPermissions") => {
                    // In bypass mode, check if there's a prompt
                    if event.prompt.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
                        eprintln!("[claudeBoard] PostConversationTurn in bypass mode - treating as completed");
                        HookEventType::TaskCompleted
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        // Stop events
        "Stop" => HookEventType::TaskCompleted,
        "StopFailure" => HookEventType::TaskCompleted,

        // Subagent events - these should be filtered by agent_id but just in case
        "SubagentStart" => return None,
        "SubagentStop" => return None,

        // System/config events - don't affect task state
        "Notification" => return None,
        "ConfigChange" => return None,
        "WorktreeCreate" => return None,
        "WorktreeRemove" => return None,
        "CwdChanged" => return None,
        "FileChanged" => return None,
        "Setup" => return None,
        "InstructionsLoaded" => return None,

        // Compact events - task continues
        "PreCompact" => return None,
        "PostCompact" => return None,

        // Unknown events
        _ => {
            eprintln!("[claudeBoard] unknown hook event type: {}", event.hook_event_name);
            return None;
        }
    };

    // Extract task title from prompt (user input) or cwd
    let title = extract_task_title(&event);

    Some(HookEvent {
        event_type,
        session_id: event.session_id,
        pid: event.pid,
        title,
        occurred_at: event.occurred_at,
    })
}

pub fn build_router<F, N>(store: Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> Router
where
    F: Fn() -> io::Result<Vec<String>> + Send + Sync + 'static,
    N: Fn() -> String + Send + Sync + 'static,
{
    let refresh_store = Arc::clone(&store);
    let scan_refresh = Arc::new(move || {
        eprintln!("[claudeBoard] refresh start source=router");
        let rows = scan_rows()?;
        eprintln!("[claudeBoard] refresh scan_rows source=router row_count={}", rows.len());

        // Extract alive pids from scan rows
        let alive_pids: Vec<u32> = rows
            .iter()
            .filter_map(|row| {
                let parts: Vec<&str> = row.split('\t').collect();
                parts.get(1)?.parse::<u32>().ok()
            })
            .collect();
        eprintln!("[claudeBoard] refresh alive_pids source=router pids={:?}", alive_pids);

        let timestamp = now();
        eprintln!("[claudeBoard] refresh timestamp source=router value={timestamp}");
        let tasks = rebuild_tasks_from_rows(&rows, &timestamp);
        eprintln!("[claudeBoard] refresh rebuilt source=router task_count={}", tasks.len());
        refresh_store.lock().unwrap().replace_scanned_tasks(tasks, &alive_pids);
        Ok(())
    });

    Router::new()
        .route("/events", post(post_event))
        .route("/refresh", post(post_refresh))
        .route("/snapshot", get(get_snapshot))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(AppState {
            store,
            scan_refresh,
        })
}

pub fn refresh_scan<F, N>(store: &Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> io::Result<()>
where
    F: Fn() -> io::Result<Vec<String>>,
    N: Fn() -> String,
{
    eprintln!("[claudeBoard] refresh start source=function");
    let rows = scan_rows()?;
    eprintln!("[claudeBoard] refresh scan_rows source=function row_count={}", rows.len());

    // Extract alive pids from scan rows
    let alive_pids: Vec<u32> = rows
        .iter()
        .filter_map(|row| {
            let parts: Vec<&str> = row.split('\t').collect();
            parts.get(1)?.parse::<u32>().ok()
        })
        .collect();
    eprintln!("[claudeBoard] refresh alive_pids source=function pids={:?}", alive_pids);

    let timestamp = now();
    eprintln!("[claudeBoard] refresh timestamp source=function value={timestamp}");
    let tasks = rebuild_tasks_from_rows(&rows, &timestamp);
    eprintln!("[claudeBoard] refresh rebuilt source=function task_count={}", tasks.len());
    store.lock().unwrap().replace_scanned_tasks(tasks, &alive_pids);
    Ok(())
}

async fn post_event(
    State(state): State<AppState>,
    Json(event): Json<ClaudeCodeHookEvent>,
) -> StatusCode {
    eprintln!("[claudeBoard] received hook event: {} for session {}",
        event.hook_event_name, event.session_id);

    match convert_claude_code_event(event) {
        Some(hook_event) => {
            eprintln!("[claudeBoard] converted to: {:?} for pid {} title {}",
                hook_event.event_type, hook_event.pid, hook_event.title);
            let changed_status = state.store.lock().unwrap().apply(hook_event);
            if matches!(changed_status, Some(crate::model::TaskStatus::NeedsUser)) {
                let _ = play_sound_file("waiting".to_string());
            }
            if matches!(changed_status, Some(crate::model::TaskStatus::Completed)) {
                let _ = play_sound_file("completed".to_string());
            }
            StatusCode::ACCEPTED
        }
        None => {
            eprintln!("[claudeBoard] event filtered out");
            StatusCode::ACCEPTED
        }
    }
}

async fn post_refresh(State(state): State<AppState>) -> StatusCode {
    match (state.scan_refresh)() {
        Ok(()) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_snapshot(State(state): State<AppState>) -> Json<TaskSnapshot> {
    let snapshot = state.store.lock().unwrap().snapshot();
    eprintln!(
        "[claudeBoard] snapshot served total={} running={} needs_user={} completed={}",
        snapshot.counts.total,
        snapshot.counts.running,
        snapshot.counts.needs_user,
        snapshot.counts.completed
    );
    Json(snapshot)
}

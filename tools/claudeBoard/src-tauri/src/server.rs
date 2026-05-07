use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    event_log::append_event_log,
    focus::{resolve_focus, FocusRequest, HostActivator},
    model::{
        DebugSnapshot, HookDebugDisposition, HookDebugEntry, HookEvent, HookEventType, TaskSnapshot,
        TaskLiveness,
    },
    scan::{compute_scan, rebuild_tasks_from_rows},
    session_meta::{get_task_title_by_pid, get_task_title_by_session_id},
    session_state::save_snapshot,
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
    pub state_path: Option<PathBuf>,
}

struct ProcessHostActivator;

impl HostActivator for ProcessHostActivator {
    fn activate(&self, attempt: &crate::focus::FocusAttempt) -> bool {
        let command = if cfg!(target_os = "macos") {
            crate::focus::macos::command_for(attempt)
        } else if cfg!(target_os = "windows") {
            crate::focus::windows::command_for(attempt)
        } else {
            return false;
        };

        Command::new(&command.program)
            .args(&command.args)
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
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

fn conversation_content(event: &ClaudeCodeHookEvent) -> Option<String> {
    event
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|prompt| !prompt.is_empty())
        .map(ToOwned::to_owned)
}

fn prompt_preview(prompt: Option<&str>) -> Option<String> {
    prompt
        .map(str::trim)
        .filter(|prompt| !prompt.is_empty())
        .map(|prompt| {
            let mut preview: String = prompt.chars().take(80).collect();
            if prompt.chars().count() > 80 {
                preview.push_str("...");
            }
            preview
        })
}

fn filtered_debug_entry(event: &ClaudeCodeHookEvent, reason: &str) -> HookDebugEntry {
    HookDebugEntry {
        occurred_at: event.occurred_at.clone(),
        hook_event_name: event.hook_event_name.clone(),
        session_id: event.session_id.clone(),
        pid: event.pid,
        title: extract_task_title(event),
        permission_mode: event.permission_mode.clone(),
        prompt_preview: prompt_preview(event.prompt.as_deref()),
        agent_id: event.agent_id.clone(),
        disposition: HookDebugDisposition::Filtered,
        mapped_event_type: None,
        filter_reason: Some(reason.to_string()),
        previous_status: None,
        next_status: None,
    }
}

fn accepted_debug_entry(event: &ClaudeCodeHookEvent, mapped_event_type: HookEventType) -> HookDebugEntry {
    HookDebugEntry {
        occurred_at: event.occurred_at.clone(),
        hook_event_name: event.hook_event_name.clone(),
        session_id: event.session_id.clone(),
        pid: event.pid,
        title: extract_task_title(event),
        permission_mode: event.permission_mode.clone(),
        prompt_preview: prompt_preview(event.prompt.as_deref()),
        agent_id: event.agent_id.clone(),
        disposition: HookDebugDisposition::Accepted,
        mapped_event_type: Some(mapped_event_type),
        filter_reason: None,
        previous_status: None,
        next_status: None,
    }
}

fn convert_claude_code_event(event: ClaudeCodeHookEvent) -> Result<(HookEvent, HookDebugEntry), HookDebugEntry> {
    eprintln!("[claudeBoard] converting event: {} permission_mode={:?} prompt={:?} agent_id={:?}",
        event.hook_event_name, event.permission_mode, event.prompt.as_ref().map(|p| &p[..p.len().min(30)]), event.agent_id);

    if event.agent_id.is_some()
        && !matches!(event.hook_event_name.as_str(), "SubagentStart" | "SubagentStop")
    {
        eprintln!("[claudeBoard] filtering out subagent task with agent_id={:?}", event.agent_id);
        return Err(filtered_debug_entry(&event, "subagent_event"));
    }

    if is_observer_process(event.pid) {
        eprintln!("[claudeBoard] filtering out observer process pid={}", event.pid);
        return Err(filtered_debug_entry(&event, "observer_process"));
    }

    let event_type = match event.hook_event_name.as_str() {
        "SessionStart" | "UserPromptSubmit" | "PreToolUse" | "SubagentStart" | "TaskCreated"
        | "PreCompact" | "ElicitationResult" => HookEventType::TaskCreated,

        "PermissionRequest" | "Elicitation" => HookEventType::PermissionRequest,

        "PermissionDenied" => HookEventType::PermissionDenied,

        "TaskCompleted" | "Stop" => HookEventType::TaskCompleted,

        "PostToolUse" => return Err(filtered_debug_entry(&event, "post_tool_use_ignored")),
        "PostToolUseFailure" => {
            return Err(filtered_debug_entry(&event, "post_tool_use_failure_ignored"))
        }
        "StopFailure" => return Err(filtered_debug_entry(&event, "stop_failure_ignored")),
        "SubagentStop" => return Err(filtered_debug_entry(&event, "subagent_stop_ignored")),
        "SessionEnd" => HookEventType::SessionEnd,

        "PostConversationTurn" => match event.permission_mode.as_deref() {
            Some("acceptEdits") => HookEventType::PermissionRequest,
            Some("bypassPermissions") => {
                if event.prompt.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
                    eprintln!("[claudeBoard] PostConversationTurn in bypass mode - treating as completed");
                    HookEventType::TaskCompleted
                } else {
                    return Err(filtered_debug_entry(&event, "post_conversation_turn_with_prompt"));
                }
            }
            _ => return Err(filtered_debug_entry(&event, "post_conversation_turn_ignored")),
        },

        "Notification" => return Err(filtered_debug_entry(&event, "notification_event")),
        "ConfigChange" => return Err(filtered_debug_entry(&event, "config_change")),
        "WorktreeCreate" => return Err(filtered_debug_entry(&event, "worktree_create")),
        "WorktreeRemove" => return Err(filtered_debug_entry(&event, "worktree_remove")),
        "CwdChanged" => return Err(filtered_debug_entry(&event, "cwd_changed")),
        "FileChanged" => return Err(filtered_debug_entry(&event, "file_changed")),
        "Setup" => return Err(filtered_debug_entry(&event, "setup")),
        "InstructionsLoaded" => return Err(filtered_debug_entry(&event, "instructions_loaded")),
        "PostCompact" => return Err(filtered_debug_entry(&event, "post_compact")),

        _ => {
            eprintln!("[claudeBoard] unknown hook event type: {}", event.hook_event_name);
            return Err(filtered_debug_entry(&event, "unknown_event_type"));
        }
    };

    let title = extract_task_title(&event);
    let conversation_content = conversation_content(&event);
    let debug_entry = accepted_debug_entry(&event, event_type.clone());

    Ok((
        HookEvent {
            event_type,
            session_id: event.session_id,
            agent_id: event.agent_id,
            pid: event.pid,
            title,
            conversation_content,
            occurred_at: event.occurred_at,
        },
        debug_entry,
    ))
}

pub fn build_router_with_state_path<F, N>(
    store: Arc<Mutex<TaskStore>>,
    scan_rows: F,
    now: N,
    state_path: Option<PathBuf>,
) -> Router
where
    F: Fn() -> io::Result<Vec<String>> + Send + Sync + 'static,
    N: Fn() -> String + Send + Sync + 'static,
{
    let refresh_store = Arc::clone(&store);
    let refresh_state_path = state_path.clone();
    let scan_refresh = Arc::new(move || {
        eprintln!("[claudeBoard] refresh start source=router");
        let rows = scan_rows()?;
        eprintln!("[claudeBoard] refresh scan_rows source=router row_count={}", rows.len());

        let timestamp = now();
        eprintln!("[claudeBoard] refresh timestamp source=router value={timestamp}");
        let tasks = rebuild_tasks_from_rows(&rows, &timestamp);
        let alive_pids: Vec<u32> = tasks.iter().map(|task| task.pid).collect();
        let debug_entries = tasks
            .iter()
            .map(|task| crate::model::ScanDebugEntry {
                pid: Some(task.pid),
                ppid: None,
                state: None,
                command: task.title.clone(),
                decision: crate::model::ScanDecision::Accepted,
                reason: None,
                accepted_row: Some(task.task_id.clone()),
                task: Some(task.clone()),
            })
            .collect();
        eprintln!("[claudeBoard] refresh rebuilt source=router task_count={}", tasks.len());
        let mut store = refresh_store.lock().unwrap();
        store.replace_scanned_tasks_with_debug(
            tasks,
            &alive_pids,
            &timestamp,
            crate::model::ScanDebugSnapshot {
                occurred_at: timestamp.clone(),
                entries: debug_entries,
            },
        );
        if let Some(path) = &refresh_state_path {
            if let Err(error) = save_snapshot(path, &store.persisted_snapshot()) {
                eprintln!("[claudeBoard] failed to save session state after refresh: {error}");
            }
        }
        Ok(())
    });

    Router::new()
        .route("/events", post(post_event))
        .route("/refresh", post(post_refresh))
        .route("/snapshot", get(get_snapshot))
        .route("/debug/snapshot", get(get_debug_snapshot))
        .route("/tasks/:id/focus", post(post_focus_task))
        .route("/notifications/:id/ack", post(post_ack_notification))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(AppState {
            store,
            scan_refresh,
            state_path,
        })
}

pub fn build_router<F, N>(store: Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> Router
where
    F: Fn() -> io::Result<Vec<String>> + Send + Sync + 'static,
    N: Fn() -> String + Send + Sync + 'static,
{
    build_router_with_state_path(store, scan_rows, now, None)
}

pub fn refresh_scan<F, N>(store: &Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> io::Result<()>
where
    F: Fn() -> io::Result<Vec<String>>,
    N: Fn() -> String,
{
    eprintln!("[claudeBoard] refresh start source=function");
    let rows = scan_rows()?;
    eprintln!("[claudeBoard] refresh scan_rows source=function row_count={}", rows.len());

    let timestamp = now();
    eprintln!("[claudeBoard] refresh timestamp source=function value={timestamp}");
    let computation = compute_scan(
        &rows
            .iter()
            .map(|row| {
                let parts: Vec<&str> = row.split('\t').collect();
                format!("{} 0 S claude", parts.get(1).copied().unwrap_or("0"))
            })
            .collect::<Vec<_>>()
            .join("\n"),
        &timestamp,
    );
    let tasks = rebuild_tasks_from_rows(&computation.rows, &timestamp);
    eprintln!("[claudeBoard] refresh rebuilt source=function task_count={}", tasks.len());
    store
        .lock()
        .unwrap()
        .replace_scanned_tasks_with_debug(tasks, &computation.alive_pids, &timestamp, computation.debug);
    Ok(())
}

async fn post_event(
    State(state): State<AppState>,
    Json(event): Json<ClaudeCodeHookEvent>,
) -> StatusCode {
    eprintln!("[claudeBoard] received hook event: {} for session {}",
        event.hook_event_name, event.session_id);
    append_event_log(&format!(
        "hook_received name={} session={} pid={} agent_id={:?}",
        event.hook_event_name, event.session_id, event.pid, event.agent_id
    ));

    match convert_claude_code_event(event) {
        Ok((hook_event, debug_entry)) => {
            eprintln!("[claudeBoard] converted to: {:?} for pid {} title {}",
                hook_event.event_type, hook_event.pid, hook_event.title);
            let mut store = state.store.lock().unwrap();
            store.apply_debug(hook_event, debug_entry);
            if let Some(path) = &state.state_path {
                if let Err(error) = save_snapshot(path, &store.persisted_snapshot()) {
                    eprintln!("[claudeBoard] failed to save session state: {error}");
                }
            }
            StatusCode::ACCEPTED
        }
        Err(debug_entry) => {
            eprintln!("[claudeBoard] event filtered out");
            state.store.lock().unwrap().record_filtered_hook_event(debug_entry);
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

async fn post_ack_notification(
    State(state): State<AppState>,
    Path(notification_id): Path<u64>,
) -> StatusCode {
    let mut store = state.store.lock().unwrap();
    store.ack_notification(notification_id);
    if let Some(path) = &state.state_path {
        if let Err(error) = save_snapshot(path, &store.persisted_snapshot()) {
            eprintln!("[claudeBoard] failed to save session state after notification ack: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    StatusCode::ACCEPTED
}

async fn post_focus_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> StatusCode {
    let task = {
        let store = state.store.lock().unwrap();
        store
            .snapshot()
            .tasks
            .into_iter()
            .find(|task| task.task_id == task_id)
    };

    let Some(task) = task else {
        return StatusCode::NOT_FOUND;
    };

    if task.liveness == TaskLiveness::Dead {
        return StatusCode::NOT_FOUND;
    }

    let activator = ProcessHostActivator;
    let focused = resolve_focus(
        &activator,
        &FocusRequest {
            task_id: task.task_id,
            window_target: task.window_target,
        },
    );

    if focused {
        StatusCode::ACCEPTED
    } else {
        StatusCode::BAD_GATEWAY
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

async fn get_debug_snapshot(State(state): State<AppState>) -> Json<DebugSnapshot> {
    Json(state.store.lock().unwrap().debug_snapshot())
}

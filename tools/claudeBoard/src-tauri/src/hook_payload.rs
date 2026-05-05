use serde_json::Value;

use crate::model::{HookEvent, HookEventMetadata, HookEventType};

pub fn parse_hook_event_line(line: &str) -> Option<HookEvent> {
    let value = serde_json::from_str::<Value>(line).ok()?;
    normalize_hook_event_value(value)
}

pub fn normalize_hook_event_value(value: Value) -> Option<HookEvent> {
    serde_json::from_value::<HookEvent>(value.clone())
        .ok()
        .or_else(|| normalize_raw_payload(&value))
}

fn normalize_raw_payload(value: &Value) -> Option<HookEvent> {
    let event_type = match value.get("hook_event_name")?.as_str()? {
        "UserPromptSubmit" => HookEventType::UserPromptSubmit,
        "PreToolUse" => HookEventType::PreToolUse,
        "PostToolUse" => HookEventType::PostToolUse,
        "Stop" => HookEventType::Stop,
        "StopFailure" => HookEventType::StopFailure,
        "PermissionRequest" => HookEventType::PermissionRequest,
        "PermissionDenied" => HookEventType::PermissionDenied,
        "SessionStart" => HookEventType::SessionStart,
        "SessionEnd" => HookEventType::SessionEnd,
        _ => return None,
    };

    let session_id = value.get("session_id")?.as_str()?.to_string();
    let pid = value
        .get("claude_board_pid")
        .or_else(|| value.get("pid"))
        .and_then(Value::as_u64)
        .and_then(|pid| u32::try_from(pid).ok())?;
    let title = value
        .get("claude_board_title")
        .or_else(|| value.get("title"))
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("claude")
        .to_string();
    let occurred_at = value
        .get("claude_board_occurred_at")
        .or_else(|| value.get("occurred_at"))
        .and_then(Value::as_str)?
        .to_string();

    let metadata = HookEventMetadata {
        prompt: value.get("prompt").and_then(Value::as_str).map(ToOwned::to_owned),
        tool_name: value
            .get("tool_name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        last_assistant_message: value
            .get("last_assistant_message")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        transcript_path: value
            .get("transcript_path")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        cwd: value.get("cwd").and_then(Value::as_str).map(ToOwned::to_owned),
    };

    Some(HookEvent {
        event_type,
        session_id,
        pid,
        title,
        occurred_at,
        metadata,
    })
}

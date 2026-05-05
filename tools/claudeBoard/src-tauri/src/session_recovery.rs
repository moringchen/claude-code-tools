use serde_json::Value;

use crate::model::TaskStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveredSessionState {
    pub pid: u32,
    pub display_title: String,
    pub latest_prompt_summary: Option<String>,
    pub status: TaskStatus,
    pub completed_message: Option<String>,
    pub transcript_path: Option<String>,
    pub recovered_at: String,
    pub last_hook_event: Option<String>,
}

pub fn recover_session_state(
    pid: u32,
    fallback_title: &str,
    transcript_path: Option<&str>,
    transcript_contents: &str,
    recovered_at: &str,
) -> Option<RecoveredSessionState> {
    let mut latest_prompt_summary = None;
    let mut recovered_display_title = None;
    let mut fallback_prompt_summary = None;
    let mut pinned_prompt_summary = None;
    let mut has_custom_title = false;
    let mut status = TaskStatus::IdleOrUnknown;
    let mut completed_message = None;
    let mut last_message_type: Option<&str> = None;
    let mut last_assistant_content: Option<String> = None;
    let mut last_hook_event: Option<String> = None;

    for line in transcript_contents.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        match value.get("type").and_then(Value::as_str) {
            Some("user") => {
                if let Some(prompt) = extract_prompt_text(&value) {
                    let pin_prompt = value
                        .get("message")
                        .and_then(|message| message.get("content"))
                        .and_then(Value::as_str)
                        .map(|text| text.contains("<command-args>"))
                        .unwrap_or(false);
                    update_prompt_state(
                        &prompt,
                        pin_prompt,
                        &mut latest_prompt_summary,
                        &mut fallback_prompt_summary,
                        &mut pinned_prompt_summary,
                        &mut recovered_display_title,
                        has_custom_title,
                    );
                }
                status = TaskStatus::Running;
                last_message_type = Some("user");
                last_hook_event = Some("UserPromptSubmit".into());
            }
            Some("assistant") => {
                // Check if assistant message contains a question or options
                if let Some(content) = extract_assistant_text(&value) {
                    last_assistant_content = Some(content.clone());
                    // Only set to NeedsUser if last message was from assistant and it's asking something
                    if is_assistant_waiting_for_user(&content) {
                        status = TaskStatus::NeedsUser;
                    }
                }
                last_message_type = Some("assistant");
            }
            Some("last-prompt") => {
                if let Some(prompt) = value.get("lastPrompt").and_then(Value::as_str) {
                    update_prompt_state(
                        prompt,
                        true,
                        &mut latest_prompt_summary,
                        &mut fallback_prompt_summary,
                        &mut pinned_prompt_summary,
                        &mut recovered_display_title,
                        has_custom_title,
                    );
                }
            }
            Some("stop") => {
                status = TaskStatus::NeedsUser;
                last_hook_event = Some("Stop".into());
                completed_message = value
                    .get("last_assistant_message")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            Some("stop_failure") => {
                status = TaskStatus::NeedsUser;
                last_hook_event = Some("StopFailure".into());
            }
            Some("permission_request") => {
                status = TaskStatus::NeedsUser;
                last_hook_event = Some("PermissionRequest".into());
            }
            Some("custom-title") => {
                if let Some(title) = value.get("title").and_then(Value::as_str) {
                    has_custom_title = true;
                    recovered_display_title = Some(title.to_string());
                }
            }
            _ => {}
        }
    }

    // If last message was from assistant and no user response yet, check if it's waiting
    if last_message_type == Some("assistant") {
        if let Some(content) = last_assistant_content {
            if is_assistant_waiting_for_user(&content) {
                status = TaskStatus::NeedsUser;
            }
        }
    }

    let latest_prompt_summary = pinned_prompt_summary
        .or(latest_prompt_summary)
        .or(fallback_prompt_summary);

    Some(RecoveredSessionState {
        pid,
        display_title: recovered_display_title.unwrap_or_else(|| fallback_title.to_string()),
        latest_prompt_summary,
        status,
        completed_message,
        transcript_path: transcript_path.map(ToOwned::to_owned),
        recovered_at: recovered_at.to_string(),
        last_hook_event,
    })
}

fn update_prompt_state(
    prompt: &str,
    pin_prompt: bool,
    latest_prompt_summary: &mut Option<String>,
    fallback_prompt_summary: &mut Option<String>,
    pinned_prompt_summary: &mut Option<String>,
    recovered_display_title: &mut Option<String>,
    has_custom_title: bool,
) {
    let Some(prompt) = normalize_prompt_text(prompt) else {
        return;
    };
    let is_injected_prompt_noise = is_injected_prompt_noise(&prompt);

    if !is_injected_prompt_noise && fallback_prompt_summary.is_none() {
        *fallback_prompt_summary = Some(prompt.clone());
    }

    if is_meaningful_prompt(&prompt) {
        if pin_prompt && pinned_prompt_summary.is_none() {
            *pinned_prompt_summary = Some(prompt.clone());
        }
        if pinned_prompt_summary.is_none() {
            *latest_prompt_summary = Some(prompt.clone());
        }
        if !has_custom_title {
            if pin_prompt {
                if recovered_display_title.is_none() {
                    *recovered_display_title = Some(prompt);
                }
            } else if pinned_prompt_summary.is_none() {
                *recovered_display_title = Some(prompt);
            }
        }
    } else if !is_injected_prompt_noise && !has_custom_title && recovered_display_title.is_none() {
        *recovered_display_title = Some(prompt);
    }
}

fn extract_prompt_text(value: &Value) -> Option<String> {
    let content = value.get("message")?.get("content")?;

    match content {
        Value::String(text) => normalize_prompt_text(text),
        Value::Array(items) => items
            .iter()
            .find_map(|item| {
                if item.get("type").and_then(Value::as_str) == Some("text") {
                    item.get("text").and_then(Value::as_str)
                } else {
                    None
                }
            })
            .and_then(normalize_prompt_text),
        _ => None,
    }
}

fn extract_assistant_text(value: &Value) -> Option<String> {
    let content = value.get("message")?.get("content")?;

    match content {
        Value::String(text) => Some(text.trim().to_string()),
        Value::Array(items) => {
            let text: String = items
                .iter()
                .filter_map(|item| {
                    if item.get("type").and_then(Value::as_str) == Some("text") {
                        item.get("text").and_then(Value::as_str)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            if text.is_empty() {
                None
            } else {
                Some(text.trim().to_string())
            }
        }
        _ => None,
    }
}

fn is_assistant_waiting_for_user(content: &str) -> bool {
    let content_lower = content.to_lowercase();

    // Check for multiple choice options (A/B/C/D or 1/2/3/4)
    // Support both "A." and "A)" formats, with or without space after
    let has_multiple_choice =
        content_lower.contains("a.") && content_lower.contains("b.") && content_lower.contains("c.") ||
        content_lower.contains("a)") && content_lower.contains("b)") && content_lower.contains("c)") ||
        content_lower.contains("1.") && content_lower.contains("2.") && content_lower.contains("3.") ||
        content_lower.contains("1)") && content_lower.contains("2)") && content_lower.contains("3)") ||
        content_lower.contains("a.") && content_lower.contains("b.") ||
        content_lower.contains("1.") && content_lower.contains("2.");

    // Check for question patterns
    let has_question =
        content_lower.contains("你希望") ||
        content_lower.contains("请") ||
        content_lower.contains("?") ||
        content_lower.contains("？") ||
        content_lower.contains("选择") ||
        content_lower.contains("哪个") ||
        content_lower.contains("什么") ||
        content_lower.contains("如何") ||
        content_lower.contains("是否") ||
        content_lower.contains("怎么") ||
        content_lower.contains("建议") ||
        content_lower.contains("推荐") ||
        content_lower.contains("想要") ||
        content_lower.contains("需要") ||
        content_lower.contains("请确认") ||
        content_lower.contains("请选择一个") ||
        content_lower.contains("请告诉我") ||
        content_lower.contains("请回复") ||
        content_lower.contains("请回答") ||
        content_lower.contains("第一个问题") ||
        content_lower.contains("第二个问题") ||
        content_lower.contains("可以吗") ||
        content_lower.contains("对吗") ||
        content_lower.contains("好吗") ||
        content_lower.contains("行吗");

    // Check for visual companion pattern
    let is_visual_companion =
        content_lower.contains("视觉伴侣") ||
        content_lower.contains("visual companion") ||
        content_lower.contains("浏览器") ||
        content_lower.contains("localhost") ||
        content_lower.contains("http://localhost");

    // Check for waiting patterns
    let is_waiting =
        content_lower.contains("等待") ||
        content_lower.contains("waiting") ||
        content_lower.contains("让我知道") ||
        content_lower.contains("告诉我") ||
        content_lower.contains("请回复") ||
        content_lower.contains("请回答") ||
        content_lower.contains("请提供");

    // More lenient detection: if it has multiple choice OR explicit question patterns, mark as waiting
    has_multiple_choice || has_question || is_visual_companion || is_waiting
}

fn normalize_prompt_text(text: &str) -> Option<String> {
    let normalized = if let Some(command_args) = extract_tagged_block(text, "command-args") {
        command_args
    } else {
        text.trim().replace('\n', " ")
    };

    let normalized = strip_command_prefix(&normalized).trim().to_string();
    (!normalized.is_empty()).then_some(normalized)
}

fn strip_command_prefix(text: &str) -> &str {
    let trimmed = text.trim();
    let Some(rest) = trimmed.strip_prefix('/') else {
        return trimmed;
    };
    let Some(space_index) = rest.find(' ') else {
        return trimmed;
    };

    rest[space_index + 1..].trim_start()
}

fn extract_tagged_block(text: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = text.find(&start_tag)? + start_tag.len();
    let end = text[start..].find(&end_tag)? + start;
    Some(text[start..end].trim().replace('\n', " "))
}

fn is_injected_prompt_noise(prompt: &str) -> bool {
    const INJECTED_PROMPT_MARKERS: [&str; 6] = [
        "Base directory for this skill:",
        "<EXTREMELY-IMPORTANT>",
        "<SUBAGENT-STOP>",
        "## Instruction Priority",
        "## How to Access Skills",
        "ARGUMENTS:",
    ];

    INJECTED_PROMPT_MARKERS
        .iter()
        .any(|marker| prompt.contains(marker))
}

pub fn is_meaningful_prompt(prompt: &str) -> bool {
    let trimmed = prompt.trim();
    if is_injected_prompt_noise(trimmed) {
        return false;
    }
    if trimmed.chars().count() > 8 {
        return true;
    }

    !matches!(
        trimmed,
        "继续" | "好的继续" | "确认" | "好" | "可以" | "好了吗" | "1" | "2" | "3"
    )
}

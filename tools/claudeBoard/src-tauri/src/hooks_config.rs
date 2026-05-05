use serde_json::{json, Map, Value};

const HOOK_EVENTS: [&str; 11] = [
    "TaskCreated",
    "TaskCompleted",
    "PermissionRequest",
    "PermissionDenied",
    "SessionEnd",
    "UserPromptSubmit",
    "PreToolUse",
    "PostConversationTurn",
    "PreUserInteraction",
    "Stop",
    "StopFailure",
];

pub fn upsert_hooks(settings_json: &str, dispatch_command: &str) -> serde_json::Result<String> {
    let mut settings: Value = serde_json::from_str(settings_json)?;

    if !settings.is_object() {
        settings = json!({});
    }

    let root = settings.as_object_mut().unwrap();
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    if !hooks.is_object() {
        *hooks = Value::Object(Map::new());
    }

    let hooks_map = hooks.as_object_mut().unwrap();

    for event_name in HOOK_EVENTS {
        let event_value = hooks_map
            .entry(event_name.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));

        if !event_value.is_array() {
            *event_value = Value::Array(Vec::new());
        }

        let event_entries = event_value.as_array_mut().unwrap();
        let matcher_index = event_entries
            .iter()
            .position(|entry| entry.get("matcher") == Some(&Value::String("*".to_string())));

        let matcher_entry = if let Some(index) = matcher_index {
            &mut event_entries[index]
        } else {
            event_entries.push(json!({
                "matcher": "*",
                "hooks": []
            }));
            event_entries.last_mut().unwrap()
        };

        if !matcher_entry.is_object() {
            *matcher_entry = json!({
                "matcher": "*",
                "hooks": []
            });
        }

        let matcher_object = matcher_entry.as_object_mut().unwrap();
        matcher_object.insert("matcher".to_string(), Value::String("*".to_string()));

        let hooks_value = matcher_object
            .entry("hooks".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));

        if !hooks_value.is_array() {
            *hooks_value = Value::Array(Vec::new());
        }

        let hook_entries = hooks_value.as_array_mut().unwrap();
        let has_dispatch_hook = hook_entries.iter().any(|hook| {
            hook.get("type") == Some(&Value::String("command".to_string()))
                && hook.get("command") == Some(&Value::String(dispatch_command.to_string()))
        });

        if !has_dispatch_hook {
            hook_entries.push(json!({
                "type": "command",
                "command": dispatch_command
            }));
        }
    }

    Ok(serde_json::to_string_pretty(&settings).unwrap())
}

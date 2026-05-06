use claude_board::hooks_config::upsert_hooks;
use serde_json::{json, Value};

fn parse_output(output: serde_json::Result<String>) -> Value {
    serde_json::from_str(&output.expect("hook config should update")).expect("output should be valid json")
}

fn event_hooks<'a>(settings: &'a Value, event_name: &str) -> &'a [Value] {
    settings
        .get("hooks")
        .and_then(|hooks| hooks.get(event_name))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .expect("event hooks should exist")
}

#[test]
fn inserts_required_claude_code_hooks_into_empty_settings() {
    let dispatch_command = "~/.claude-board/hook-dispatch.sh";
    let output = upsert_hooks(r#"{"hooks":{}}"#, dispatch_command);
    let settings = parse_output(output);

    for event_name in [
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
        "SubagentStart",
        "SubagentStop",
    ] {
        assert_eq!(
            event_hooks(&settings, event_name),
            [json!({
                "matcher": "*",
                "hooks": [
                    {
                        "type": "command",
                        "command": dispatch_command
                    }
                ]
            })]
        );
    }
}

#[test]
fn preserves_existing_hooks_and_appends_dispatch_hook_once() {
    let dispatch_command = "~/.claude-board/hook-dispatch.sh";
    let input = r#"
    {
      "keep": true,
      "hooks": {
        "TaskCreated": [
          {
            "matcher": "*",
            "hooks": [
              { "type": "command", "command": "existing-created" }
            ]
          }
        ],
        "PermissionDenied": [
          {
            "matcher": "*",
            "hooks": [
              { "type": "command", "command": "existing-denied" }
            ]
          }
        ],
        "UnrelatedEvent": [
          {
            "matcher": "custom",
            "hooks": [
              { "type": "command", "command": "leave-me-alone" }
            ]
          }
        ]
      }
    }
    "#;

    let output = upsert_hooks(input, dispatch_command);
    let settings = parse_output(output);

    assert_eq!(settings.get("keep"), Some(&Value::Bool(true)));
    assert_eq!(
        settings
            .get("hooks")
            .and_then(|hooks| hooks.get("UnrelatedEvent"))
            .cloned(),
        Some(json!([
            {
                "matcher": "custom",
                "hooks": [
                    { "type": "command", "command": "leave-me-alone" }
                ]
            }
        ]))
    );

    assert_eq!(
        event_hooks(&settings, "TaskCreated"),
        [json!({
            "matcher": "*",
            "hooks": [
                { "type": "command", "command": "existing-created" },
                { "type": "command", "command": dispatch_command }
            ]
        })]
    );
    assert_eq!(
        event_hooks(&settings, "PermissionDenied"),
        [json!({
            "matcher": "*",
            "hooks": [
                { "type": "command", "command": "existing-denied" },
                { "type": "command", "command": dispatch_command }
            ]
        })]
    );
}

#[test]
fn repeated_upsert_does_not_duplicate_dispatch_hook() {
    let dispatch_command = "~/.claude-board/hook-dispatch.sh";
    let once = upsert_hooks(r#"{"hooks":{}}"#, dispatch_command);
    let twice = upsert_hooks(&once.expect("first update should work"), dispatch_command);
    let settings = parse_output(twice);

    for event_name in [
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
        "SubagentStart",
        "SubagentStop",
    ] {
        assert_eq!(event_hooks(&settings, event_name).len(), 1);
        let hooks = event_hooks(&settings, event_name)[0]
            .get("hooks")
            .and_then(Value::as_array)
            .expect("hooks array should exist");
        assert_eq!(hooks.len(), 1);
        assert_eq!(
            hooks[0],
            json!({
                "type": "command",
                "command": dispatch_command
            })
        );
    }
}

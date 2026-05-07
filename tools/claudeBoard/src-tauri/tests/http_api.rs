use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use claude_board::{
    model::{DebugSnapshot, TaskSnapshot, TaskStatus},
    server::{build_router, build_router_with_state_path},
    store::TaskStore,
};
use tower::ServiceExt;

#[tokio::test]
async fn debug_snapshot_includes_scan_and_hook_diagnostics() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        store,
        || Ok(vec!["local-9301\t9301\tworkspace\tTerminal\tterminal\t\t\tDebug scan task".to_string()]),
        || "2026-05-06T15:00:00Z".to_string(),
    );

    let refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::ACCEPTED);

    let event = serde_json::json!({
        "hook_event_name": "PermissionRequest",
        "session_id": "session-debug",
        "claude_board_pid": 9302,
        "claude_board_title": "Approve debug command",
        "claude_board_occurred_at": "2026-05-06T15:01:00Z",
        "cwd": "/workspace",
        "prompt": "show me debug info"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/debug/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(snapshot.status(), StatusCode::OK);
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: DebugSnapshot = serde_json::from_slice(&body).unwrap();

    assert!(snapshot.snapshot.tasks.iter().any(|task| task.session_id == "session-debug"));
    assert!(snapshot.snapshot.tasks.iter().any(|task| task.session_id == "local-9301"));
    assert!(snapshot.recent_hook_events.iter().any(|entry| {
        entry.session_id == "session-debug"
            && entry.hook_event_name == "PermissionRequest"
            && entry.mapped_event_type == Some(claude_board::model::HookEventType::PermissionRequest)
    }));
    assert!(snapshot.latest_scan.entries.iter().any(|entry| {
        entry.pid == Some(9301) && entry.task.as_ref().map(|task| task.session_id.as_str()) == Some("local-9301")
    }));
}

#[tokio::test]
async fn focus_endpoint_returns_not_found_for_unknown_task() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-07T12:00:00Z".to_string());

    let response = app
        .oneshot(Request::post("/tasks/missing/focus").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn focus_endpoint_returns_accepted_for_live_task() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        Arc::clone(&store),
        || Ok(Vec::new()),
        || "2026-05-07T12:05:00Z".to_string(),
    );

    store.lock().unwrap().apply(claude_board::model::HookEvent {
        event_type: claude_board::model::HookEventType::TaskCreated,
        session_id: "session-focus-live".into(),
        agent_id: None,
        pid: 780,
        title: "Focus me".into(),
        conversation_content: Some("Focus me".into()),
        occurred_at: "2026-05-07T12:05:00Z".into(),
    });
    store.lock().unwrap().replace_scanned_tasks(
        vec![claude_board::model::TaskCard {
            task_id: "scan:session-focus-live:780".into(),
            session_id: "session-focus-live".into(),
            pid: 780,
            title: "Focus me".into(),
            status: TaskStatus::IdleOrUnknown,
            source: "scan_recovered".into(),
            window_target: claude_board::model::WindowTarget {
                host_kind: "terminal".into(),
                app: "Finder".into(),
                descriptor: "finder".into(),
                tab_id: None,
                pane_id: None,
            },
            started_at: "2026-05-07T12:05:00Z".into(),
            updated_at: "2026-05-07T12:05:00Z".into(),
            completed_at: None,
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[780],
        "2026-05-07T12:05:01Z",
    );

    let response = app
        .oneshot(Request::post("/tasks/session-focus-live/focus").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn post_event_does_not_require_backend_sound_playback_to_queue_notifications() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-05T16:10:00Z".to_string());

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "hook_event_name": "PermissionRequest",
                        "session_id": "session-no-backend-sound",
                        "claude_board_pid": 9500,
                        "claude_board_title": "Approve command",
                        "claude_board_occurred_at": "2026-05-05T16:10:00Z",
                        "cwd": "/workspace"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.notifications.len(), 1);
    assert_eq!(snapshot.notifications[0].sound_type, claude_board::model::NotificationSoundType::Waiting);
}

#[tokio::test]
async fn ingests_events_and_returns_snapshot() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-04-24T17:00:00Z".to_string());

    // Use ClaudeCodeHookEvent format (matching what Claude Code hooks send)
    let create_event = serde_json::json!({
        "hook_event_name": "TaskCreated",
        "session_id": "session-1",
        "claude_board_pid": 321,
        "claude_board_title": "Review PR",
        "claude_board_occurred_at": "2026-04-24T17:00:00Z",
        "cwd": "/workspace"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(snapshot.status(), StatusCode::OK);
    assert_eq!(
        snapshot.headers().get("access-control-allow-origin").unwrap(),
        "*"
    );
}

#[tokio::test]
async fn finished_task_stays_visible_as_completed_while_process_is_still_alive() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        store,
        || Ok(vec!["task-1\t321\tS\tTerminal\tterminal\t\t\tReview PR".to_string()]),
        || "2026-04-24T17:00:00Z".to_string(),
    );

    for event in [
        serde_json::json!({
            "hook_event_name": "TaskCreated",
            "session_id": "session-1",
            "claude_board_pid": 321,
            "claude_board_title": "Review PR",
            "claude_board_occurred_at": "2026-04-24T17:00:00Z",
            "cwd": "/workspace"
        }),
        serde_json::json!({
            "hook_event_name": "TaskCompleted",
            "session_id": "session-1",
            "claude_board_pid": 321,
            "claude_board_title": "Review PR",
            "claude_board_occurred_at": "2026-04-24T17:01:00Z",
            "cwd": "/workspace"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.completed, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-1");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
}

#[tokio::test]
async fn stop_event_settles_session_to_completed_and_later_user_prompt_reopens_it() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-06T19:00:00Z".to_string());

    for event in [
        serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "session-stop-reopen",
            "claude_board_pid": 9700,
            "claude_board_title": "Write reply",
            "claude_board_occurred_at": "2026-05-06T19:00:00Z",
            "cwd": "/workspace",
            "prompt": "first turn"
        }),
        serde_json::json!({
            "hook_event_name": "Stop",
            "session_id": "session-stop-reopen",
            "claude_board_pid": 9700,
            "claude_board_title": "Write reply",
            "claude_board_occurred_at": "2026-05-06T19:00:10Z",
            "cwd": "/workspace"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let completed_snapshot = app
        .clone()
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let completed_body = axum::body::to_bytes(completed_snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let completed_snapshot: TaskSnapshot = serde_json::from_slice(&completed_body).unwrap();

    assert_eq!(completed_snapshot.tasks.len(), 1);
    assert_eq!(completed_snapshot.tasks[0].status, TaskStatus::Completed);

    let reopen_response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "hook_event_name": "UserPromptSubmit",
                        "session_id": "session-stop-reopen",
                        "claude_board_pid": 9700,
                        "claude_board_title": "Write reply",
                        "claude_board_occurred_at": "2026-05-06T19:01:00Z",
                        "cwd": "/workspace",
                        "prompt": "second turn"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reopen_response.status(), StatusCode::ACCEPTED);

    let reopened_snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let reopened_body = axum::body::to_bytes(reopened_snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let reopened_snapshot: TaskSnapshot = serde_json::from_slice(&reopened_body).unwrap();

    assert_eq!(reopened_snapshot.tasks.len(), 1);
    assert_eq!(reopened_snapshot.tasks[0].status, TaskStatus::Running);
}

#[tokio::test]
async fn permission_denied_event_settles_task_to_completed_in_snapshot() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-06T18:20:00Z".to_string());

    for event in [
        serde_json::json!({
            "hook_event_name": "PermissionRequest",
            "session_id": "session-denied",
            "claude_board_pid": 9600,
            "claude_board_title": "Approve command",
            "claude_board_occurred_at": "2026-05-06T18:20:00Z",
            "cwd": "/workspace"
        }),
        serde_json::json!({
            "hook_event_name": "PermissionDenied",
            "session_id": "session-denied",
            "claude_board_pid": 9600,
            "claude_board_title": "Approve command",
            "claude_board_occurred_at": "2026-05-06T18:21:00Z",
            "cwd": "/workspace"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
    assert_eq!(snapshot.notifications.len(), 2);
    assert_eq!(snapshot.notifications[0].sound_type, claude_board::model::NotificationSoundType::Waiting);
    assert_eq!(snapshot.notifications[1].sound_type, claude_board::model::NotificationSoundType::Completed);
}


#[tokio::test]
async fn maps_claude_code_hook_events_to_task_statuses() {
    let events = [
        ("SessionStart", Some(TaskStatus::Running)),
        ("UserPromptSubmit", Some(TaskStatus::Running)),
        ("PreToolUse", Some(TaskStatus::Running)),
        ("SubagentStart", Some(TaskStatus::Running)),
        ("TaskCreated", Some(TaskStatus::Running)),
        ("PreCompact", Some(TaskStatus::Running)),
        ("PermissionDenied", Some(TaskStatus::Completed)),
        ("ElicitationResult", Some(TaskStatus::Running)),
        ("PermissionRequest", Some(TaskStatus::NeedsUser)),
        ("Elicitation", Some(TaskStatus::NeedsUser)),
        ("PostToolUse", None),
        ("PostToolUseFailure", None),
        ("SubagentStop", None),
        ("TaskCompleted", Some(TaskStatus::Completed)),
        ("Stop", Some(TaskStatus::Completed)),
        ("StopFailure", None),
        ("SessionEnd", None),
    ];

    for (index, (hook_event_name, expected_status)) in events.into_iter().enumerate() {
        let store = Arc::new(Mutex::new(TaskStore::default()));
        let app = build_router(store, || Ok(Vec::new()), || "2026-04-24T18:20:00Z".to_string());
        let event = serde_json::json!({
            "hook_event_name": hook_event_name,
            "session_id": format!("session-{index}"),
            "claude_board_pid": 9000 + index,
            "claude_board_title": hook_event_name,
            "claude_board_occurred_at": "2026-04-24T18:20:00Z",
            "cwd": "/workspace",
            "agent_id": if hook_event_name == "SubagentStop" { Some("agent-1") } else { None }
        });

        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED, "{hook_event_name}");

        let snapshot = app
            .clone()
            .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(snapshot.status(), StatusCode::OK, "{hook_event_name}");
        let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
            .await
            .unwrap();
        let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

        match expected_status {
            Some(expected_status) => {
                let task = snapshot
                    .tasks
                    .iter()
                    .find(|task| task.session_id == format!("session-{index}"))
                    .unwrap_or_else(|| panic!("missing task for {hook_event_name}"));

                assert_eq!(task.status, expected_status, "{hook_event_name}");
            }
            None => {
                assert!(
                    snapshot
                        .tasks
                        .iter()
                        .all(|task| task.session_id != format!("session-{index}")),
                    "terminal task should be removed for {hook_event_name}"
                );
                assert_eq!(snapshot.counts.completed, 0, "{hook_event_name}");
            }
        }
    }
}


#[tokio::test]
async fn ignored_hook_events_do_not_change_visible_task_state() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-06T19:10:00Z".to_string());

    let create = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "session-ignored-events",
        "claude_board_pid": 9800,
        "claude_board_title": "Write reply",
        "claude_board_occurred_at": "2026-05-06T19:10:00Z",
        "cwd": "/workspace",
        "prompt": "first turn"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&create).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    for (index, hook_event_name) in ["PostToolUse", "PostToolUseFailure", "StopFailure", "SubagentStop"]
        .into_iter()
        .enumerate()
    {
        let event = serde_json::json!({
            "hook_event_name": hook_event_name,
            "session_id": "session-ignored-events",
            "claude_board_pid": 9800,
            "claude_board_title": "Write reply",
            "claude_board_occurred_at": format!("2026-05-06T19:10:0{}Z", index + 1),
            "cwd": "/workspace",
            "agent_id": if hook_event_name == "SubagentStop" { Some("agent-1") } else { None }
        });

        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED, "{hook_event_name}");
    }

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-ignored-events");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Running);
}

#[tokio::test]
async fn ignored_hook_events_do_not_persist_snapshot_changes() {
    let temp_path = std::env::temp_dir().join(format!(
        "claude-board-ignored-events-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&temp_path);

    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router_with_state_path(
        Arc::clone(&store),
        || Ok(Vec::new()),
        || "2026-05-06T19:20:00Z".to_string(),
        Some(temp_path.clone()),
    );

    for event in [
        serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "session-ignore-persist",
            "claude_board_pid": 9810,
            "claude_board_title": "Write reply",
            "claude_board_occurred_at": "2026-05-06T19:20:00Z",
            "cwd": "/workspace",
            "prompt": "first turn"
        }),
        serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": "session-ignore-persist",
            "claude_board_pid": 9810,
            "claude_board_title": "Write reply",
            "claude_board_occurred_at": "2026-05-06T19:20:01Z",
            "cwd": "/workspace"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let persisted = std::fs::read_to_string(&temp_path).unwrap();
    let snapshot: TaskSnapshot = serde_json::from_str(&persisted).unwrap();
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Running);

    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn event_log_is_reset_before_startup_separator() {
    let log_path = std::env::temp_dir().join(format!(
        "claude-board-events-log-reset-{}.log",
        std::process::id()
    ));
    std::fs::write(&log_path, "old line\n").unwrap();

    std::fs::remove_file(&log_path).unwrap();
    claude_board::event_log::append_event_log_to_path(
        &log_path,
        "----- daemon_start -----",
        Some("2026-05-06T18:31:00+08:00"),
    )
    .unwrap();

    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(contents, "2026-05-06T18:31:00+08:00 ----- daemon_start -----\n");

    let _ = std::fs::remove_file(&log_path);
}

#[test]
fn event_log_writes_startup_separator() {
    let log_path = std::env::temp_dir().join(format!(
        "claude-board-events-log-separator-{}.log",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&log_path);

    claude_board::event_log::append_event_log_to_path(
        &log_path,
        "----- daemon_start -----",
        Some("2026-05-06T18:30:00+08:00"),
    )
    .unwrap();

    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert_eq!(contents, "2026-05-06T18:30:00+08:00 ----- daemon_start -----\n");

    let _ = std::fs::remove_file(&log_path);
}

#[test]
fn event_log_uses_local_timezone_offset() {
    let log_path = std::env::temp_dir().join(format!(
        "claude-board-events-log-timezone-{}.log",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&log_path);

    claude_board::event_log::append_event_log_to_path(
        &log_path,
        "hook_received name=Stop session=session-1 pid=123 agent_id=None",
        Some("2026-05-06T18:00:00+08:00"),
    )
    .unwrap();

    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.starts_with("2026-05-06T18:00:00+08:00 hook_received"));

    let _ = std::fs::remove_file(&log_path);
}

#[test]
fn event_log_rotates_after_exceeding_five_megabytes() {
    let log_path = std::env::temp_dir().join(format!(
        "claude-board-events-log-{}.log",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&log_path);

    let oversized = "x".repeat(5 * 1024 * 1024 + 32);
    std::fs::write(&log_path, oversized).unwrap();

    claude_board::event_log::append_event_log_to_path(&log_path, "fresh line", None).unwrap();

    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.ends_with(" fresh line\n"));

    let _ = std::fs::remove_file(&log_path);
}

#[tokio::test]
async fn terminal_events_remove_session_records_when_tasks_close() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-04-24T18:25:00Z".to_string());

    for event in [
        serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "session-content",
            "claude_board_pid": 9100,
            "claude_board_title": "workspace",
            "claude_board_occurred_at": "2026-04-24T18:25:00Z",
            "cwd": "/workspace",
            "prompt": "设计一个贪吃蛇游戏"
        }),
        serde_json::json!({
            "hook_event_name": "Stop",
            "session_id": "session-content",
            "claude_board_pid": 9100,
            "claude_board_title": "workspace",
            "claude_board_occurred_at": "2026-04-24T18:26:00Z",
            "cwd": "/workspace"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-content");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
}

#[tokio::test]
async fn ack_removes_pending_notification_and_repeat_ack_is_harmless() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store.clone(), || Ok(Vec::new()), || "2026-05-05T16:00:00Z".to_string());

    let event = serde_json::json!({
        "hook_event_name": "PermissionRequest",
        "session_id": "session-ack",
        "claude_board_pid": 9400,
        "claude_board_title": "Approve command",
        "claude_board_occurred_at": "2026-05-05T16:00:00Z",
        "cwd": "/workspace"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .clone()
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.notifications.len(), 1);
    let notification_id = snapshot.notifications[0].id;

    let ack_response = app
        .clone()
        .oneshot(
            Request::post(format!("/notifications/{notification_id}/ack"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ack_response.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .clone()
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert!(snapshot.notifications.is_empty());
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::NeedsUser);

    let repeated_ack_response = app
        .oneshot(
            Request::post(format!("/notifications/{notification_id}/ack"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(repeated_ack_response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn refresh_scan_uses_current_timestamp_for_store_updates() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        store.clone(),
        || Ok(vec!["task-1\t9300\tS\tTerminal\tterminal\t\t\tRefresh task".to_string()]),
        || "2026-04-24T18:40:00Z".to_string(),
    );

    let response = app
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = store.lock().unwrap().snapshot();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.task_id == "scan:task-1:9300")
        .unwrap();

    assert_eq!(task.updated_at, "2026-04-24T18:40:00Z");
}

#[tokio::test]
async fn subagent_stop_does_not_complete_parent_session_task() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-04-24T18:30:00Z".to_string());

    for event in [
        serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "session-parent",
            "claude_board_pid": 9200,
            "claude_board_title": "Parent task",
            "claude_board_occurred_at": "2026-04-24T18:30:00Z",
            "cwd": "/workspace",
            "prompt": "Parent task"
        }),
        serde_json::json!({
            "hook_event_name": "SubagentStart",
            "session_id": "session-parent",
            "claude_board_pid": 9201,
            "claude_board_title": "Subagent task",
            "claude_board_occurred_at": "2026-04-24T18:31:00Z",
            "cwd": "/workspace",
            "agent_id": "agent-1"
        }),
        serde_json::json!({
            "hook_event_name": "SubagentStop",
            "session_id": "session-parent",
            "claude_board_pid": 9201,
            "claude_board_title": "Subagent task",
            "claude_board_occurred_at": "2026-04-24T18:32:00Z",
            "cwd": "/workspace",
            "agent_id": "agent-1"
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.task_id == "session-parent" && task.status == TaskStatus::Running));
}

#[tokio::test]
async fn refresh_does_not_persist_removed_dead_hook_tasks() {
    let temp_path = std::env::temp_dir().join(format!(
        "claude-board-refresh-prune-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&temp_path);

    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router_with_state_path(
        Arc::clone(&store),
        || Ok(Vec::new()),
        || "2026-04-24T19:00:00Z".to_string(),
        Some(temp_path.clone()),
    );

    store.lock().unwrap().apply(claude_board::model::HookEvent {
        event_type: claude_board::model::HookEventType::TaskCreated,
        session_id: "session-dead-completed".into(),
        agent_id: None,
        pid: 9300,
        title: "Closed terminal completed task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T18:58:00Z".into(),
    });
    store.lock().unwrap().apply(claude_board::model::HookEvent {
        event_type: claude_board::model::HookEventType::TaskCompleted,
        session_id: "session-dead-completed".into(),
        agent_id: None,
        pid: 9300,
        title: "Closed terminal completed task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T18:59:00Z".into(),
    });

    let response = app
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let saved = std::fs::read_to_string(&temp_path).unwrap();
    let snapshot: TaskSnapshot = serde_json::from_str(&saved).unwrap();
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-dead-completed" && task.liveness == claude_board::model::TaskLiveness::Dead));
    assert!(snapshot
        .sessions
        .iter()
        .any(|record| record.session_id == "session-dead-completed"));

    let _ = std::fs::remove_file(temp_path);
}

#[tokio::test]
async fn refresh_merges_scanned_tasks_and_preserves_hook_precedence() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        store,
        || {
            Ok(vec![
                "session-1\t321\tworkspace\tGhostty\tterminal\t\t\tReview PR".to_string(),
                "session-2\t654\tworkspace\tGhostty\tterminal\t\t\tDraft spec".to_string(),
            ])
        },
        || "2026-04-24T18:00:00Z".to_string(),
    );

    // Use ClaudeCodeHookEvent format (matching what Claude Code hooks send)
    let create_event = serde_json::json!({
        "hook_event_name": "TaskCreated",
        "session_id": "session-1",
        "claude_board_pid": 321,
        "claude_board_title": "Review PR",
        "claude_board_occurred_at": "2026-04-24T17:00:00Z",
        "cwd": "/workspace"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(refresh.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(snapshot.status(), StatusCode::OK);

    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.counts.total, 2);
    // hook task session-1 is Running; unrelated scanned-only rows remain visible.
    assert_eq!(snapshot.counts.running, 1);
    assert_eq!(snapshot.counts.completed, 0);
    assert_eq!(snapshot.tasks.len(), 2);
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-1" && task.pid == 321 && task.source == "hook"));
    assert_eq!(
        snapshot
            .tasks
            .iter()
            .filter(|task| task.session_id == "session-1" && task.pid == 321)
            .count(),
        1
    );
}

#[tokio::test]
async fn refresh_exposes_windows_scan_rows_in_snapshot() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(
        store,
        || {
            Ok(vec![
                "session-win\t903\tworkspace\tTerminal\tterminal\t\t\tclaude.exe code".to_string(),
            ])
        },
        || "2026-04-24T18:05:00Z".to_string(),
    );

    let refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(refresh.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(snapshot.status(), StatusCode::OK);

    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert!(snapshot.tasks.iter().any(|task| {
        task.session_id == "session-win"
            && task.pid == 903
            && task.source == "scan_recovered"
            && task.title == "claude.exe code"
    }));
}

#[tokio::test]
async fn refresh_uses_fresh_timestamps_for_scanned_tasks() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let sequence = Arc::new(AtomicUsize::new(0));
    let app = build_router(
        store,
        || {
            Ok(vec![
                "session-time\t904\tworkspace\tTerminal\tterminal\t\t\tclaude.exe review".to_string(),
            ])
        },
        {
            let sequence = Arc::clone(&sequence);
            move || match sequence.fetch_add(1, Ordering::SeqCst) {
                0 => "2026-04-24T18:10:00Z".to_string(),
                _ => "2026-04-24T18:11:00Z".to_string(),
            }
        },
    );

    let first_refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(first_refresh.status(), StatusCode::ACCEPTED);

    let second_refresh = app
        .clone()
        .oneshot(Request::post("/refresh").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(second_refresh.status(), StatusCode::ACCEPTED);

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(snapshot.status(), StatusCode::OK);

    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX)
        .await
        .unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.session_id == "session-time" && task.pid == 904)
        .unwrap();

    assert_eq!(task.started_at, "2026-04-24T18:11:00Z");
    assert_eq!(task.updated_at, "2026-04-24T18:11:00Z");
}

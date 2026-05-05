use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use claude_board::{
    model::{HookEvent, HookEventType, TaskSnapshot},
    server::build_router,
    store::TaskStore,
};
use tower::ServiceExt;

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

    assert_eq!(snapshot.counts.total, 1);
    // hook task session-1 is Running (received TaskCreated event); scanned-only rows are hidden once hook tasks exist
    assert_eq!(snapshot.counts.running, 1);
    assert_eq!(snapshot.counts.completed, 0);
    assert_eq!(snapshot.tasks.len(), 1);
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

use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use claude_task_window::{
    model::{HookEvent, HookEventType},
    server::build_router,
    store::TaskStore,
};
use tower::ServiceExt;

#[tokio::test]
async fn ingests_events_and_returns_snapshot() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store);

    let create = HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-1".into(),
        pid: 321,
        title: "Review PR".into(),
        occurred_at: "2026-04-24T17:00:00Z".into(),
    };

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

    let snapshot = app
        .oneshot(Request::get("/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(snapshot.status(), StatusCode::OK);
}

use claude_task_window::model::{HookEvent, HookEventType, TaskStatus};
use claude_task_window::store::TaskStore;

#[test]
fn applies_running_needs_user_and_completed_events() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-1".into(),
        pid: 101,
        title: "Write implementation plan".into(),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-1".into(),
        pid: 101,
        title: "Write implementation plan".into(),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-1".into(),
        pid: 101,
        title: "Write implementation plan".into(),
        occurred_at: "2026-04-24T16:02:00Z".into(),
    });

    let snapshot = store.snapshot();
    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.needs_user, 0);
    assert_eq!(snapshot.counts.completed, 1);
    assert_eq!(snapshot.counts.running, 0);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
    assert_eq!(snapshot.tasks[0].pid, 101);
}

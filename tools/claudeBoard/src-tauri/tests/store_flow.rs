use claude_board::model::{HookEvent, HookEventType, TaskCard, TaskStatus, WindowTarget};
use claude_board::store::TaskStore;

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

#[test]
fn replace_scanned_tasks_prefers_hook_tasks_for_matching_session_and_pid() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-dup".into(),
        pid: 201,
        title: "Approve tool call".into(),
        occurred_at: "2026-04-24T16:05:00Z".into(),
    });

    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-dup:201".into(),
            session_id: "session-dup".into(),
            pid: 201,
            title: "Recovered duplicate session".into(),
            status: TaskStatus::Running,
            source: "scan_recovered".into(),
            window_target: WindowTarget {
                host_kind: "terminal".into(),
                app: "Ghostty".into(),
                descriptor: "ghostty-main".into(),
                tab_id: None,
                pane_id: None,
            },
            started_at: "2026-04-24T16:04:00Z".into(),
            updated_at: "2026-04-24T16:04:00Z".into(),
            completed_at: None,
        }],
        &[201],
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.needs_user, 1);
    assert_eq!(snapshot.counts.running, 0);
    assert_eq!(snapshot.counts.completed, 0);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].source, "hook");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::NeedsUser);
    assert_eq!(snapshot.tasks[0].session_id, "session-dup");
    assert_eq!(snapshot.tasks[0].pid, 201);
}

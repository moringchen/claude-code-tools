use claude_board::model::{HookEvent, HookEventType, TaskCard, TaskStatus, WindowTarget};
use claude_board::store::TaskStore;

#[test]
fn apply_reports_only_real_status_transitions() {
    let mut store = TaskStore::default();

    let first_waiting = store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-sound".into(),
        pid: 501,
        title: "Approve command".into(),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    let repeated_waiting = store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-sound".into(),
        pid: 501,
        title: "Approve command".into(),
        occurred_at: "2026-04-24T16:02:00Z".into(),
    });
    let completed = store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-sound".into(),
        pid: 501,
        title: "Approve command".into(),
        occurred_at: "2026-04-24T16:03:00Z".into(),
    });

    assert_eq!(first_waiting, Some(TaskStatus::NeedsUser));
    assert_eq!(repeated_waiting, None);
    assert_eq!(completed, Some(TaskStatus::Completed));
}

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
fn snapshot_hides_completed_tasks_when_active_tasks_exist() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        pid: 301,
        title: "Finished task".into(),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-active".into(),
        pid: 302,
        title: "Needs approval".into(),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.needs_user, 1);
    assert_eq!(snapshot.counts.completed, 0);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-active");
}

#[test]
fn snapshot_hides_scanned_tasks_when_hook_tasks_exist() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-active".into(),
        pid: 401,
        title: "Needs approval".into(),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-402:402".into(),
            session_id: "local-402".into(),
            pid: 402,
            title: "Recovered stale process".into(),
            status: TaskStatus::NotStarted,
            source: "scan_recovered".into(),
            window_target: WindowTarget {
                host_kind: "terminal".into(),
                app: "Terminal".into(),
                descriptor: "terminal".into(),
                tab_id: None,
                pane_id: None,
            },
            started_at: "2026-04-24T16:00:00Z".into(),
            updated_at: "2026-04-24T16:00:00Z".into(),
            completed_at: None,
        }],
        &[401, 402],
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].source, "hook");
}

#[test]
fn snapshot_shows_scanned_tasks_when_hook_tasks_are_only_completed_history() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        pid: 601,
        title: "Finished history".into(),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-602:602".into(),
            session_id: "local-602".into(),
            pid: 602,
            title: "Current scanned task".into(),
            status: TaskStatus::NotStarted,
            source: "scan_recovered".into(),
            window_target: WindowTarget {
                host_kind: "terminal".into(),
                app: "Terminal".into(),
                descriptor: "terminal".into(),
                tab_id: None,
                pane_id: None,
            },
            started_at: "2026-04-24T16:01:00Z".into(),
            updated_at: "2026-04-24T16:01:00Z".into(),
            completed_at: None,
        }],
        &[602],
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].source, "scan_recovered");
    assert_eq!(snapshot.tasks[0].session_id, "local-602");
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

#[test]
fn task_created_reopens_completed_session_as_running() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-reused".into(),
        pid: 701,
        title: "Previous turn".into(),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    let transition = store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-reused".into(),
        pid: 701,
        title: "Next turn".into(),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.snapshot();

    assert_eq!(transition, Some(TaskStatus::Running));
    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.running, 1);
    assert_eq!(snapshot.counts.completed, 0);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Running);
    assert_eq!(snapshot.tasks[0].title, "Next turn");
    assert_eq!(snapshot.tasks[0].completed_at, None);
}

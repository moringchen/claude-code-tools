use claude_board::model::{
    HookEvent, HookEventType, NotificationSoundType, SnapshotCounts, TaskCard, TaskLiveness,
    TaskSnapshot, TaskStatus, WindowTarget,
};
use claude_board::store::TaskStore;

#[test]
fn permission_request_enqueues_waiting_notification_in_same_snapshot() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-atomic".into(),
        agent_id: None,
        pid: 111,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:10:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-atomic".into(),
        agent_id: None,
        pid: 111,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:11:00Z".into(),
    });

    let snapshot = store.snapshot();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.task_id == "session-atomic")
        .unwrap();

    assert_eq!(task.status, TaskStatus::NeedsUser);
    assert_eq!(snapshot.notifications.len(), 1);
    assert_eq!(snapshot.notifications[0].sound_type, NotificationSoundType::Waiting);
}

#[test]
fn permission_denied_settles_task_to_completed_and_enqueues_completed_notification() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-denied".into(),
        agent_id: None,
        pid: 811,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T08:01:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionDenied,
        session_id: "session-denied".into(),
        agent_id: None,
        pid: 811,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T08:02:00Z".into(),
    });

    let snapshot = store.snapshot();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.task_id == "session-denied")
        .unwrap();

    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(snapshot.notifications.len(), 2);
    assert_eq!(snapshot.notifications[0].sound_type, NotificationSoundType::Waiting);
    assert_eq!(snapshot.notifications[1].sound_type, NotificationSoundType::Completed);
}

#[test]
fn stop_settles_session_to_completed_and_next_task_created_reopens_it_to_running() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-reopen".into(),
        agent_id: None,
        pid: 911,
        title: "Write reply".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T10:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-reopen".into(),
        agent_id: None,
        pid: 911,
        title: "Write reply".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T10:00:10Z".into(),
    });

    let completed_snapshot = store.snapshot();
    let completed_task = completed_snapshot
        .tasks
        .iter()
        .find(|task| task.task_id == "session-reopen")
        .unwrap();
    assert_eq!(completed_task.status, TaskStatus::Completed);

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-reopen".into(),
        agent_id: None,
        pid: 911,
        title: "Write reply".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T10:01:00Z".into(),
    });

    let reopened_snapshot = store.snapshot();
    let reopened_task = reopened_snapshot
        .tasks
        .iter()
        .find(|task| task.task_id == "session-reopen")
        .unwrap();
    assert_eq!(reopened_task.status, TaskStatus::Running);
}

#[test]
fn completed_transition_enqueues_one_completed_notification_only_once() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-sound".into(),
        agent_id: None,
        pid: 501,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-sound".into(),
        agent_id: None,
        pid: 501,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:03:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-sound".into(),
        agent_id: None,
        pid: 501,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:04:00Z".into(),
    });

    let snapshot = store.persisted_snapshot();

    assert_eq!(snapshot.notifications.len(), 2);
    assert_eq!(snapshot.notifications[0].sound_type, NotificationSoundType::Waiting);
    assert_eq!(snapshot.notifications[1].sound_type, NotificationSoundType::Completed);
}

#[test]
fn persisted_snapshot_restores_pending_notifications() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-restore".into(),
        agent_id: None,
        pid: 701,
        title: "Restore pending notification".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T17:00:00Z".into(),
    });

    let persisted = store.persisted_snapshot();

    let mut restored = TaskStore::default();
    restored.restore_snapshot(persisted);

    let snapshot = restored.persisted_snapshot();
    assert_eq!(snapshot.notifications.len(), 1);
    assert_eq!(snapshot.notifications[0].task_id, "session-restore");
    assert_eq!(snapshot.notifications[0].status, TaskStatus::NeedsUser);
}

#[test]
fn restore_snapshot_does_not_resurrect_closed_sessions_from_persisted_state() {
    let previous_snapshot = TaskSnapshot {
        counts: SnapshotCounts::default(),
        tasks: Vec::new(),
        sessions: vec![claude_board::model::SessionRecord {
            session_id: "session-closed".into(),
            last_conversation_content: "Closed session".into(),
            last_hook_event: HookEventType::SessionEnd,
            last_status: TaskStatus::Completed,
            updated_at: "2026-04-24T17:11:00Z".into(),
        }],
        notifications: Vec::new(),
    };

    let mut restored = TaskStore::default();
    restored.restore_snapshot(previous_snapshot);
    restored.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-closed:777".into(),
            session_id: "session-closed".into(),
            pid: 777,
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
            started_at: "2026-04-24T17:12:00Z".into(),
            updated_at: "2026-04-24T17:12:00Z".into(),
            completed_at: None,
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[777],
        "2026-04-24T17:12:00Z",
    );

    let snapshot = restored.snapshot();

    assert!(snapshot.tasks.is_empty());
    assert!(snapshot
        .sessions
        .iter()
        .all(|record| record.session_id != "session-closed"));
}

#[test]
fn ignores_subagent_task_keys_in_visible_snapshot() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-main".into(),
        agent_id: None,
        pid: 123,
        title: "Main task".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T15:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-main".into(),
        agent_id: Some("subagent-1".into()),
        pid: 123,
        title: "Subagent task".into(),
        conversation_content: None,
        occurred_at: "2026-05-06T15:00:10Z".into(),
    });

    let snapshot = store.snapshot();

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].task_id, "session-main");
    assert_eq!(snapshot.tasks[0].title, "Main task");
}

#[test]
fn restore_snapshot_drops_persisted_subagent_hook_tasks() {
    let previous_snapshot = TaskSnapshot {
        counts: SnapshotCounts::default(),
        tasks: vec![TaskCard {
            task_id: "session-main:subagent-1".into(),
            session_id: "session-main".into(),
            pid: 123,
            title: "Subagent task".into(),
            status: TaskStatus::Running,
            source: "hook".into(),
            window_target: WindowTarget {
                host_kind: "unknown".into(),
                app: "unknown".into(),
                descriptor: "unknown".into(),
                tab_id: None,
                pane_id: None,
            },
            started_at: "2026-05-06T15:00:00Z".into(),
            updated_at: "2026-05-06T15:00:00Z".into(),
            completed_at: None,
            liveness: TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        sessions: vec![claude_board::model::SessionRecord {
            session_id: "session-main".into(),
            last_conversation_content: "Main task".into(),
            last_hook_event: HookEventType::TaskCreated,
            last_status: TaskStatus::Running,
            updated_at: "2026-05-06T15:00:00Z".into(),
        }],
        notifications: Vec::new(),
    };

    let mut restored = TaskStore::default();
    restored.restore_snapshot(previous_snapshot);

    let snapshot = restored.snapshot();
    let persisted = restored.persisted_snapshot();

    assert!(snapshot.tasks.is_empty());
    assert!(persisted.tasks.is_empty());
}

#[test]
fn applies_running_needs_user_and_completed_events() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-1".into(),
        agent_id: None,
        pid: 101,
        title: "Write implementation plan".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-1".into(),
        agent_id: None,
        pid: 101,
        title: "Write implementation plan".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-1".into(),
        agent_id: None,
        pid: 101,
        title: "Write implementation plan".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:02:00Z".into(),
    });

    let snapshot = store.snapshot();
    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.needs_user, 0);
    assert_eq!(snapshot.counts.completed, 1);
    assert_eq!(snapshot.counts.running, 0);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-1");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
}

#[test]
fn snapshot_hides_completed_tasks_when_active_tasks_exist() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        agent_id: None,
        pid: 301,
        title: "Finished task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-active".into(),
        agent_id: None,
        pid: 302,
        title: "Needs approval".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 2);
    assert_eq!(snapshot.counts.needs_user, 1);
    assert_eq!(snapshot.counts.completed, 1);
    assert_eq!(snapshot.tasks.len(), 2);
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-done" && task.status == TaskStatus::Completed));
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-active" && task.status == TaskStatus::NeedsUser));
}

#[test]
fn snapshot_hides_matching_scanned_task_when_hook_task_exists() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-active".into(),
        agent_id: None,
        pid: 401,
        title: "Needs approval".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-active:401".into(),
            session_id: "session-active".into(),
            pid: 401,
            title: "Recovered duplicate process".into(),
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[401],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].source, "hook");
}

#[test]
fn completed_hook_session_does_not_reappear_from_matching_scanned_task() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-done".into(),
        agent_id: None,
        pid: 601,
        title: "Finished history".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T15:59:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        agent_id: None,
        pid: 601,
        title: "Finished history".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-done:601".into(),
            session_id: "session-done".into(),
            pid: 601,
            title: "Recovered duplicate history".into(),
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[601],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.completed, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-done");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
}

#[test]
fn snapshot_keeps_other_scanned_sessions_when_hook_task_exists() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-active".into(),
        agent_id: None,
        pid: 401,
        title: "Active hook task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-402:402".into(),
            session_id: "local-402".into(),
            pid: 402,
            title: "Recovered other session".into(),
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[401, 402],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 2);
    assert_eq!(snapshot.counts.running, 1);
    assert_eq!(snapshot.tasks.len(), 2);
    assert_eq!(snapshot.tasks[0].source, "hook");
    assert_eq!(snapshot.tasks[1].source, "scan_recovered");
    assert_eq!(snapshot.tasks[1].session_id, "local-402");
}

#[test]
fn replace_scanned_tasks_prefers_hook_tasks_for_matching_session_and_pid() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-dup".into(),
        agent_id: None,
        pid: 201,
        title: "Approve tool call".into(),
        conversation_content: None,
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[201],
        "2026-04-24T16:05:00Z",
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
fn snapshot_hides_local_scanned_task_with_same_pid_as_hook_task() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-active".into(),
        agent_id: None,
        pid: 401,
        title: "Active hook task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-401:401".into(),
            session_id: "local-401".into(),
            pid: 401,
            title: "Recovered duplicate local process".into(),
            status: TaskStatus::IdleOrUnknown,
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[401],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].source, "hook");
}

#[test]
fn snapshot_keeps_scanned_task_with_same_session_but_different_pid() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-active".into(),
        agent_id: None,
        pid: 401,
        title: "Active hook task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });
    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-active:402".into(),
            session_id: "session-active".into(),
            pid: 402,
            title: "Same session other process".into(),
            status: TaskStatus::Running,
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[401, 402],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.counts.total, 2);
    assert_eq!(snapshot.tasks.len(), 2);
    assert!(snapshot.tasks.iter().any(|task| task.source == "scan_recovered" && task.pid == 402));
}

#[test]
fn restore_snapshot_keeps_last_hook_status_for_scanned_session() {
    let mut previous_store = TaskStore::default();
    previous_store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-restored".into(),
        agent_id: None,
        pid: 801,
        title: "Approve restored command".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    let previous_snapshot = previous_store.snapshot();

    let mut restored_store = TaskStore::default();
    restored_store.restore_snapshot(previous_snapshot);
    restored_store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:session-restored:801".into(),
            session_id: "session-restored".into(),
            pid: 801,
            title: "Recovered duplicate session".into(),
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
            liveness: claude_board::model::TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[801],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = restored_store.snapshot();

    assert_eq!(snapshot.counts.total, 1);
    assert_eq!(snapshot.counts.needs_user, 1);
    assert_eq!(snapshot.tasks[0].source, "hook");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::NeedsUser);
    assert_eq!(snapshot.tasks[0].title, "Approve restored command");
}

#[test]
fn task_created_reopens_completed_session_as_running() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-reused".into(),
        agent_id: None,
        pid: 701,
        title: "Previous turn".into(),
        conversation_content: Some("Previous turn".into()),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    let transition = store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-reused".into(),
        agent_id: None,
        pid: 701,
        title: "Next turn".into(),
        conversation_content: Some("Next turn".into()),
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

#[test]
fn task_created_resumes_needs_user_session_as_running() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-replied".into(),
        agent_id: None,
        pid: 901,
        title: "Approve command".into(),
        conversation_content: Some("Approve command".into()),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    let transition = store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-replied".into(),
        agent_id: None,
        pid: 901,
        title: "Run approved command".into(),
        conversation_content: Some("Run approved command".into()),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.snapshot();

    assert_eq!(transition, Some(TaskStatus::Running));
    assert_eq!(snapshot.counts.needs_user, 0);
    assert_eq!(snapshot.counts.running, 1);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Running);
    assert_eq!(snapshot.tasks[0].title, "Run approved command");
}

#[test]
fn snapshot_records_last_session_content_hook_event_and_status() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-recorded".into(),
        agent_id: None,
        pid: 902,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-recorded".into(),
        agent_id: None,
        pid: 902,
        title: "Continue after approval".into(),
        conversation_content: Some("Continue after approval".into()),
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.snapshot();
    let record = snapshot
        .sessions
        .iter()
        .find(|record| record.session_id == "session-recorded")
        .unwrap();

    assert_eq!(record.last_conversation_content, "Continue after approval");
    assert_eq!(record.last_hook_event, HookEventType::TaskCreated);
    assert_eq!(record.last_status, TaskStatus::Running);
    assert_eq!(record.updated_at, "2026-04-24T16:01:00Z");
}

#[test]
fn snapshot_prefers_last_conversation_content_for_visible_title() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-title-priority".into(),
        agent_id: None,
        pid: 779,
        title: "workspace".into(),
        conversation_content: Some("Latest conversation topic".into()),
        occurred_at: "2026-05-07T12:00:00Z".into(),
    });

    let snapshot = store.snapshot();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.session_id == "session-title-priority")
        .unwrap();

    assert_eq!(task.title, "Latest conversation topic");
}

#[test]
fn scanned_task_metadata_overrides_unknown_hook_title_and_window_target() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-merge".into(),
        agent_id: None,
        pid: 777,
        title: "workspace".into(),
        conversation_content: None,
        occurred_at: "2026-05-07T10:00:00Z".into(),
    });

    store.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-777:777".into(),
            session_id: "local-777".into(),
            pid: 777,
            title: "Real task title".into(),
            status: TaskStatus::IdleOrUnknown,
            source: "scan_recovered".into(),
            window_target: WindowTarget {
                host_kind: "tmux".into(),
                app: "Ghostty".into(),
                descriptor: "dev".into(),
                tab_id: Some("team".into()),
                pane_id: Some("1.2".into()),
            },
            started_at: "2026-05-07T10:00:00Z".into(),
            updated_at: "2026-05-07T10:00:00Z".into(),
            completed_at: None,
            liveness: TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[777],
        "2026-05-07T10:00:01Z",
    );

    let snapshot = store.snapshot();
    let task = snapshot
        .tasks
        .iter()
        .find(|task| task.session_id == "session-merge")
        .unwrap();

    assert_eq!(task.title, "Real task title");
    assert_eq!(task.window_target.host_kind, "tmux");
    assert_eq!(task.window_target.app, "Ghostty");
    assert_eq!(task.window_target.descriptor, "dev");
    assert_eq!(task.window_target.tab_id.as_deref(), Some("team"));
    assert_eq!(task.window_target.pane_id.as_deref(), Some("1.2"));
}

#[test]
fn scan_refresh_marks_missing_hook_task_dead_before_retention_prune() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-dead-mark".into(),
        agent_id: None,
        pid: 778,
        title: "Needs approval".into(),
        conversation_content: None,
        occurred_at: "2026-05-07T11:00:00Z".into(),
    });
    store.replace_scanned_tasks(Vec::new(), &[], "2026-05-07T11:01:00Z");

    let persisted = store.persisted_snapshot();
    let task = persisted
        .tasks
        .iter()
        .find(|task| task.session_id == "session-dead-mark")
        .unwrap();

    assert_eq!(task.liveness, TaskLiveness::Dead);
    assert_eq!(task.removed_at.as_deref(), Some("2026-05-07T11:01:00Z"));
    assert_eq!(task.removed_reason, Some(claude_board::model::RemovalReason::ProcessExited));
}

#[test]
fn dead_tasks_are_hidden_but_persisted_until_retention_expires() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-dead-recent".into(),
        agent_id: None,
        pid: 906,
        title: "Recently closed terminal".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.replace_scanned_tasks(Vec::new(), &[], "2026-04-24T16:01:00Z");

    let snapshot = store.snapshot();
    let persisted = store.persisted_snapshot();

    assert!(snapshot.tasks.is_empty());
    assert!(persisted
        .tasks
        .iter()
        .any(|task| task.session_id == "session-dead-recent" && task.liveness == TaskLiveness::Dead));
}

#[test]
fn dead_task_session_records_remain_until_retention_prune() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-dead-expired".into(),
        agent_id: None,
        pid: 907,
        title: "Expired closed terminal".into(),
        conversation_content: Some("Expired closed terminal".into()),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.replace_scanned_tasks(Vec::new(), &[], "2026-04-24T16:01:00Z");

    let persisted = store.persisted_snapshot();

    assert!(persisted
        .tasks
        .iter()
        .any(|task| task.session_id == "session-dead-expired" && task.liveness == TaskLiveness::Dead));
    assert!(persisted
        .sessions
        .iter()
        .any(|record| record.session_id == "session-dead-expired"));
}

#[test]
fn restore_snapshot_keeps_completed_hook_tasks_without_resurrecting_scanned_idle_rows() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        agent_id: None,
        pid: 908,
        title: "Finished task".into(),
        conversation_content: Some("Finished task".into()),
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });

    let persisted = store.persisted_snapshot();
    let mut restored = TaskStore::default();
    restored.restore_snapshot(persisted);

    restored.replace_scanned_tasks(
        vec![TaskCard {
            task_id: "scan:local-908:908".into(),
            session_id: "local-908".into(),
            pid: 908,
            title: "Recovered idle task".into(),
            status: TaskStatus::IdleOrUnknown,
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
            liveness: TaskLiveness::Alive,
            removed_at: None,
            removed_reason: None,
        }],
        &[908],
        "2026-04-24T16:01:00Z",
    );

    let snapshot = restored.snapshot();

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].session_id, "session-done");
    assert_eq!(snapshot.tasks[0].status, TaskStatus::Completed);
    assert_eq!(snapshot.counts.completed, 1);
}

#[test]
fn persisted_snapshot_keeps_completed_hook_tasks_when_active_tasks_exist() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-done".into(),
        agent_id: None,
        pid: 904,
        title: "Finished task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:00:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-waiting".into(),
        agent_id: None,
        pid: 905,
        title: "Waiting task".into(),
        conversation_content: None,
        occurred_at: "2026-04-24T16:01:00Z".into(),
    });

    let snapshot = store.persisted_snapshot();

    assert_eq!(snapshot.tasks.len(), 2);
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-done" && task.status == TaskStatus::Completed));
    assert!(snapshot
        .tasks
        .iter()
        .any(|task| task.session_id == "session-waiting" && task.status == TaskStatus::NeedsUser));
}

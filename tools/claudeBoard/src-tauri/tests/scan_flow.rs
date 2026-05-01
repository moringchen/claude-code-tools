use claude_task_window::scan::rebuild_tasks_from_rows;

#[test]
fn skips_short_rows_when_rebuilding_scan_tasks() {
    let rows = vec!["session-1\t901\tclaude\tGhostty\tghostty-main".to_string()];

    let tasks = rebuild_tasks_from_rows(&rows, "2026-04-24T17:30:00Z");

    assert!(tasks.is_empty());
}

#[test]
fn skips_rows_with_invalid_pid_when_rebuilding_scan_tasks() {
    let rows = vec![
        "session-1\tnot-a-pid\tclaude\tGhostty\tghostty-main\t\t\tRefactor overlay".to_string(),
    ];

    let tasks = rebuild_tasks_from_rows(&rows, "2026-04-24T17:30:00Z");

    assert!(tasks.is_empty());
}

#[test]
fn rebuilds_tmux_and_ghostty_context_from_scan_rows() {
    let rows = vec![
        "session-1\t901\tclaude\tGhostty\tghostty-main\t\t\tRefactor overlay".to_string(),
        "session-2\t902\tclaude\tGhostty\ttmux\tdev\t1.2\tFix permission prompt".to_string(),
    ];

    let tasks = rebuild_tasks_from_rows(&rows, "2026-04-24T17:30:00Z");

    assert_eq!(tasks.len(), 2);

    assert_eq!(tasks[0].task_id, "scan:session-1:901");
    assert_eq!(tasks[0].pid, 901);
    assert_eq!(
        tasks[0].status,
        claude_task_window::model::TaskStatus::Running
    );
    assert_eq!(tasks[0].source, "scan_recovered");
    assert_eq!(tasks[0].window_target.host_kind, "terminal");
    assert_eq!(tasks[0].window_target.app, "Ghostty");
    assert_eq!(tasks[0].window_target.descriptor, "ghostty-main");
    assert_eq!(tasks[0].window_target.tab_id, None);
    assert_eq!(tasks[0].window_target.pane_id, None);

    assert_eq!(tasks[1].task_id, "scan:session-2:902");
    assert_eq!(tasks[1].pid, 902);
    assert_eq!(tasks[1].window_target.host_kind, "tmux");
    assert_eq!(tasks[1].window_target.tab_id.as_deref(), Some("dev"));
    assert_eq!(tasks[1].window_target.pane_id.as_deref(), Some("1.2"));
    assert_eq!(tasks[1].source, "scan_recovered");
}

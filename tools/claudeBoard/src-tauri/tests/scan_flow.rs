use claude_board::scan::{compute_scan, parse_scan_row, rebuild_tasks_from_rows};

#[test]
fn parse_scan_row_accepts_plain_claude_command() {
    let row = parse_scan_row("123 0 S /usr/local/bin/claude");

    assert_eq!(
        row.as_deref(),
        Some("local-123\t123\tworkspace\tTerminal\tterminal\t\t\tclaude")
    );
}

#[test]
fn parse_scan_row_accepts_claude_code_command() {
    let row = parse_scan_row("456 0 S /opt/homebrew/bin/claude code");

    assert_eq!(
        row.as_deref(),
        Some("local-456\t456\tworkspace\tTerminal\tterminal\t\t\tclaude code")
    );
}

#[test]
fn parse_scan_row_accepts_windows_claude_executable() {
    let row = parse_scan_row(r#"457 0 S claude.exe"#);

    assert_eq!(
        row.as_deref(),
        Some("local-457\t457\tworkspace\tTerminal\tterminal\t\t\tclaude.exe")
    );
}

#[test]
fn parse_scan_row_accepts_windows_claude_path_with_arguments() {
    let row = parse_scan_row(r#"458 0 S C:\\Users\\me\\AppData\\Local\\Programs\\Claude\\claude.exe code"#);

    assert_eq!(
        row.as_deref(),
        Some("local-458\t458\tworkspace\tTerminal\tterminal\t\t\tclaude.exe code")
    );
}

#[test]
fn parse_scan_row_rejects_claude_board_daemon() {
    assert_eq!(parse_scan_row("789 0 S claude_boardd"), None);
}

#[test]
fn parse_scan_row_rejects_claude_board_path() {
    assert_eq!(
        parse_scan_row("790 0 S /Users/me/tools/claudeBoard/src-tauri/target/debug/claude"),
        None
    );
}

#[test]
fn parse_scan_row_rejects_unrelated_claude_substring() {
    assert_eq!(parse_scan_row("791 0 S /usr/bin/notclaude helper"), None);
}

#[test]
fn parse_scan_row_rejects_windows_unrelated_claude_substring() {
    assert_eq!(parse_scan_row(r#"792 0 S C:\\tools\\notclaude.exe helper"#), None);
}

#[test]
fn parse_scan_row_rejects_windows_claude_board_path() {
    assert_eq!(
        parse_scan_row(r#"793 0 S C:\\workspace\\claudeBoard\\target\\debug\\claude.exe"#),
        None
    );
}

#[test]
fn parse_scan_row_accepts_claude_with_claude_board_argument() {
    let row = parse_scan_row(r#"794 0 S claude --cwd C:\\workspace\\claudeBoard"#);

    assert_eq!(
        row.as_deref(),
        Some("local-794\t794\tworkspace\tTerminal\tterminal\t\t\tclaude --cwd C:\\\\workspace\\\\claudeBoard")
    );
}

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
fn rebuilds_tasks_for_all_local_claude_code_sessions() {
    let rows = vec![
        "session-1\t901\tclaude\tGhostty\tghostty-main\t\t\tRefactor overlay".to_string(),
        "session-2\t902\tclaude\tGhostty\ttmux\tdev\t1.2\tFix permission prompt".to_string(),
    ];

    let tasks = rebuild_tasks_from_rows(&rows, "2026-04-24T17:30:00Z");

    assert_eq!(tasks.len(), 2);

    assert_eq!(tasks[0].task_id, "scan:session-1:901");
    assert_eq!(tasks[0].session_id, "session-1");
    assert_eq!(tasks[0].pid, 901);
    assert_eq!(tasks[0].status, claude_board::model::TaskStatus::IdleOrUnknown);
    assert_eq!(tasks[0].source, "scan_recovered");

    assert_eq!(tasks[1].task_id, "scan:session-2:902");
    assert_eq!(tasks[1].session_id, "session-2");
    assert_eq!(tasks[1].pid, 902);
    assert_eq!(tasks[1].status, claude_board::model::TaskStatus::IdleOrUnknown);
    assert_eq!(tasks[1].source, "scan_recovered");
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
    assert_eq!(tasks[0].status, claude_board::model::TaskStatus::IdleOrUnknown);
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

#[test]
fn compute_scan_records_accepted_and_rejected_entries() {
    let computation = compute_scan(
        "901 0 S /usr/local/bin/claude\n902 901 S /usr/local/bin/claude code\n903 0 T /usr/local/bin/claude",
        "2026-05-06T15:10:00Z",
    );

    assert_eq!(computation.rows.len(), 1);
    assert_eq!(computation.alive_pids, vec![901]);
    assert_eq!(computation.debug.occurred_at, "2026-05-06T15:10:00Z");

    let accepted = computation
        .debug
        .entries
        .iter()
        .find(|entry| entry.pid == Some(901))
        .unwrap();
    assert_eq!(accepted.decision, claude_board::model::ScanDecision::Accepted);
    assert_eq!(accepted.task.as_ref().map(|task| task.session_id.as_str()), Some("local-901"));

    let child = computation
        .debug
        .entries
        .iter()
        .find(|entry| entry.pid == Some(902))
        .unwrap();
    assert_eq!(child.decision, claude_board::model::ScanDecision::Rejected);
    assert_eq!(child.reason.as_deref(), Some("child_claude_process"));

    let stopped = computation
        .debug
        .entries
        .iter()
        .find(|entry| entry.pid == Some(903))
        .unwrap();
    assert_eq!(stopped.decision, claude_board::model::ScanDecision::Rejected);
    assert_eq!(stopped.reason.as_deref(), Some("stopped_process"));
}

#[test]
fn compute_scan_keeps_debug_trace_for_main_scan_flow() {
    let computation = compute_scan(
        "200 1 S /usr/local/bin/claude\n201 200 S /usr/local/bin/claude\n202 1 T /usr/local/bin/claude\n203 1 S /usr/bin/python\n",
        "2026-04-24T17:30:00Z",
    );

    assert_eq!(computation.rows.len(), 1);
    assert_eq!(computation.alive_pids, vec![200]);
    assert_eq!(computation.debug.entries.len(), 3);
    assert_eq!(computation.debug.entries[0].accepted_row.as_deref(), Some("local-200\t200\tworkspace\tTerminal\tterminal\t\t\tclaude"));
    assert_eq!(computation.debug.entries[1].reason.as_deref(), Some("child_claude_process"));
    assert_eq!(computation.debug.entries[2].reason.as_deref(), Some("stopped_process"));
}

#[test]
fn rebuilds_windows_scan_rows_into_not_started_tasks() {
    let rows = vec!["session-win\t903\tworkspace\tTerminal\tterminal\t\t\tclaude.exe code".to_string()];

    let tasks = rebuild_tasks_from_rows(&rows, "2026-04-24T17:30:00Z");

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, "scan:session-win:903");
    assert_eq!(tasks[0].title, "claude.exe code");
    assert_eq!(tasks[0].status, claude_board::model::TaskStatus::IdleOrUnknown);
    assert_eq!(tasks[0].source, "scan_recovered");
}

use crate::model::{TaskCard, TaskStatus, WindowTarget};

pub fn parse_scan_row(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let pid = parts.next()?;
    let command_parts = parts.collect::<Vec<_>>();
    if command_parts.is_empty() {
        return None;
    }

    if !is_claude_code_command(&command_parts) {
        return None;
    }

    let title = display_command_title(&command_parts);
    let row = format!(
        "local-{pid}\t{pid}\tworkspace\tTerminal\tterminal\t\t\t{title}"
    );
    eprintln!("[claudeBoard] scan accepted pid={pid} title={title:?}");
    Some(row)
}

fn executable_basename(token: &str) -> &str {
    token
        .rsplit(|character| character == '/' || character == '\\')
        .next()
        .unwrap_or(token)
}

fn normalized_executable_name(token: &str) -> String {
    let lowercase = executable_basename(token).to_ascii_lowercase();
    lowercase
        .strip_suffix(".exe")
        .unwrap_or(&lowercase)
        .to_string()
}

fn display_command_title(command_parts: &[&str]) -> String {
    let executable = executable_basename(command_parts[0]);
    if command_parts.len() == 1 {
        executable.to_string()
    } else {
        format!("{} {}", executable, command_parts[1..].join(" "))
    }
}

fn is_claude_code_command(command_parts: &[&str]) -> bool {
    let executable_token = command_parts[0];
    let normalized = normalized_executable_name(executable_token);

    if normalized == "claude_boardd"
        || (normalized == "claude" && executable_token.contains("claudeBoard"))
    {
        return false;
    }

    normalized == "claude"
}

pub fn rebuild_tasks_from_rows(rows: &[String], now: &str) -> Vec<TaskCard> {
    let mut tasks = Vec::new();
    let mut skipped_short_rows = 0;
    let mut skipped_invalid_pid_rows = 0;

    for row in rows {
        let parts = row.split('\t').collect::<Vec<_>>();
        if parts.len() < 8 {
            skipped_short_rows += 1;
            continue;
        }

        let pid = match parts[1].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => {
                skipped_invalid_pid_rows += 1;
                continue;
            }
        };
        let host_kind = if parts[4] == "tmux" {
            "tmux"
        } else {
            "terminal"
        };

        tasks.push(TaskCard {
            task_id: format!("scan:{}:{}", parts[0], parts[1]),
            session_id: parts[0].to_string(),
            pid,
            title: parts[7].to_string(),
            status: TaskStatus::Running,
            source: "scan_recovered".into(),
            window_target: WindowTarget {
                host_kind: host_kind.into(),
                app: parts[3].to_string(),
                descriptor: parts[4].to_string(),
                tab_id: if parts[5].is_empty() {
                    None
                } else {
                    Some(parts[5].to_string())
                },
                pane_id: if parts[6].is_empty() {
                    None
                } else {
                    Some(parts[6].to_string())
                },
            },
            started_at: now.to_string(),
            updated_at: now.to_string(),
            completed_at: None,
        });
    }

    eprintln!(
        "[claudeBoard] scan rebuild completed rebuilt_count={} skipped_short_rows={} skipped_invalid_pid_rows={}",
        tasks.len(), skipped_short_rows, skipped_invalid_pid_rows
    );
    tasks
}

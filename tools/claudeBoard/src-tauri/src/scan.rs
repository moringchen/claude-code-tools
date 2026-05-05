use crate::model::{TaskCard, TaskStatus, WindowTarget};
use crate::session_meta::get_task_title_by_pid;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub state: String,
    pub command: String,
}

pub fn parse_ps_output(output: &str) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let pid_str = match parts.next() {
            Some(p) => p,
            None => continue,
        };
        let ppid_str = match parts.next() {
            Some(p) => p,
            None => continue,
        };
        let state = match parts.next() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let command = parts.collect::<Vec<_>>().join(" ");

        if let (Ok(pid), Ok(ppid)) = (pid_str.parse::<u32>(), ppid_str.parse::<u32>()) {
            processes.push(ProcessInfo {
                pid,
                ppid,
                state,
                command,
            });
        }
    }

    processes
}

pub fn filter_parent_claude_processes(processes: &[ProcessInfo]) -> Vec<&ProcessInfo> {
    let claude_pids: HashSet<u32> = processes
        .iter()
        .filter(|p| is_claude_code_command(&p.command))
        .map(|p| p.pid)
        .collect();

    processes
        .iter()
        .filter(|p| {
            if !is_claude_code_command(&p.command) {
                return false;
            }
            // 排除停止的进程 (state = T)
            if p.state == "T" {
                return false;
            }
            // 排除子进程（PPID 是另一个 claude 进程）
            !claude_pids.contains(&p.ppid)
        })
        .collect()
}

pub fn parse_scan_row(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let pid = parts.next()?;
    let _ppid = parts.next()?; // skip ppid
    let _state = parts.next()?; // skip state
    let command_parts = parts.collect::<Vec<_>>();
    if command_parts.is_empty() {
        return None;
    }

    if !is_claude_code_command(&command_parts.join(" ")) {
        return None;
    }

    let title = extract_task_title(&command_parts);
    let row = format!(
        "local-{pid}\t{pid}\tworkspace\tTerminal\tterminal\t\t\t{title}"
    );
    eprintln!("[claudeBoard] scan accepted pid={pid} title={title:?}");
    Some(row)
}

fn extract_task_title(command_parts: &[&str]) -> String {
    let executable = executable_basename(command_parts[0]);
    if command_parts.len() == 1 {
        executable.to_string()
    } else {
        format!("{} {}", executable, command_parts[1..].join(" "))
    }
}

fn is_claude_code_command(command: &str) -> bool {
    let parts: Vec<_> = command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    let normalized = normalized_executable_name(parts[0]);

    // 排除 claudeBoard 相关进程
    if normalized == "claude_boardd"
        || normalized == "claude_board"
        || (normalized == "claude" && parts[0].contains("claudeBoard"))
    {
        return false;
    }

    // 排除 observer/sidecar 进程（有 --disallowedTools 参数禁用了主要工具）
    let joined = parts.join(" ");
    if joined.contains("--disallowedTools") && joined.contains("Bash,Read,Write,Edit") {
        return false;
    }

    normalized == "claude"
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

pub fn scan_and_filter_rows(ps_output: &str) -> Vec<String> {
    let processes = parse_ps_output(ps_output);
    let parent_processes = filter_parent_claude_processes(&processes);

    parent_processes
        .iter()
        .filter_map(|p| {
            // 格式: pid ppid state command (state 在 parse_scan_row 中会被跳过)
            let line = format!("{} {} {} {}", p.pid, p.ppid, p.state, p.command);
            parse_scan_row(&line)
        })
        .collect()
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

        // Try to get a better title from session metadata/history
        let scanned_title = &parts[7];
        let improved_title = get_task_title_by_pid(pid, None, scanned_title);
        eprintln!("[claudeBoard] scan title pid={} original={:?} improved={:?}", pid, scanned_title, improved_title);

        tasks.push(TaskCard {
            task_id: format!("scan:{}:{}", parts[0], parts[1]),
            session_id: parts[0].to_string(),
            pid,
            title: improved_title,
            status: TaskStatus::NotStarted,
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

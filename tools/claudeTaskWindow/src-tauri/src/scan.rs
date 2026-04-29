use crate::model::{TaskCard, TaskStatus, WindowTarget};

pub fn rebuild_tasks_from_rows(rows: &[String], now: &str) -> Vec<TaskCard> {
    rows.iter()
        .filter_map(|row| {
            let parts = row.split('\t').collect::<Vec<_>>();
            if parts.len() < 8 {
                return None;
            }

            let pid = parts[1].parse::<u32>().ok()?;
            let host_kind = if parts[4] == "tmux" {
                "tmux"
            } else {
                "terminal"
            };

            Some(TaskCard {
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
            })
        })
        .collect::<Vec<_>>()
}

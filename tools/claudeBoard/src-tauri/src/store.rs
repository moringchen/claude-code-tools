use std::collections::HashMap;

use crate::model::{
    HookEvent, HookEventType, SnapshotCounts, TaskCard, TaskSnapshot, TaskStatus, WindowTarget,
};

#[derive(Default)]
pub struct TaskStore {
    hook_tasks: HashMap<String, TaskCard>,
    scanned_tasks: HashMap<String, TaskCard>,
}

impl TaskStore {
    pub fn apply(&mut self, event: HookEvent) -> Option<TaskStatus> {
        // Use session_id only as key for aggregation (one task per session)
        let key = event.session_id.clone();
        let task = self
            .hook_tasks
            .entry(key.clone())
            .or_insert_with(|| TaskCard {
                task_id: key.clone(),
                session_id: event.session_id.clone(),
                pid: event.pid,
                title: event.title.clone(),
                status: TaskStatus::NotStarted,
                source: "hook".into(),
                window_target: WindowTarget {
                    host_kind: "unknown".into(),
                    app: "unknown".into(),
                    descriptor: "unknown".into(),
                    tab_id: None,
                    pane_id: None,
                },
                started_at: event.occurred_at.clone(),
                updated_at: event.occurred_at.clone(),
                completed_at: None,
            });

        task.updated_at = event.occurred_at.clone();
        // Update title with latest prompt
        task.title = event.title.clone();

        let previous_status = task.status.clone();
        match event.event_type {
            HookEventType::TaskCreated => {
                if task.status == TaskStatus::NotStarted {
                    task.status = TaskStatus::Running;
                }
            }
            HookEventType::PermissionRequest => task.status = TaskStatus::NeedsUser,
            HookEventType::PermissionDenied => task.status = TaskStatus::NeedsUser,
            HookEventType::TaskCompleted | HookEventType::SessionEnd => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(event.occurred_at);
            }
        }
        if task.status != previous_status {
            Some(task.status.clone())
        } else {
            None
        }
    }

    pub fn replace_scanned_tasks(&mut self, tasks: Vec<TaskCard>, alive_pids: &[u32]) {
        let previous_count = self.scanned_tasks.len();
        let next_count = tasks.len();
        let alive_pids_set: std::collections::HashSet<u32> = alive_pids.iter().copied().collect();

        eprintln!(
            "[claudeBoard] store replace_scanned_tasks previous_count={} next_count={} alive_pids={:?}",
            previous_count, next_count, alive_pids
        );

        // Update scanned tasks - only keep tasks whose pid is alive
        self.scanned_tasks = tasks
            .into_iter()
            .filter(|task| alive_pids_set.contains(&task.pid))
            .map(|task| (task.task_id.clone(), task))
            .collect();

        // Also clean up hook_tasks - remove tasks whose process has exited
        let hook_pids_to_remove: Vec<String> = self
            .hook_tasks
            .values()
            .filter(|task| {
                // Keep completed tasks (they finished normally)
                // But remove running/needs_user tasks whose process died
                task.status != TaskStatus::Completed && !alive_pids_set.contains(&task.pid)
            })
            .map(|task| task.task_id.clone())
            .collect();

        for pid in hook_pids_to_remove {
            eprintln!("[claudeBoard] removing dead hook task pid={}", pid);
            self.hook_tasks.remove(&pid);
        }
    }

    pub fn snapshot(&self) -> TaskSnapshot {
        let has_active_hook_tasks = self
            .hook_tasks
            .values()
            .any(|task| task.status != TaskStatus::Completed);
        let mut tasks = self.hook_tasks.values().cloned().collect::<Vec<_>>();

        if !has_active_hook_tasks {
            tasks.extend(self.scanned_tasks.values().cloned());
        }
        let has_active_tasks = tasks
            .iter()
            .any(|task| task.status != TaskStatus::Completed);
        if has_active_tasks {
            tasks.retain(|task| task.status != TaskStatus::Completed);
        }

        tasks.sort_by(|left, right| {
            let rank = |status: &TaskStatus| match status {
                TaskStatus::NeedsUser => 0,
                TaskStatus::Running => 1,
                TaskStatus::NotStarted => 2,
                TaskStatus::Completed => 3,
                TaskStatus::IdleOrUnknown => 4,
            };

            rank(&left.status)
                .cmp(&rank(&right.status))
                .then_with(|| right.updated_at.cmp(&left.updated_at))
        });

        let counts = tasks
            .iter()
            .fold(SnapshotCounts::default(), |mut counts, task| {
                counts.total += 1;
                match task.status {
                    TaskStatus::NeedsUser => counts.needs_user += 1,
                    TaskStatus::Completed => counts.completed += 1,
                    TaskStatus::Running => counts.running += 1,
                    TaskStatus::NotStarted | TaskStatus::IdleOrUnknown => {}
                }
                counts
            });

        TaskSnapshot { counts, tasks }
    }
}

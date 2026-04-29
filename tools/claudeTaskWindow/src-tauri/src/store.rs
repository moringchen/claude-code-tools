use std::collections::HashMap;

use crate::model::{
    HookEvent, HookEventType, SnapshotCounts, TaskCard, TaskSnapshot, TaskStatus, WindowTarget,
};

#[derive(Default)]
pub struct TaskStore {
    tasks: HashMap<String, TaskCard>,
}

impl TaskStore {
    pub fn apply(&mut self, event: HookEvent) {
        let key = format!("{}:{}:{}", event.session_id, event.pid, event.title);
        let task = self.tasks.entry(key.clone()).or_insert_with(|| TaskCard {
            task_id: key.clone(),
            session_id: event.session_id.clone(),
            pid: event.pid,
            title: event.title.clone(),
            status: TaskStatus::Running,
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

        match event.event_type {
            HookEventType::TaskCreated => task.status = TaskStatus::Running,
            HookEventType::PermissionRequest => task.status = TaskStatus::NeedsUser,
            HookEventType::PermissionDenied => task.status = TaskStatus::NeedsUser,
            HookEventType::TaskCompleted | HookEventType::SessionEnd => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(event.occurred_at);
            }
        }
    }

    pub fn snapshot(&self) -> TaskSnapshot {
        let mut tasks = self.tasks.values().cloned().collect::<Vec<_>>();
        tasks.sort_by(|left, right| {
            let rank = |status: &TaskStatus| match status {
                TaskStatus::NeedsUser => 0,
                TaskStatus::Completed => 1,
                TaskStatus::Running => 2,
                TaskStatus::IdleOrUnknown => 3,
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
                    TaskStatus::IdleOrUnknown => {}
                }
                counts
            });

        TaskSnapshot { counts, tasks }
    }
}

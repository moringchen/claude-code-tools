use std::collections::HashMap;

use crate::event_log::append_event_log;
use crate::model::{
    DebugSnapshot, HookDebugEntry, HookEvent, HookEventType, NotificationEvent,
    NotificationSoundType, ScanDebugSnapshot, SessionRecord, SnapshotCounts, TaskCard,
    TaskLiveness, TaskSnapshot, TaskStatus, WindowTarget,
};

const MAX_HOOK_DEBUG_EVENTS: usize = 100;

fn is_subagent_task_id(task_id: &str, session_id: &str) -> bool {
    task_id
        .strip_prefix(session_id)
        .and_then(|suffix| suffix.strip_prefix(':'))
        .is_some_and(|suffix| !suffix.is_empty())
}

#[derive(Default)]
pub struct TaskStore {
    hook_tasks: HashMap<String, TaskCard>,
    scanned_tasks: HashMap<String, TaskCard>,
    session_records: HashMap<String, SessionRecord>,
    closed_sessions: HashMap<String, String>,
    pending_notifications: Vec<NotificationEvent>,
    next_notification_id: u64,
    recent_hook_events: Vec<HookDebugEntry>,
    latest_scan: ScanDebugSnapshot,
}

impl TaskStore {
    pub fn apply_debug(&mut self, event: HookEvent, debug_entry: HookDebugEntry) -> Option<TaskStatus> {
        self.push_hook_debug_entry(debug_entry);
        self.apply(event)
    }

    pub fn record_filtered_hook_event(&mut self, debug_entry: HookDebugEntry) {
        self.push_hook_debug_entry(debug_entry);
    }

    pub fn replace_scanned_tasks_with_debug(
        &mut self,
        tasks: Vec<TaskCard>,
        alive_pids: &[u32],
        occurred_at: &str,
        debug: ScanDebugSnapshot,
    ) {
        self.latest_scan = debug;
        self.replace_scanned_tasks(tasks, alive_pids, occurred_at);
    }

    pub fn debug_snapshot(&self) -> DebugSnapshot {
        DebugSnapshot {
            snapshot: self.snapshot(),
            recent_hook_events: self.recent_hook_events.clone(),
            latest_scan: self.latest_scan.clone(),
        }
    }

    pub fn apply(&mut self, event: HookEvent) -> Option<TaskStatus> {
        if event.agent_id.is_some() {
            return None;
        }

        // Use session_id only as key for aggregation (one task per session)
        let key = event.session_id.clone();

        if matches!(event.event_type, HookEventType::SessionEnd)
            || (matches!(event.event_type, HookEventType::TaskCompleted)
                && event.agent_id.is_some()
                && !self.closed_sessions.contains_key(&event.session_id))
        {
            self.hook_tasks.remove(&key);
            self.scanned_tasks
                .retain(|_, task| task.session_id != event.session_id);
            self.session_records.remove(&event.session_id);
            self.closed_sessions
                .insert(event.session_id.clone(), event.occurred_at.clone());
            return None;
        }

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
                liveness: TaskLiveness::Alive,
                removed_at: None,
                removed_reason: None,
            });

        task.updated_at = event.occurred_at.clone();
        task.liveness = TaskLiveness::Alive;
        task.removed_at = None;
        task.removed_reason = None;
        task.title = event.title.clone();

        let previous_status = task.status.clone();
        match event.event_type {
            HookEventType::TaskCreated => {
                if matches!(
                    task.status,
                    TaskStatus::NotStarted | TaskStatus::NeedsUser | TaskStatus::Completed
                ) {
                    task.status = TaskStatus::Running;
                    task.completed_at = None;
                }
            }
            HookEventType::PermissionRequest => task.status = TaskStatus::NeedsUser,
            HookEventType::PermissionDenied => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(event.occurred_at.clone());
                self.scanned_tasks
                    .retain(|_, scanned| scanned.session_id != event.session_id);
            }
            HookEventType::TaskCompleted => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(event.occurred_at.clone());
                self.scanned_tasks
                    .retain(|_, scanned| scanned.session_id != event.session_id);
            }
            HookEventType::SessionEnd => {
                self.hook_tasks.remove(&key);
                self.scanned_tasks
                    .retain(|_, task| task.session_id != event.session_id);
                self.session_records.remove(&event.session_id);
                self.closed_sessions
                    .insert(event.session_id.clone(), event.occurred_at.clone());
                return None;
            }
        }
        let task_snapshot = task.clone();
        let status_changed = task.status != previous_status;
        if status_changed {
            append_event_log(&format!(
                "status_transition session={} pid={} {:?}->{:?} title={}",
                event.session_id,
                event.pid,
                previous_status,
                task.status,
                task.title
            ));
        }
        let changed_status = status_changed.then(|| task.status.clone());
        let last_conversation_content = event
            .conversation_content
            .clone()
            .or_else(|| {
                self.session_records
                    .get(&event.session_id)
                    .map(|record| record.last_conversation_content.clone())
            })
            .unwrap_or_else(|| event.title.clone());
        self.session_records.insert(
            event.session_id.clone(),
            SessionRecord {
                session_id: event.session_id.clone(),
                last_conversation_content,
                last_hook_event: event.event_type.clone(),
                last_status: task_snapshot.status.clone(),
                updated_at: event.occurred_at.clone(),
            },
        );
        self.closed_sessions.remove(&event.session_id);
        if status_changed {
            self.enqueue_notification(&task_snapshot, &event.occurred_at);
        }
        changed_status
    }

    pub fn replace_scanned_tasks(&mut self, tasks: Vec<TaskCard>, alive_pids: &[u32], occurred_at: &str) {
        let previous_count = self.scanned_tasks.len();
        let next_count = tasks.len();
        let alive_pids_set: std::collections::HashSet<u32> = alive_pids.iter().copied().collect();

        eprintln!(
            "[claudeBoard] store replace_scanned_tasks previous_count={} next_count={} alive_pids={:?}",
            previous_count, next_count, alive_pids
        );

        let scanned_by_pid = tasks
            .iter()
            .filter(|task| alive_pids_set.contains(&task.pid))
            .map(|task| (task.pid, task))
            .collect::<std::collections::HashMap<_, _>>();

        for hook_task in self.hook_tasks.values_mut() {
            if let Some(scanned_task) = scanned_by_pid.get(&hook_task.pid) {
                if hook_task.title == "workspace" || hook_task.title == "unknown" {
                    hook_task.title = scanned_task.title.clone();
                }
                if hook_task.window_target.host_kind == "unknown" {
                    hook_task.window_target = scanned_task.window_target.clone();
                }
            }
        }

        self.scanned_tasks = tasks
            .into_iter()
            .filter(|task| alive_pids_set.contains(&task.pid))
            .filter(|task| !self.closed_sessions.contains_key(&task.session_id))
            .filter(|task| {
                !self
                    .hook_tasks
                    .values()
                    .any(|hook| hook.pid == task.pid && hook.status == TaskStatus::Completed)
            })
            .map(|task| (task.task_id.clone(), task))
            .collect();

        let hook_pids_to_remove: Vec<String> = self
            .hook_tasks
            .values()
            .filter(|task| {
                task.liveness == TaskLiveness::Alive
                    && (!alive_pids_set.contains(&task.pid)
                        || (task.status == TaskStatus::Completed
                            && self
                                .closed_sessions
                                .contains_key(&task.session_id)
                            && self
                                .scanned_tasks
                                .values()
                                .all(|scanned| scanned.pid != task.pid)))
            })
            .map(|task| task.task_id.clone())
            .collect();

        for task_id in hook_pids_to_remove {
            eprintln!("[claudeBoard] removing dead hook task id={}", task_id);
            if let Some(task) = self.hook_tasks.get_mut(&task_id) {
                task.liveness = TaskLiveness::Dead;
                task.removed_at = Some(occurred_at.to_string());
                task.removed_reason = Some(crate::model::RemovalReason::ProcessExited);
            }
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: TaskSnapshot) {
        let TaskSnapshot {
            sessions,
            notifications,
            tasks,
            ..
        } = snapshot;

        self.session_records = sessions
            .iter()
            .filter(|record| !matches!(record.last_hook_event, HookEventType::SessionEnd))
            .cloned()
            .map(|record| (record.session_id.clone(), record))
            .collect();
        self.pending_notifications = notifications;
        self.next_notification_id = self
            .pending_notifications
            .iter()
            .map(|notification| notification.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.hook_tasks = tasks
            .into_iter()
            .filter(|task| task.source == "hook")
            .filter(|task| !is_subagent_task_id(&task.task_id, &task.session_id))
            .map(|task| (task.task_id.clone(), task))
            .collect();
        self.closed_sessions = sessions
            .into_iter()
            .filter(|record| matches!(record.last_hook_event, HookEventType::SessionEnd))
            .map(|record| (record.session_id, record.updated_at))
            .collect();
    }

    pub fn snapshot(&self) -> TaskSnapshot {
        let mut all_tasks = self
            .hook_tasks
            .values()
            .filter(|task| task.liveness == TaskLiveness::Alive)
            .cloned()
            .collect::<Vec<_>>();
        let hook_sessions = self
            .hook_tasks
            .values()
            .map(|task| (task.pid, task.session_id.as_str()))
            .collect::<std::collections::HashSet<_>>();
        let hook_pids = self
            .hook_tasks
            .values()
            .map(|task| task.pid)
            .collect::<std::collections::HashSet<_>>();

        all_tasks.extend(
            self.scanned_tasks
                .values()
                .filter(|task| {
                    !hook_sessions.contains(&(task.pid, task.session_id.as_str()))
                        && !(task.session_id.starts_with("local-") && hook_pids.contains(&task.pid))
                })
                .cloned(),
        );

        let completed_count = all_tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .count();

        let mut tasks = all_tasks;

        for task in &mut tasks {
            if let Some(record) = self.session_records.get(&task.session_id) {
                if !record.last_conversation_content.is_empty()
                    && (task.title == "workspace" || task.title == "unknown")
                {
                    task.title = record.last_conversation_content.clone();
                }
            }
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

        let mut counts = Self::counts_for(&tasks);
        counts.completed = completed_count;

        let mut sessions = self.session_records.values().cloned().collect::<Vec<_>>();
        sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

        TaskSnapshot {
            counts,
            tasks,
            sessions,
            notifications: self.pending_notifications.clone(),
        }
    }

    fn counts_for(tasks: &[TaskCard]) -> SnapshotCounts {
        tasks
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
            })
    }

    pub fn prune_dead_tasks(&mut self, now: &str) {
        let expired_session_ids = self
            .hook_tasks
            .values()
            .filter(|task| task.liveness == TaskLiveness::Dead)
            .filter_map(|task| {
                task.removed_at
                    .as_deref()
                    .filter(|removed_at| Self::is_dead_retention_expired(removed_at, now))
                    .map(|_| task.session_id.clone())
            })
            .collect::<std::collections::HashSet<_>>();

        if expired_session_ids.is_empty() {
            return;
        }

        self.hook_tasks
            .retain(|_, task| !expired_session_ids.contains(&task.session_id));
        self.session_records
            .retain(|session_id, _| !expired_session_ids.contains(session_id));
    }

    fn is_dead_retention_expired(removed_at: &str, now: &str) -> bool {
        match (
            time::OffsetDateTime::parse(removed_at, &time::format_description::well_known::Rfc3339),
            time::OffsetDateTime::parse(now, &time::format_description::well_known::Rfc3339),
        ) {
            (Ok(removed_at), Ok(now)) => now - removed_at > time::Duration::days(7),
            _ => false,
        }
    }

    fn push_hook_debug_entry(&mut self, entry: HookDebugEntry) {
        self.recent_hook_events.push(entry);
        if self.recent_hook_events.len() > MAX_HOOK_DEBUG_EVENTS {
            let overflow = self.recent_hook_events.len() - MAX_HOOK_DEBUG_EVENTS;
            self.recent_hook_events.drain(0..overflow);
        }
    }

    pub fn ack_notification(&mut self, notification_id: u64) -> bool {
        let previous_len = self.pending_notifications.len();
        self.pending_notifications
            .retain(|notification| notification.id != notification_id);
        self.pending_notifications.len() != previous_len
    }

    pub fn persisted_snapshot(&self) -> TaskSnapshot {
        let mut tasks = self
            .hook_tasks
            .values()
            .filter(|task| {
                (task.liveness == TaskLiveness::Alive || task.removed_at.is_some())
                    && !is_subagent_task_id(&task.task_id, &task.session_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        tasks.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        let mut sessions = self.session_records.values().cloned().collect::<Vec<_>>();
        sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

        TaskSnapshot {
            counts: Self::counts_for(&tasks),
            tasks,
            sessions,
            notifications: self.pending_notifications.clone(),
        }
    }

    fn enqueue_notification(&mut self, task: &TaskCard, occurred_at: &str) {
        let sound_type = match task.status {
            TaskStatus::NeedsUser => Some(NotificationSoundType::Waiting),
            TaskStatus::Completed => Some(NotificationSoundType::Completed),
            TaskStatus::NotStarted | TaskStatus::Running | TaskStatus::IdleOrUnknown => None,
        };

        if let Some(sound_type) = sound_type {
            let id = if self.next_notification_id == 0 {
                self.next_notification_id = 1;
                0
            } else {
                self.next_notification_id += 1;
                self.next_notification_id - 1
            };
            self.pending_notifications.push(NotificationEvent {
                id,
                session_id: task.session_id.clone(),
                task_id: task.task_id.clone(),
                status: task.status.clone(),
                sound_type,
                occurred_at: occurred_at.to_string(),
            });
        }
    }
}

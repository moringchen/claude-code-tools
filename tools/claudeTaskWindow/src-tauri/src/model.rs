use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookEventType {
    TaskCreated,
    TaskCompleted,
    PermissionRequest,
    PermissionDenied,
    SessionEnd,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HookEvent {
    pub event_type: HookEventType,
    pub session_id: String,
    pub pid: u32,
    pub title: String,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Running,
    NeedsUser,
    Completed,
    IdleOrUnknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowTarget {
    pub host_kind: String,
    pub app: String,
    pub descriptor: String,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCard {
    pub task_id: String,
    pub session_id: String,
    pub pid: u32,
    pub title: String,
    pub status: TaskStatus,
    pub source: String,
    pub window_target: WindowTarget,
    pub started_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnapshotCounts {
    pub total: usize,
    pub needs_user: usize,
    pub completed: usize,
    pub running: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TaskSnapshot {
    pub counts: SnapshotCounts,
    pub tasks: Vec<TaskCard>,
}

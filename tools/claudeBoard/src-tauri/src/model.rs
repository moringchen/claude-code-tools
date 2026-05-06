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
    pub agent_id: Option<String>,
    pub pid: u32,
    pub title: String,
    pub conversation_content: Option<String>,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    Running,
    NeedsUser,
    Completed,
    IdleOrUnknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub last_conversation_content: String,
    pub last_hook_event: HookEventType,
    pub last_status: TaskStatus,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowTarget {
    pub host_kind: String,
    pub app: String,
    pub descriptor: String,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskLiveness {
    Alive,
    Dead,
}

fn default_task_liveness() -> TaskLiveness {
    TaskLiveness::Alive
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemovalReason {
    ProcessExited,
    ShellClosed,
    TerminalClosed,
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
    #[serde(default = "default_task_liveness")]
    pub liveness: TaskLiveness,
    #[serde(default)]
    pub removed_at: Option<String>,
    #[serde(default)]
    pub removed_reason: Option<RemovalReason>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSoundType {
    Waiting,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationEvent {
    pub id: u64,
    pub session_id: String,
    pub task_id: String,
    pub status: TaskStatus,
    pub sound_type: NotificationSoundType,
    pub occurred_at: String,
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
    #[serde(default)]
    pub sessions: Vec<SessionRecord>,
    #[serde(default)]
    pub notifications: Vec<NotificationEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookDebugDisposition {
    Accepted,
    Filtered,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HookDebugEntry {
    pub occurred_at: String,
    pub hook_event_name: String,
    pub session_id: String,
    pub pid: u32,
    pub title: String,
    pub permission_mode: Option<String>,
    pub prompt_preview: Option<String>,
    pub agent_id: Option<String>,
    pub disposition: HookDebugDisposition,
    pub mapped_event_type: Option<HookEventType>,
    pub filter_reason: Option<String>,
    pub previous_status: Option<TaskStatus>,
    pub next_status: Option<TaskStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanDecision {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanDebugEntry {
    pub pid: Option<u32>,
    pub ppid: Option<u32>,
    pub state: Option<String>,
    pub command: String,
    pub decision: ScanDecision,
    pub reason: Option<String>,
    pub accepted_row: Option<String>,
    pub task: Option<TaskCard>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ScanDebugSnapshot {
    pub occurred_at: String,
    pub entries: Vec<ScanDebugEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugSnapshot {
    pub snapshot: TaskSnapshot,
    pub recent_hook_events: Vec<HookDebugEntry>,
    pub latest_scan: ScanDebugSnapshot,
}

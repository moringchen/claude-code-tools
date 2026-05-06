import type { Preferences } from "./settings";
import type { TaskCard, TaskCounts, TaskStatus, WindowTarget } from "./task-model";

export const BASE_URL =
  (globalThis as { __CLAUDE_BOARD_BASE_URL__?: string }).__CLAUDE_BOARD_BASE_URL__ ??
  "http://127.0.0.1:46123";

type SnapshotDto = {
  counts: TaskCountsDto;
  tasks: TaskCardDto[];
  notifications: NotificationEventDto[];
};

type TaskCountsDto = {
  total: number;
  needs_user: number;
  completed: number;
  running: number;
};

type TaskCardDto = {
  task_id: string;
  session_id: string;
  pid: number;
  title: string;
  status: TaskCard["status"];
  source: TaskCard["source"];
  window_target: WindowTargetDto;
  started_at: string;
  updated_at: string;
  completed_at: string | null;
};

type WindowTargetDto = {
  host_kind: WindowTarget["hostKind"];
  app: string;
  descriptor: string;
  tab_id?: string;
  pane_id?: string;
};

type NotificationEventDto = {
  id: number;
  session_id: string;
  task_id: string;
  status: TaskCard["status"];
  sound_type: NotificationSoundType;
  occurred_at: string;
};

type HookEventType =
  | "task_created"
  | "task_completed"
  | "permission_request"
  | "permission_denied"
  | "session_end";

type HookDebugDisposition = "accepted" | "filtered";
type ScanDecision = "accepted" | "rejected";

type HookDebugEntryDto = {
  occurred_at: string;
  hook_event_name: string;
  session_id: string;
  pid: number;
  title: string;
  permission_mode?: string | null;
  prompt_preview?: string | null;
  agent_id?: string | null;
  disposition: HookDebugDisposition;
  mapped_event_type?: HookEventType | null;
  filter_reason?: string | null;
  previous_status?: TaskStatus | null;
  next_status?: TaskStatus | null;
};

type ScanDebugEntryDto = {
  pid?: number | null;
  ppid?: number | null;
  state?: string | null;
  command: string;
  decision: ScanDecision;
  reason?: string | null;
  accepted_row?: string | null;
  task?: TaskCardDto | null;
};

type ScanDebugSnapshotDto = {
  occurred_at: string;
  entries: ScanDebugEntryDto[];
};

type DebugSnapshotDto = {
  snapshot: SnapshotDto;
  recent_hook_events: HookDebugEntryDto[];
  latest_scan: ScanDebugSnapshotDto;
};

export type NotificationSoundType = "waiting" | "completed";

export type NotificationEvent = {
  id: number;
  sessionId: string;
  taskId: string;
  status: TaskCard["status"];
  soundType: NotificationSoundType;
  occurredAt: string;
};

export type Snapshot = {
  counts: TaskCounts;
  tasks: TaskCard[];
  notifications: NotificationEvent[];
};

export type HookDebugEntry = {
  occurredAt: string;
  hookEventName: string;
  sessionId: string;
  pid: number;
  title: string;
  permissionMode?: string;
  promptPreview?: string;
  agentId?: string;
  disposition: HookDebugDisposition;
  mappedEventType?: HookEventType;
  filterReason?: string;
  previousStatus?: TaskStatus;
  nextStatus?: TaskStatus;
};

export type ScanDebugEntry = {
  pid?: number;
  ppid?: number;
  state?: string;
  command: string;
  decision: ScanDecision;
  reason?: string;
  acceptedRow?: string;
  task?: TaskCard;
};

export type ScanDebugSnapshot = {
  occurredAt: string;
  entries: ScanDebugEntry[];
};

export type DebugSnapshot = {
  snapshot: Snapshot;
  recentHookEvents: HookDebugEntry[];
  latestScan: ScanDebugSnapshot;
};

function normalizeCounts(counts: TaskCountsDto): TaskCounts {
  return {
    total: counts.total,
    needsUser: counts.needs_user,
    completed: counts.completed,
    running: counts.running,
  };
}

function normalizeWindowTarget(windowTarget: WindowTargetDto): WindowTarget {
  return {
    hostKind: windowTarget.host_kind,
    app: windowTarget.app,
    descriptor: windowTarget.descriptor,
    tabId: windowTarget.tab_id,
    paneId: windowTarget.pane_id,
  };
}

function normalizeTask(task: TaskCardDto): TaskCard {
  return {
    taskId: task.task_id,
    sessionId: task.session_id,
    pid: task.pid,
    title: task.title,
    status: task.status,
    source: task.source,
    windowTarget: normalizeWindowTarget(task.window_target),
    startedAt: task.started_at,
    updatedAt: task.updated_at,
    completedAt: task.completed_at,
  };
}

function normalizeNotification(notification: NotificationEventDto): NotificationEvent {
  return {
    id: notification.id,
    sessionId: notification.session_id,
    taskId: notification.task_id,
    status: notification.status,
    soundType: notification.sound_type,
    occurredAt: notification.occurred_at,
  };
}

function normalizeSnapshot(snapshot: SnapshotDto): Snapshot {
  return {
    counts: normalizeCounts(snapshot.counts),
    tasks: snapshot.tasks.map(normalizeTask),
    notifications: (snapshot.notifications ?? []).map(normalizeNotification),
  };
}

function normalizeHookDebugEntry(entry: HookDebugEntryDto): HookDebugEntry {
  return {
    occurredAt: entry.occurred_at,
    hookEventName: entry.hook_event_name,
    sessionId: entry.session_id,
    pid: entry.pid,
    title: entry.title,
    permissionMode: entry.permission_mode ?? undefined,
    promptPreview: entry.prompt_preview ?? undefined,
    agentId: entry.agent_id ?? undefined,
    disposition: entry.disposition,
    mappedEventType: entry.mapped_event_type ?? undefined,
    filterReason: entry.filter_reason ?? undefined,
    previousStatus: entry.previous_status ?? undefined,
    nextStatus: entry.next_status ?? undefined,
  };
}

function normalizeScanDebugEntry(entry: ScanDebugEntryDto): ScanDebugEntry {
  return {
    pid: entry.pid ?? undefined,
    ppid: entry.ppid ?? undefined,
    state: entry.state ?? undefined,
    command: entry.command,
    decision: entry.decision,
    reason: entry.reason ?? undefined,
    acceptedRow: entry.accepted_row ?? undefined,
    task: entry.task ? normalizeTask(entry.task) : undefined,
  };
}

function normalizeDebugSnapshot(snapshot: DebugSnapshotDto): DebugSnapshot {
  return {
    snapshot: normalizeSnapshot(snapshot.snapshot),
    recentHookEvents: snapshot.recent_hook_events.map(normalizeHookDebugEntry),
    latestScan: {
      occurredAt: snapshot.latest_scan.occurred_at,
      entries: snapshot.latest_scan.entries.map(normalizeScanDebugEntry),
    },
  };
}

export async function fetchSnapshot(): Promise<Snapshot> {
  const response = await fetch(`${BASE_URL}/snapshot`);
  if (!response.ok) {
    throw new Error(`Failed to fetch snapshot: ${response.status}`);
  }

  const snapshot = (await response.json()) as SnapshotDto;
  return normalizeSnapshot(snapshot);
}

export async function fetchDebugSnapshot(): Promise<DebugSnapshot> {
  const response = await fetch(`${BASE_URL}/debug/snapshot`);
  if (!response.ok) {
    throw new Error(`Failed to fetch debug snapshot: ${response.status}`);
  }

  const snapshot = (await response.json()) as DebugSnapshotDto;
  return normalizeDebugSnapshot(snapshot);
}

export async function ackNotification(notificationId: number): Promise<void> {
  const response = await fetch(`${BASE_URL}/notifications/${notificationId}/ack`, {
    method: "POST",
  });

  if (!response.ok) {
    throw new Error(`Failed to acknowledge notification: ${response.status}`);
  }
}

export async function focusTask(taskId: string): Promise<void> {
  const response = await fetch(`${BASE_URL}/tasks/${encodeURIComponent(taskId)}/focus`, {
    method: "POST",
  });

  if (!response.ok) {
    throw new Error(`Failed to focus task: ${response.status}`);
  }
}

export async function savePreferencesRemote(preferences: Preferences): Promise<void> {
  const response = await fetch(`${BASE_URL}/preferences`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(preferences),
  });

  if (!response.ok) {
    throw new Error(`Failed to save preferences: ${response.status}`);
  }
}

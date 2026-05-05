import type { Preferences } from "./settings";
import type { TaskCard, TaskCounts, WindowTarget } from "./task-model";

export const BASE_URL =
  (globalThis as { __CLAUDE_BOARD_BASE_URL__?: string }).__CLAUDE_BOARD_BASE_URL__ ??
  "http://127.0.0.1:46123";

type SnapshotDto = {
  counts: TaskCountsDto;
  tasks: TaskCardDto[];
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

export type Snapshot = {
  counts: TaskCounts;
  tasks: TaskCard[];
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

function normalizeSnapshot(snapshot: SnapshotDto): Snapshot {
  return {
    counts: normalizeCounts(snapshot.counts),
    tasks: snapshot.tasks.map(normalizeTask),
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

export type TaskStatus = "running" | "needs_user" | "completed" | "idle_or_unknown";
export type TaskSource = "hook" | "scan_recovered";

export type WindowTarget = {
  hostKind: "terminal" | "tmux" | "ide" | "unknown";
  app: string;
  descriptor: string;
  tabId?: string;
  paneId?: string;
};

export type TaskCard = {
  taskId: string;
  sessionId: string;
  pid: number;
  title: string;
  status: TaskStatus;
  source: TaskSource;
  windowTarget: WindowTarget;
  startedAt: string;
  updatedAt: string;
  completedAt: string | null;
};

export type TaskCounts = {
  total: number;
  needsUser: number;
  completed: number;
  running: number;
};

const statusRank: Record<TaskStatus, number> = {
  needs_user: 0,
  completed: 1,
  running: 2,
  idle_or_unknown: 3,
};

export function sortTasks(tasks: TaskCard[]): TaskCard[] {
  return [...tasks].sort((left, right) => {
    const byStatus = statusRank[left.status] - statusRank[right.status];
    if (byStatus !== 0) {
      return byStatus;
    }
    return right.updatedAt.localeCompare(left.updatedAt);
  });
}

export function buildCounts(tasks: TaskCard[]): TaskCounts {
  return tasks.reduce<TaskCounts>(
    (counts, task) => {
      counts.total += 1;
      if (task.status === "needs_user") counts.needsUser += 1;
      if (task.status === "completed") counts.completed += 1;
      if (task.status === "running") counts.running += 1;
      return counts;
    },
    { total: 0, needsUser: 0, completed: 0, running: 0 },
  );
}

import type { TaskCard } from "./task-model";

export type VisualStatus = "waiting" | "running" | "completed" | "idle";
export type ResolvedVisualStatus = VisualStatus | null;

const labels: Record<VisualStatus, string> = {
  waiting: "待回复",
  running: "进行中",
  completed: "完成",
  idle: "空闲",
};

export function resolveVisualStatus(task: TaskCard): ResolvedVisualStatus {
  switch (task.status) {
    case "running":
      return "running";
    case "needs_user":
      return "waiting";
    case "completed":
      return "completed";
    case "idle_or_unknown":
      return "idle";
    default:
      return null;
  }
}

export function visualStatusLabel(status: VisualStatus): string {
  return labels[status];
}

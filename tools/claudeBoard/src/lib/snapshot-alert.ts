import type { TaskCard } from "./task-model";
import { resolveVisualStatus, type VisualStatus } from "./visual-status";

export type SnapshotAlert = Extract<VisualStatus, "waiting" | "completed"> | null;

export function chooseSnapshotAlert(tasks: TaskCard[]): SnapshotAlert {
  let hasCompleted = false;

  for (const task of tasks) {
    const status = resolveVisualStatus(task);
    if (status === "waiting") {
      return "waiting";
    }
    if (status === "completed") {
      hasCompleted = true;
    }
  }

  return hasCompleted ? "completed" : null;
}

import { describe, expect, it } from "vitest";
import { buildCounts, sortTasks, type TaskCard } from "./task-model";

function makeTask(
  id: string,
  status: TaskCard["status"],
  updatedAt: string,
): TaskCard {
  return {
    taskId: id,
    sessionId: "session-1",
    pid: 101,
    title: `Task ${id}`,
    status,
    source: "hook",
    windowTarget: {
      hostKind: "terminal",
      app: "Ghostty",
      descriptor: `ghostty:${id}`,
    },
    startedAt: updatedAt,
    updatedAt,
    completedAt: status === "completed" ? updatedAt : null,
  };
}

describe("task-model", () => {
  it("sorts needs_user before running before completed before idle_or_unknown", () => {
    const tasks = [
      makeTask("4", "idle_or_unknown", "2026-04-24T15:03:00Z"),
      makeTask("3", "running", "2026-04-24T15:00:00Z"),
      makeTask("2", "completed", "2026-04-24T15:01:00Z"),
      makeTask("1", "needs_user", "2026-04-24T15:02:00Z"),
    ];

    expect(sortTasks(tasks).map((task) => task.taskId)).toEqual(["1", "3", "2", "4"]);
  });

  it("builds aggregate counts", () => {
    const tasks = [
      makeTask("1", "needs_user", "2026-04-24T15:02:00Z"),
      makeTask("2", "completed", "2026-04-24T15:01:00Z"),
      makeTask("3", "running", "2026-04-24T15:00:00Z"),
    ];

    expect(buildCounts(tasks)).toEqual({
      total: 3,
      needsUser: 1,
      completed: 1,
      running: 1,
    });
  });
});

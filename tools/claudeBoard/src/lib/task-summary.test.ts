import { describe, expect, it } from "vitest";
import type { TaskCard } from "./task-model";
import { buildIslandSummaries, currentIslandSummary } from "./task-summary";

const task = (id: string, status: TaskCard["status"], title: string): TaskCard => ({
  taskId: id,
  sessionId: `session-${id}`,
  pid: Number(id),
  title,
  status,
  source: "hook",
  windowTarget: {
    hostKind: "terminal",
    app: "Ghostty",
    descriptor: "main",
  },
  startedAt: "2026-04-30T00:00:00Z",
  updatedAt: "2026-04-30T00:00:00Z",
  completedAt: status === "completed" ? "2026-04-30T00:00:01Z" : null,
});

describe("task-summary", () => {
  it("shows idle text when there are no tasks", () => {
    expect(currentIslandSummary([], 0)).toBe("当前无任务");
  });

  it("prioritizes needs-user tasks before running tasks", () => {
    const summaries = buildIslandSummaries([
      task("1", "running", "build frontend"),
      task("3", "needs_user", "approve command"),
    ]);

    expect(summaries).toEqual([
      "approve command - 待回复",
      "build frontend - 进行中",
    ]);
  });

  it("uses the highest-priority visible status for the collapsed title", () => {
    const tasks = [
      { ...task("1", "needs_user", "older approval"), updatedAt: "2026-04-30T00:00:01Z" },
      { ...task("2", "running", "newer running"), updatedAt: "2026-04-30T00:00:02Z" },
    ];

    expect(currentIslandSummary(tasks, 0)).toBe("older approval - 待回复");
  });

  it("ignores the cycle index and keeps the same highest-priority summary visible", () => {
    const tasks = [
      { ...task("1", "running", "older task"), updatedAt: "2026-04-30T00:00:01Z" },
      { ...task("2", "running", "newest task"), updatedAt: "2026-04-30T00:00:02Z" },
    ];

    expect(currentIslandSummary(tasks, 0)).toBe("newest task - 进行中");
    expect(currentIslandSummary(tasks, 1)).toBe("newest task - 进行中");
    expect(currentIslandSummary(tasks, 2)).toBe("newest task - 进行中");
  });

  it("omits the status label for not-started tasks", () => {
    const tasks = [{ ...task("1", "not_started", "planned task"), updatedAt: "2026-04-30T00:00:02Z" }];

    expect(currentIslandSummary(tasks, 0)).toBe("planned task");
    expect(buildIslandSummaries(tasks)).toEqual(["planned task"]);
  });
});

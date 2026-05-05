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

  it("prioritizes needs-user tasks before running and completed tasks", () => {
    const summaries = buildIslandSummaries([
      task("1", "running", "build frontend"),
      task("2", "completed", "write docs"),
      task("3", "needs_user", "approve command"),
    ]);

    expect(summaries).toEqual([
      "需确认：approve command",
      "运行中：build frontend",
      "已完成：write docs",
    ]);
  });

  it("cycles summaries by index", () => {
    const tasks = [
      task("1", "running", "first task"),
      task("2", "running", "second task"),
    ];

    expect(currentIslandSummary(tasks, 0)).toBe("运行中：first task");
    expect(currentIslandSummary(tasks, 1)).toBe("运行中：second task");
    expect(currentIslandSummary(tasks, 2)).toBe("运行中：first task");
  });
});

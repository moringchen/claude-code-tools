import { type TaskCard, type TaskStatus } from "./task-model";

const statusLabel: Record<TaskStatus, string> = {
  needs_user: "需确认",
  running: "运行中",
  completed: "已完成",
  idle_or_unknown: "任务",
};

const summaryRank: Record<TaskStatus, number> = {
  needs_user: 0,
  running: 1,
  completed: 2,
  idle_or_unknown: 3,
};

export function buildIslandSummaries(tasks: TaskCard[]): string[] {
  return [...tasks]
    .sort((left, right) => {
      const byStatus = summaryRank[left.status] - summaryRank[right.status];
      if (byStatus !== 0) {
        return byStatus;
      }
      return right.updatedAt.localeCompare(left.updatedAt);
    })
    .map((task) => `${statusLabel[task.status]}：${task.title}`);
}

export function currentIslandSummary(tasks: TaskCard[], index: number): string {
  const summaries = buildIslandSummaries(tasks);
  if (summaries.length === 0) {
    return "当前无任务";
  }

  const currentIndex = ((index % summaries.length) + summaries.length) % summaries.length;
  return summaries[currentIndex];
}

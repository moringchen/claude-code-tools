import { type TaskCard, type TaskStatus } from "./task-model";

const statusLabel: Record<TaskStatus, string> = {
  not_started: "未开始",
  needs_user: "待回复",
  running: "进行中",
  completed: "完成",
  idle_or_unknown: "空闲",
};

const summaryRank: Record<TaskStatus, number> = {
  needs_user: 0,
  running: 1,
  not_started: 2,
  completed: 3,
  idle_or_unknown: 4,
};

function titleWithStatus(task: TaskCard): string {
  if (task.status === "not_started") {
    return task.title;
  }

  return `${task.title} - ${statusLabel[task.status]}`;
}

export function buildIslandSummaries(tasks: TaskCard[]): string[] {
  return [...tasks]
    .sort((left, right) => {
      const byStatus = summaryRank[left.status] - summaryRank[right.status];
      if (byStatus !== 0) {
        return byStatus;
      }
      return right.updatedAt.localeCompare(left.updatedAt);
    })
    .map(titleWithStatus);
}

export function buildCompactSummary(tasks: TaskCard[]): string {
  const waiting = tasks.filter((task) => task.status === "needs_user").length;
  const completed = tasks.filter((task) => task.status === "completed").length;
  return `待 ${waiting} / 完 ${completed}`;
}

export function currentIslandSummary(tasks: TaskCard[], _index: number): string {
  if (tasks.length === 0) {
    return "当前无任务";
  }

  const [highestPriorityTask] = [...tasks].sort((left, right) => {
    const byStatus = summaryRank[left.status] - summaryRank[right.status];
    if (byStatus !== 0) {
      return byStatus;
    }
    return right.updatedAt.localeCompare(left.updatedAt);
  });

  return titleWithStatus(highestPriorityTask);
}

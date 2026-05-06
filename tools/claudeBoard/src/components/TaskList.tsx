import type { TaskCard } from "../lib/task-model";
import { resolveVisualStatus, visualStatusLabel } from "../lib/visual-status";
import { StatusIcon } from "./StatusIcon";

type TaskListProps = {
  tasks: TaskCard[];
  onTaskClick: (task: TaskCard) => void;
};

export function TaskList({ tasks, onTaskClick }: TaskListProps) {
  console.log("[TaskList] render:", {
    taskCount: tasks.length,
    tasks: tasks.map((task) => ({
      taskId: task.taskId,
      sessionId: task.sessionId,
      title: task.title,
      status: task.status,
      source: task.source,
      pid: task.pid,
    })),
  });

  return (
    <ul className="island-task-list">
      {tasks.map((task) => {
        const visualStatus = resolveVisualStatus(task);
        return (
          <li key={task.taskId} className="island-task-item">
            <button type="button" className="island-task-row" onClick={() => onTaskClick(task)}>
              <span className="task-title">{task.title}</span>
              <span className="task-status-row">
                {visualStatus && <StatusIcon status={visualStatus} />}
                <span className="task-status-label">{visualStatus ? visualStatusLabel(visualStatus) : ""}</span>
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

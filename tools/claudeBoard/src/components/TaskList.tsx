import type { TaskCard } from "../lib/task-model";

type TaskListProps = {
  tasks: TaskCard[];
  onTaskClick: (task: TaskCard) => void;
};

export function TaskList({ tasks, onTaskClick }: TaskListProps) {
  return (
    <ul className="island-task-list">
      {tasks.map((task) => (
        <li key={task.taskId} className="island-task-item">
          <button type="button" className="island-task-row" onClick={() => onTaskClick(task)}>
            <span className="task-title">{task.title}</span>
            <span className="task-status">{task.status}</span>
            <span className="task-target-app">{task.windowTarget.app}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}

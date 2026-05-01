import type { TaskCard } from "../lib/task-model";

type TaskListProps = {
  tasks: TaskCard[];
  onTaskClick: (task: TaskCard) => void;
};

export function TaskList({ tasks, onTaskClick }: TaskListProps) {
  return (
    <div className="task-list" role="list">
      {tasks.map((task) => (
        <div key={task.taskId} role="listitem">
          <button type="button" className="task-row" onClick={() => onTaskClick(task)}>
            <span className="task-title">{task.title}</span>
            <span className="task-status">{task.status}</span>
            <span className="task-target-app">{task.windowTarget.app}</span>
            <span className="task-target-descriptor">{task.windowTarget.descriptor}</span>
          </button>
        </div>
      ))}
    </div>
  );
}

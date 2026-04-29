import { useMemo, useState } from "react";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { buildCounts, sortTasks, type TaskCard } from "./lib/task-model";

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [tasks] = useState<TaskCard[]>([]);

  const sortedTasks = useMemo(() => sortTasks(tasks), [tasks]);
  const counts = useMemo(() => buildCounts(sortedTasks), [sortedTasks]);

  return (
    <div className="overlay-shell">
      <OverlayBar
        counts={counts}
        isExpanded={expanded}
        onToggle={() => setExpanded((current) => !current)}
      />
      {expanded ? (
        <TaskList tasks={sortedTasks} onTaskClick={() => {}} />
      ) : null}
    </div>
  );
}

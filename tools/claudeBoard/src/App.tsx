import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { focusTask } from "./lib/api";
import { sortTasks } from "./lib/task-model";
import { currentIslandSummary } from "./lib/task-summary";
import { useSnapshot } from "./lib/use-snapshot";

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [summaryIndex, setSummaryIndex] = useState(0);
  const snapshot = useSnapshot();

  const sortedTasks = useMemo(() => sortTasks(snapshot.tasks), [snapshot.tasks]);
  const summary = useMemo(
    () => currentIslandSummary(sortedTasks, summaryIndex),
    [sortedTasks, summaryIndex],
  );

  useEffect(() => {
    if (sortedTasks.length <= 1) {
      setSummaryIndex(0);
      return;
    }

    const timer = window.setInterval(() => {
      setSummaryIndex((current) => current + 1);
    }, 2500);

    return () => window.clearInterval(timer);
  }, [sortedTasks.length]);

  const startDrag = () => {
    void getCurrentWindow().startDragging();
  };

  return (
    <div className={expanded ? "island-shell island-shell-expanded" : "island-shell"}>
      <OverlayBar
        summary={summary}
        isExpanded={expanded}
        onToggle={() => setExpanded((current) => !current)}
        onDragStart={startDrag}
      />
      {expanded ? <TaskList tasks={sortedTasks} onTaskClick={(task) => void focusTask(task.taskId)} /> : null}
    </div>
  );
}

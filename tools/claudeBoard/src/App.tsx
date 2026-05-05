import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { focusTask } from "./lib/api";
import { createOverlayLogicalSize } from "./lib/overlay-window";
import { sortTasks } from "./lib/task-model";
import { currentIslandSummary } from "./lib/task-summary";
import { useSnapshot } from "./lib/use-snapshot";

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [summaryIndex] = useState(0);
  const snapshot = useSnapshot();

  const sortedTasks = useMemo(() => sortTasks(snapshot.tasks), [snapshot.tasks]);
  const summary = useMemo(
    () => currentIslandSummary(sortedTasks, summaryIndex),
    [sortedTasks, summaryIndex],
  );

  const startDrag = () => {
    void getCurrentWindow().startDragging();
  };

  // 根据展开状态和任务数量调整窗口大小（宽度固定为260px）
  useEffect(() => {
    const updateWindowSize = async () => {
      const window = getCurrentWindow();
      await window.setSize(createOverlayLogicalSize(expanded, sortedTasks.length));
    };

    void updateWindowSize();
  }, [expanded, sortedTasks.length]);

  useEffect(() => {
    const preventContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };

    document.addEventListener("contextmenu", preventContextMenu);
    return () => document.removeEventListener("contextmenu", preventContextMenu);
  }, []);

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

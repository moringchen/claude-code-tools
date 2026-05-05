import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { focusTask } from "./lib/api";
import { playSound, getUserInteractionStatus, getDebugLogs } from "./lib/sound";
import { sortTasks } from "./lib/task-model";
import { currentIslandSummary } from "./lib/task-summary";
import { useSnapshot } from "./lib/use-snapshot";

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [summaryIndex, setSummaryIndex] = useState(0);
  const [showLogs, setShowLogs] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const snapshot = useSnapshot();
  const prevCountsRef = useRef({ needsUser: 0, completed: 0 });

  const sortedTasks = useMemo(() => sortTasks(snapshot.tasks), [snapshot.tasks]);
  const summary = useMemo(
    () => currentIslandSummary(sortedTasks, summaryIndex),
    [sortedTasks, summaryIndex],
  );

  // 播放音效当任务状态变化时
  useEffect(() => {
    const currentNeedsUser = snapshot.counts.needsUser;
    const currentCompleted = snapshot.counts.completed;
    const prev = prevCountsRef.current;

    console.log("[App] Sound effect check:", {
      currentNeedsUser,
      currentCompleted,
      prevNeedsUser: prev.needsUser,
      prevCompleted: prev.completed,
      hasUserInteraction: getUserInteractionStatus(),
    });

    if (currentNeedsUser > prev.needsUser) {
      console.log("[App] Triggering waiting sound, needsUser increased from", prev.needsUser, "to", currentNeedsUser);
      void playSound("waiting");
    }
    if (currentCompleted > prev.completed) {
      console.log("[App] Triggering completed sound, completed increased from", prev.completed, "to", currentCompleted);
      void playSound("completed");
    }

    prevCountsRef.current = {
      needsUser: currentNeedsUser,
      completed: currentCompleted,
    };
  }, [snapshot.counts]);

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

  // 根据展开状态和任务数量调整窗口大小（宽度固定为260px）
  useEffect(() => {
    const updateWindowSize = async () => {
      const window = getCurrentWindow();
      const width = 260;
      const baseHeight = 42; // 浮条高度 (缩小20%)
      const taskItemHeight = 48; // 每个任务项高度
      const taskListPadding = 20; // 列表上下padding

      if (expanded && sortedTasks.length > 0) {
        const height = baseHeight + (sortedTasks.length * taskItemHeight) + taskListPadding;
        await window.setSize(new LogicalSize(width, height));
      } else {
        await window.setSize(new LogicalSize(width, baseHeight));
      }
    };

    void updateWindowSize();
  }, [expanded, sortedTasks.length]);

  const refreshLogs = () => {
    setLogs(getDebugLogs());
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
      {/* Debug buttons */}
      <div style={{ position: "fixed", bottom: 0, right: 0, zIndex: 9999, background: "rgba(0,0,0,0.8)", padding: "5px", borderRadius: "4px" }}>
        <button onClick={() => { void playSound("waiting"); refreshLogs(); }} style={{ margin: "2px", fontSize: "10px" }}>Test Waiting</button>
        <button onClick={() => { void playSound("completed"); refreshLogs(); }} style={{ margin: "2px", fontSize: "10px" }}>Test Completed</button>
        <button onClick={() => { setShowLogs(!showLogs); refreshLogs(); }} style={{ margin: "2px", fontSize: "10px" }}>{showLogs ? "Hide Logs" : "Show Logs"}</button>
        {showLogs && (
          <div style={{ position: "absolute", bottom: "30px", right: 0, width: "400px", maxHeight: "200px", overflow: "auto", background: "rgba(0,0,0,0.9)", color: "#0f0", fontSize: "9px", padding: "5px", fontFamily: "monospace" }}>
            {logs.length === 0 ? "No logs yet..." : logs.map((l, i) => <div key={i}>{l}</div>)}
          </div>
        )}
      </div>
    </div>
  );
}

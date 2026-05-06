import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { ackNotification, focusTask } from "./lib/api";
import { playSound } from "./lib/sound";
import { createOverlayLogicalSize } from "./lib/overlay-window";
import { currentIslandSummary } from "./lib/task-summary";
import { useSnapshot } from "./lib/use-snapshot";
import { sortTasks } from "./lib/task-model";

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [summaryIndex] = useState(0);
  const snapshot = useSnapshot();
  const playedNotificationIds = useRef<Set<number>>(new Set());
  const notifications = snapshot.notifications ?? [];

  const visibleTasks = useMemo(() => snapshot.tasks, [snapshot.tasks]);
  const sortedTasks = useMemo(() => sortTasks(visibleTasks), [visibleTasks]);
  const summary = useMemo(
    () => currentIslandSummary(sortedTasks, summaryIndex),
    [sortedTasks, summaryIndex],
  );

  const startDrag = () => {
    void getCurrentWindow().startDragging();
  };

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

  useEffect(() => {
    const currentNotificationIds = new Set(notifications.map((notification) => notification.id));
    for (const notificationId of playedNotificationIds.current) {
      if (!currentNotificationIds.has(notificationId)) {
        playedNotificationIds.current.delete(notificationId);
      }
    }

    for (const notification of notifications) {
      if (playedNotificationIds.current.has(notification.id)) {
        continue;
      }
      playedNotificationIds.current.add(notification.id);

      void playSound(notification.soundType)
        .then(() => ackNotification(notification.id))
        .catch((error) => {
          playedNotificationIds.current.delete(notification.id);
          console.error("[sound] playback failed:", error);
        });
    }
  }, [notifications]);

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

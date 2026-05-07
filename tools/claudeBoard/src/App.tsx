import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { OverlayBar } from "./components/OverlayBar";
import { TaskList } from "./components/TaskList";
import { ackNotification, focusTask } from "./lib/api";
import { playSound } from "./lib/sound";
import { createOverlayLogicalSize, type OverlayMode } from "./lib/overlay-window";
import { buildCompactSummary, currentIslandSummary } from "./lib/task-summary";
import { useSnapshot } from "./lib/use-snapshot";
import { sortTasks } from "./lib/task-model";

export default function App() {
  const [overlayMode, setOverlayMode] = useState<OverlayMode>("collapsed");
  const [summaryIndex] = useState(0);
  const snapshot = useSnapshot();
  const playedNotificationIds = useRef<Set<number>>(new Set());
  const notifications = snapshot.notifications ?? [];

  const visibleTasks = useMemo(() => snapshot.tasks, [snapshot.tasks]);
  const sortedTasks = useMemo(() => sortTasks(visibleTasks), [visibleTasks]);
  const summary = useMemo(() => {
    if (overlayMode === "compact") {
      return buildCompactSummary(sortedTasks);
    }

    return currentIslandSummary(sortedTasks, summaryIndex);
  }, [overlayMode, sortedTasks, summaryIndex]);
  const isExpanded = overlayMode === "expanded";
  const isCompact = overlayMode === "compact";

  const startDrag = () => {
    void getCurrentWindow().startDragging();
  };

  useEffect(() => {
    const updateWindowSize = async () => {
      const window = getCurrentWindow();
      await window.setSize(createOverlayLogicalSize(overlayMode, sortedTasks.length));
    };

    void updateWindowSize();
  }, [overlayMode, sortedTasks.length]);

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
    <div
      className={[
        "island-shell",
        isExpanded ? "island-shell-expanded" : "",
        isCompact ? "island-shell-compact" : "",
      ].filter(Boolean).join(" ")}
    >
      <OverlayBar
        summary={summary}
        isExpanded={isExpanded}
        isCompact={isCompact}
        onToggle={() => setOverlayMode((current) => {
          if (current === "compact") {
            return "collapsed";
          }

          return current === "expanded" ? "collapsed" : "expanded";
        })}
        onEnterCompactMode={() => setOverlayMode("compact")}
        onDragStart={startDrag}
      />
      {isExpanded ? <TaskList tasks={sortedTasks} onTaskClick={(task) => void focusTask(task.taskId)} /> : null}
    </div>
  );
}

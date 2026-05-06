import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Snapshot } from "./lib/api";
import App from "./App";
import { ackNotification, focusTask } from "./lib/api";
import { playSound } from "./lib/sound";
import { useSnapshot } from "./lib/use-snapshot";

const startDragging = vi.fn();
const setSize = vi.fn();

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ startDragging, setSize }),
  LogicalSize: class LogicalSize {
    constructor(public width: number, public height: number) {}
  },
}));

vi.mock("./lib/api", () => ({
  focusTask: vi.fn(),
  ackNotification: vi.fn(),
}));

vi.mock("./lib/use-snapshot", () => ({
  useSnapshot: vi.fn(),
}));

vi.mock("./lib/sound", () => ({
  playSound: vi.fn(),
  markUserInteraction: vi.fn(),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const runningTask = {
  taskId: "task-1",
  sessionId: "session-1",
  pid: 123,
  title: "测试任务",
  status: "running" as const,
  source: "hook" as const,
  windowTarget: {
    hostKind: "terminal" as const,
    app: "Ghostty",
    descriptor: "main",
  },
  startedAt: "2026-04-30T00:00:00Z",
  updatedAt: "2026-04-30T00:00:01Z",
  completedAt: null,
};

const emptyNotifications: Snapshot["notifications"] = [];

describe("App", () => {
  it("renders the overlay even without window-label-based routing", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    });

    render(<App />);
    expect(screen.getByText("测试任务 - 进行中")).toBeTruthy();
  });

  it("plays and acknowledges waiting notifications after render", async () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 1, completed: 0, running: 0 },
      tasks: [
        {
          taskId: "task-1",
          sessionId: "session-1",
          pid: 123,
          title: "Approve tool call",
          status: "needs_user",
          source: "hook",
          windowTarget: {
            hostKind: "terminal",
            app: "Ghostty",
            descriptor: "main",
          },
          startedAt: "2026-05-05T15:00:00Z",
          updatedAt: "2026-05-05T15:01:00Z",
          completedAt: null,
        },
      ],
      notifications: [
        {
          id: 1,
          sessionId: "session-1",
          taskId: "task-1",
          status: "needs_user",
          soundType: "waiting",
          occurredAt: "2026-05-05T15:01:00Z",
        },
      ],
    });

    vi.mocked(playSound).mockResolvedValue(undefined);
    vi.mocked(ackNotification).mockResolvedValue(undefined);

    render(<App />);

    await waitFor(() => expect(playSound).toHaveBeenCalledTimes(1));
    expect(playSound).toHaveBeenCalledWith("waiting");
    await waitFor(() => expect(ackNotification).toHaveBeenCalledWith(1));
  });

  it("plays and acknowledges completed notifications after render", async () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 1, running: 0 },
      tasks: [
        {
          taskId: "task-complete",
          sessionId: "session-complete",
          pid: 124,
          title: "Ship feature",
          status: "completed",
          source: "hook",
          windowTarget: {
            hostKind: "terminal",
            app: "Ghostty",
            descriptor: "main",
          },
          startedAt: "2026-05-05T15:00:00Z",
          updatedAt: "2026-05-05T15:02:00Z",
          completedAt: "2026-05-05T15:02:00Z",
        },
      ],
      notifications: [
        {
          id: 2,
          sessionId: "session-complete",
          taskId: "task-complete",
          status: "completed",
          soundType: "completed",
          occurredAt: "2026-05-05T15:02:00Z",
        },
      ],
    });

    vi.mocked(playSound).mockResolvedValue(undefined);
    vi.mocked(ackNotification).mockResolvedValue(undefined);

    render(<App />);

    await waitFor(() => expect(playSound).toHaveBeenCalledTimes(1));
    expect(playSound).toHaveBeenCalledWith("completed");
    await waitFor(() => expect(ackNotification).toHaveBeenCalledWith(2));
  });

  it("plays completed notifications when a later snapshot arrives", async () => {
    vi.mocked(playSound).mockResolvedValue(undefined);
    vi.mocked(ackNotification).mockResolvedValue(undefined);

    const firstSnapshot: Snapshot = {
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    };
    const secondSnapshot: Snapshot = {
      counts: { total: 1, needsUser: 0, completed: 1, running: 0 },
      tasks: [
        {
          ...runningTask,
          taskId: "task-complete-late",
          sessionId: "session-complete-late",
          title: "Late completion",
          status: "completed",
          updatedAt: "2026-05-05T15:03:00Z",
          completedAt: "2026-05-05T15:03:00Z",
        },
      ],
      notifications: [
        {
          id: 3,
          sessionId: "session-complete-late",
          taskId: "task-complete-late",
          status: "completed",
          soundType: "completed",
          occurredAt: "2026-05-05T15:03:00Z",
        },
      ],
    };

    let currentSnapshot = firstSnapshot;
    vi.mocked(useSnapshot).mockImplementation(() => currentSnapshot);

    const { rerender } = render(<App />);

    expect(playSound).not.toHaveBeenCalled();

    currentSnapshot = secondSnapshot;
    rerender(<App />);

    await waitFor(() => expect(playSound).toHaveBeenCalledTimes(1));
    expect(playSound).toHaveBeenCalledWith("completed");
    await waitFor(() => expect(ackNotification).toHaveBeenCalledWith(3));
  });

  it("shows waiting text ahead of a newer running task in the collapsed summary", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 2, needsUser: 1, completed: 0, running: 1 },
      tasks: [
        {
          ...runningTask,
          taskId: "task-waiting",
          sessionId: "session-waiting",
          title: "等待授权",
          status: "needs_user",
          updatedAt: "2026-04-30T00:00:01Z",
        },
        {
          ...runningTask,
          taskId: "task-running",
          sessionId: "session-running",
          title: "较新的运行任务",
          status: "running",
          updatedAt: "2026-04-30T00:00:02Z",
        },
      ],
      notifications: emptyNotifications,
    });

    render(<App />);
    expect(screen.getByText("等待授权 - 待回复")).toBeTruthy();
  });

  it("shows a single collapsed task summary", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    });

    render(<App />);
    expect(screen.getByText("测试任务 - 进行中")).toBeTruthy();
    expect(screen.queryByRole("list")).toBeNull();
  });

  it("resizes the overlay window when expanding the task list", async () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 2, needsUser: 0, completed: 0, running: 2 },
      tasks: [
        runningTask,
        {
          ...runningTask,
          taskId: "task-2",
          sessionId: "session-2",
          title: "第二个任务",
          updatedAt: "2026-04-30T00:00:02Z",
        },
      ],
      notifications: emptyNotifications,
    });

    render(<App />);

    await waitFor(() => expect(setSize).toHaveBeenCalledTimes(1));
    expect(setSize).toHaveBeenLastCalledWith(expect.objectContaining({ width: 260, height: 64 }));

    const summaryButton = screen.getByRole("button", { name: "第二个任务 - 进行中" });
    fireEvent.mouseDown(summaryButton, { clientX: 40, clientY: 40 });
    fireEvent.click(summaryButton);

    await waitFor(() => expect(setSize).toHaveBeenCalledTimes(2));
    expect(setSize).toHaveBeenLastCalledWith(expect.objectContaining({ width: 260, height: 164 }));
    expect(screen.getByRole("list")).toBeTruthy();
  });

  it("keeps completed and idle tasks visible in the expanded overlay list", async () => {
    const mixedSnapshot = {
      counts: { total: 3, needsUser: 0, completed: 1, running: 1 },
      tasks: [
        runningTask,
        {
          ...runningTask,
          taskId: "task-completed",
          sessionId: "session-completed",
          title: "已完成任务",
          status: "completed" as const,
          updatedAt: "2026-04-30T00:00:03Z",
          completedAt: "2026-04-30T00:00:03Z",
        },
        {
          ...runningTask,
          taskId: "task-idle",
          sessionId: "session-idle",
          title: "空闲任务",
          status: "idle_or_unknown" as const,
          updatedAt: "2026-04-30T00:00:04Z",
        },
      ],
      notifications: emptyNotifications,
    } satisfies Snapshot;

    vi.mocked(useSnapshot).mockReturnValue(mixedSnapshot);

    render(<App />);

    const summaryButton = screen.getByRole("button", { name: "测试任务 - 进行中" });
    fireEvent.mouseDown(summaryButton, { clientX: 40, clientY: 40 });
    fireEvent.click(summaryButton);

    const overlayList = await screen.findByRole("list");
    expect(within(overlayList).getByText("测试任务")).toBeTruthy();
    expect(within(overlayList).getByText("已完成任务")).toBeTruthy();
    expect(within(overlayList).getByText("空闲任务")).toBeTruthy();
  });

  it("shows completed history rows alongside active tasks when expanded", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 3, needsUser: 0, completed: 2, running: 1 },
      tasks: [
        runningTask,
        {
          ...runningTask,
          taskId: "task-snake",
          sessionId: "session-snake",
          title: "帮我设计一个贪吃蛇",
          status: "completed",
          updatedAt: "2026-04-30T00:00:00Z",
          completedAt: "2026-04-30T00:00:00Z",
        },
        {
          ...runningTask,
          taskId: "task-hooks",
          sessionId: "session-hooks",
          title: "24 个 hook 事件，哪些可以认为是claudecode在运行中",
          status: "completed",
          updatedAt: "2026-04-30T00:00:00Z",
          completedAt: "2026-04-30T00:00:00Z",
        },
      ],
      notifications: emptyNotifications,
    });

    render(<App />);
    const summaryButton = screen.getByRole("button", { name: "测试任务 - 进行中" });
    fireEvent.mouseDown(summaryButton, { clientX: 40, clientY: 40 });
    fireEvent.click(summaryButton);

    expect(screen.getByRole("list")).toBeTruthy();
    expect(screen.getByText("帮我设计一个贪吃蛇")).toBeTruthy();
    expect(screen.getByText("24 个 hook 事件，哪些可以认为是claudecode在运行中")).toBeTruthy();
  });

  it("starts native dragging after pointer movement passes the threshold", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    });

    render(<App />);
    const button = screen.getByRole("button", { name: "测试任务 - 进行中" });
    fireEvent.mouseDown(button, { clientX: 10, clientY: 10 });
    fireEvent.mouseMove(button, { clientX: 26, clientY: 10 });

    expect(startDragging).toHaveBeenCalledTimes(1);
  });

  it("suppresses the trailing click from a drag gesture but allows the next fresh click", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    });

    render(<App />);
    const summaryButton = screen.getByRole("button", { name: "测试任务 - 进行中" });
    fireEvent.mouseDown(summaryButton, { clientX: 10, clientY: 10 });
    fireEvent.mouseMove(summaryButton, { clientX: 26, clientY: 10 });
    fireEvent.mouseLeave(summaryButton);
    fireEvent.click(summaryButton);

    expect(screen.queryByRole("list")).toBeNull();

    fireEvent.mouseDown(summaryButton, { clientX: 40, clientY: 40 });
    fireEvent.click(summaryButton);

    expect(screen.getByRole("list")).toBeTruthy();
  });

  it("prevents the WebView default context menu", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
      notifications: emptyNotifications,
    });

    render(<App />);
    const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });

    const allowed = document.dispatchEvent(event);

    expect(allowed).toBe(false);
    expect(event.defaultPrevented).toBe(true);
  });
});

import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { focusTask } from "./lib/api";
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
}));

vi.mock("./lib/use-snapshot", () => ({
  useSnapshot: vi.fn(),
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

describe("App", () => {
  it("shows idle text when no tasks exist", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 0, needsUser: 0, completed: 0, running: 0 },
      tasks: [],
    });

    render(<App />);
    expect(screen.getByText("当前无任务")).toBeTruthy();
    expect(screen.queryByText("Test Waiting")).toBeNull();
    expect(screen.queryByText("Test Completed")).toBeNull();
    expect(screen.queryByText("Show Logs")).toBeNull();
  });

  it("shows a single collapsed task summary", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
    });

    render(<App />);
    expect(screen.getByText("测试任务 - 进行中")).toBeTruthy();
    expect(screen.queryByRole("list")).toBeNull();
  });

  it("expands downward to show every task row", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
    });

    render(<App />);
    const summaryButton = screen.getByRole("button", { name: "测试任务 - 进行中" });
    fireEvent.mouseDown(summaryButton, { clientX: 40, clientY: 40 });
    fireEvent.click(summaryButton);

    const list = screen.getByRole("list");
    expect(list).toBeTruthy();
    expect(within(list).getByRole("button", { name: /测试任务/ })).toBeTruthy();
  });

  it("starts native dragging after pointer movement passes the threshold", () => {
    vi.mocked(useSnapshot).mockReturnValue({
      counts: { total: 1, needsUser: 0, completed: 0, running: 1 },
      tasks: [runningTask],
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
    });

    render(<App />);
    const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });

    const allowed = document.dispatchEvent(event);

    expect(allowed).toBe(false);
    expect(event.defaultPrevented).toBe(true);
  });
});


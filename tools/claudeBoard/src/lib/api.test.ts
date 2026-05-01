import { afterEach, describe, expect, it, vi } from "vitest";
import { BASE_URL, fetchSnapshot, focusTask, savePreferencesRemote } from "./api";

describe("api normalization", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("normalizes snake_case window target fields from the snapshot response", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        counts: {
          total: 1,
          needs_user: 0,
          completed: 0,
          running: 1,
        },
        tasks: [
          {
            task_id: "task-1",
            session_id: "session-1",
            pid: 123,
            title: "Task 1",
            status: "running",
            source: "hook",
            window_target: {
              host_kind: "tmux",
              app: "Ghostty",
              descriptor: "session:1",
              tab_id: "tab-7",
              pane_id: "pane-9",
            },
            started_at: "2026-04-29T00:00:00Z",
            updated_at: "2026-04-29T00:01:00Z",
            completed_at: null,
          },
        ],
      }),
    });

    vi.stubGlobal("fetch", fetchMock);

    const snapshot = await fetchSnapshot();

    expect(fetchMock).toHaveBeenCalledWith(`${BASE_URL}/snapshot`);
    expect(snapshot).toEqual({
      counts: {
        total: 1,
        needsUser: 0,
        completed: 0,
        running: 1,
      },
      tasks: [
        {
          taskId: "task-1",
          sessionId: "session-1",
          pid: 123,
          title: "Task 1",
          status: "running",
          source: "hook",
          windowTarget: {
            hostKind: "tmux",
            app: "Ghostty",
            descriptor: "session:1",
            tabId: "tab-7",
            paneId: "pane-9",
          },
          startedAt: "2026-04-29T00:00:00Z",
          updatedAt: "2026-04-29T00:01:00Z",
          completedAt: null,
        },
      ],
    });
  });
});

describe("api requests", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("posts focusTask requests to the daemon", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true });
    vi.stubGlobal("fetch", fetchMock);

    await focusTask("task/with spaces");

    expect(fetchMock).toHaveBeenCalledWith(
      `${BASE_URL}/tasks/${encodeURIComponent("task/with spaces")}/focus`,
      { method: "POST" },
    );
  });

  it("posts preferences to the daemon", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true });
    vi.stubGlobal("fetch", fetchMock);

    const preferences = {
      notifyCompleted: true,
      notifyNeedsUser: false,
      speakCompleted: false,
      speakNeedsUser: true,
    };

    await savePreferencesRemote(preferences);

    expect(fetchMock).toHaveBeenCalledWith(`${BASE_URL}/preferences`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(preferences),
    });
  });
});

import { afterEach, describe, expect, it, vi } from "vitest";
import { BASE_URL, fetchDebugSnapshot, fetchSnapshot, focusTask, savePreferencesRemote } from "./api";

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
      notifications: [],
    });
  });

  it("normalizes nested debug snapshot payloads", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        snapshot: {
          counts: {
            total: 1,
            needs_user: 1,
            completed: 0,
            running: 0,
          },
          tasks: [
            {
              task_id: "task-1",
              session_id: "session-1",
              pid: 321,
              title: "Approve debug command",
              status: "needs_user",
              source: "hook",
              window_target: {
                host_kind: "terminal",
                app: "Ghostty",
                descriptor: "main",
              },
              started_at: "2026-05-06T15:00:00Z",
              updated_at: "2026-05-06T15:01:00Z",
              completed_at: null,
            },
          ],
          notifications: [],
        },
        recent_hook_events: [
          {
            occurred_at: "2026-05-06T15:01:00Z",
            hook_event_name: "PermissionRequest",
            session_id: "session-1",
            pid: 321,
            title: "Approve debug command",
            permission_mode: "default",
            prompt_preview: "show me debug info",
            agent_id: null,
            disposition: "accepted",
            mapped_event_type: "permission_request",
            filter_reason: null,
            previous_status: "running",
            next_status: "needs_user",
          },
        ],
        latest_scan: {
          occurred_at: "2026-05-06T15:00:00Z",
          entries: [
            {
              pid: 320,
              ppid: 0,
              state: "S",
              command: "/usr/local/bin/claude",
              decision: "accepted",
              reason: null,
              accepted_row: "local-320\t320\tworkspace\tTerminal\tterminal\t\t\tclaude",
              task: {
                task_id: "scan:local-320:320",
                session_id: "local-320",
                pid: 320,
                title: "claude",
                status: "idle_or_unknown",
                source: "scan_recovered",
                window_target: {
                  host_kind: "terminal",
                  app: "Terminal",
                  descriptor: "terminal",
                },
                started_at: "2026-05-06T15:00:00Z",
                updated_at: "2026-05-06T15:00:00Z",
                completed_at: null,
              },
            },
          ],
        },
      }),
    });

    vi.stubGlobal("fetch", fetchMock);

    const snapshot = await fetchDebugSnapshot();

    expect(fetchMock).toHaveBeenCalledWith(`${BASE_URL}/debug/snapshot`);
    expect(snapshot.snapshot.counts.needsUser).toBe(1);
    expect(snapshot.snapshot.tasks[0].windowTarget.hostKind).toBe("terminal");
    expect(snapshot.recentHookEvents[0]).toEqual({
      occurredAt: "2026-05-06T15:01:00Z",
      hookEventName: "PermissionRequest",
      sessionId: "session-1",
      pid: 321,
      title: "Approve debug command",
      permissionMode: "default",
      promptPreview: "show me debug info",
      agentId: undefined,
      disposition: "accepted",
      mappedEventType: "permission_request",
      filterReason: undefined,
      previousStatus: "running",
      nextStatus: "needs_user",
    });
    expect(snapshot.latestScan.entries[0].task?.taskId).toBe("scan:local-320:320");
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

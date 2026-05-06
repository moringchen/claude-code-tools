# claudeBoard Atomic Status and Notification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make claudeBoard commit task-list status changes and sound notifications together so the frontend always reads matching task state and pending sound events from the same snapshot.

**Architecture:** Extend the Rust snapshot/store model with a persisted notification queue and ack API, then switch the React polling client to consume notification events from `/snapshot` instead of relying on backend immediate playback or count-diff inference. Keep the atomic boundary inside `TaskStore` while the mutex is held, and persist notifications through the existing snapshot save/load path.

**Tech Stack:** Rust, Axum, Serde, Tauri, React, TypeScript, Vitest.

---

### Task 1: Add notification events to the shared snapshot model

**Files:**
- Modify: `tools/claudeBoard/src-tauri/src/model.rs`
- Modify: `tools/claudeBoard/src/lib/api.ts`
- Modify: `tools/claudeBoard/src/lib/use-snapshot.ts`
- Test: `tools/claudeBoard/src/App.test.tsx`

- [ ] **Step 1: Write the failing frontend normalization test**

Add a test case to `tools/claudeBoard/src/App.test.tsx` that supplies `useSnapshot()` with a snapshot containing a `notifications` array and verifies the app can render without dropping the snapshot shape.

```tsx
it("accepts snapshots that include pending notifications", () => {
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

  render(<App />);

  expect(screen.getByText("Approve tool call - 待处理")).toBeTruthy();
});
```

- [ ] **Step 2: Run the frontend test to verify it fails**

Run: `npm test -- src/App.test.tsx`
Expected: FAIL because the `Snapshot` type and normalization path do not yet include `notifications`.

- [ ] **Step 3: Add the shared snapshot fields**

Update `tools/claudeBoard/src-tauri/src/model.rs` to add the persisted notification type and snapshot field.

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSoundType {
    Waiting,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationEvent {
    pub id: u64,
    pub session_id: String,
    pub task_id: String,
    pub status: TaskStatus,
    pub sound_type: NotificationSoundType,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TaskSnapshot {
    pub counts: SnapshotCounts,
    pub tasks: Vec<TaskCard>,
    #[serde(default)]
    pub sessions: Vec<SessionRecord>,
    #[serde(default)]
    pub notifications: Vec<NotificationEvent>,
}
```

- [ ] **Step 4: Add frontend DTO normalization for notifications**

Update `tools/claudeBoard/src/lib/api.ts` to parse the new field.

```ts
type NotificationEventDto = {
  id: number;
  session_id: string;
  task_id: string;
  status: TaskCard["status"];
  sound_type: "waiting" | "completed";
  occurred_at: string;
};

type SnapshotDto = {
  counts: TaskCountsDto;
  tasks: TaskCardDto[];
  notifications: NotificationEventDto[];
};

export type NotificationEvent = {
  id: number;
  sessionId: string;
  taskId: string;
  status: TaskCard["status"];
  soundType: "waiting" | "completed";
  occurredAt: string;
};

export type Snapshot = {
  counts: TaskCounts;
  tasks: TaskCard[];
  notifications: NotificationEvent[];
};

function normalizeNotification(event: NotificationEventDto): NotificationEvent {
  return {
    id: event.id,
    sessionId: event.session_id,
    taskId: event.task_id,
    status: event.status,
    soundType: event.sound_type,
    occurredAt: event.occurred_at,
  };
}

function normalizeSnapshot(snapshot: SnapshotDto): Snapshot {
  return {
    counts: normalizeCounts(snapshot.counts),
    tasks: snapshot.tasks.map(normalizeTask),
    notifications: snapshot.notifications.map(normalizeNotification),
  };
}
```

Also update `tools/claudeBoard/src/lib/use-snapshot.ts` so `EMPTY_SNAPSHOT` includes `notifications: []`.

```ts
const EMPTY_SNAPSHOT: Snapshot = {
  counts: {
    total: 0,
    needsUser: 0,
    completed: 0,
    running: 0,
  },
  tasks: [],
  notifications: [],
};
```

- [ ] **Step 5: Run the frontend test to verify it passes**

Run: `npm test -- src/App.test.tsx`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/claudeBoard/src-tauri/src/model.rs tools/claudeBoard/src/lib/api.ts tools/claudeBoard/src/lib/use-snapshot.ts tools/claudeBoard/src/App.test.tsx
git commit -m "feat: add snapshot notification model"
```

### Task 2: Make TaskStore enqueue notifications atomically with status changes

**Files:**
- Modify: `tools/claudeBoard/src-tauri/src/store.rs`
- Modify: `tools/claudeBoard/src-tauri/src/model.rs`
- Test: `tools/claudeBoard/src-tauri/tests/store_flow.rs`

- [ ] **Step 1: Write the failing store test for waiting notifications**

Add a store test in `tools/claudeBoard/src-tauri/tests/store_flow.rs` that drives `TaskCreated` then `PermissionRequest`, then asserts the same snapshot contains both `NeedsUser` status and one pending `waiting` notification.

```rust
#[test]
fn permission_request_enqueues_waiting_notification_in_same_snapshot() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCreated,
        session_id: "session-atomic".into(),
        agent_id: None,
        pid: 111,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:10:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-atomic".into(),
        agent_id: None,
        pid: 111,
        title: "Approve command".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:11:00Z".into(),
    });

    let snapshot = store.snapshot();
    let task = snapshot.tasks.iter().find(|task| task.task_id == "session-atomic").unwrap();

    assert_eq!(task.status, TaskStatus::NeedsUser);
    assert_eq!(snapshot.notifications.len(), 1);
    assert_eq!(snapshot.notifications[0].sound_type, claude_board::model::NotificationSoundType::Waiting);
}
```

- [ ] **Step 2: Run the store test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml permission_request_enqueues_waiting_notification_in_same_snapshot -- --exact`
Expected: FAIL because `TaskSnapshot` and `TaskStore` do not yet provide notification queue state.

- [ ] **Step 3: Add store-owned pending notification state**

Update `tools/claudeBoard/src-tauri/src/store.rs` to persist queued notification events and next id.

```rust
#[derive(Default)]
pub struct TaskStore {
    hook_tasks: HashMap<String, TaskCard>,
    scanned_tasks: HashMap<String, TaskCard>,
    session_records: HashMap<String, SessionRecord>,
    pending_notifications: Vec<NotificationEvent>,
    next_notification_id: u64,
}
```

Initialize `next_notification_id` to `1` when it is `0` before enqueueing the first event.

- [ ] **Step 4: Enqueue notifications from real status transitions only**

Inside `TaskStore::apply`, compare the previous and next task statuses and enqueue only on transitions into `NeedsUser` or `Completed`.

```rust
fn enqueue_notification(
    &mut self,
    task_id: &str,
    session_id: &str,
    status: TaskStatus,
    sound_type: NotificationSoundType,
    occurred_at: &str,
) {
    if self.next_notification_id == 0 {
        self.next_notification_id = 1;
    }

    self.pending_notifications.push(NotificationEvent {
        id: self.next_notification_id,
        session_id: session_id.to_string(),
        task_id: task_id.to_string(),
        status,
        sound_type,
        occurred_at: occurred_at.to_string(),
    });
    self.next_notification_id += 1;
}
```

Use it after the task status is updated.

```rust
if task.status != previous_status {
    match task.status {
        TaskStatus::NeedsUser => self.enqueue_notification(
            &task.task_id,
            &task.session_id,
            TaskStatus::NeedsUser,
            NotificationSoundType::Waiting,
            &task.updated_at,
        ),
        TaskStatus::Completed => self.enqueue_notification(
            &task.task_id,
            &task.session_id,
            TaskStatus::Completed,
            NotificationSoundType::Completed,
            task.completed_at.as_deref().unwrap_or(&task.updated_at),
        ),
        _ => {}
    }
}
```

- [ ] **Step 5: Include notifications in snapshots and persisted snapshots**

Extend both `snapshot()` and `persisted_snapshot()` so they return a copy of the pending queue.

```rust
let notifications = self.pending_notifications.clone();

TaskSnapshot {
    counts,
    tasks,
    sessions,
    notifications,
}
```

Also update `restore_snapshot()` so it restores `pending_notifications` and advances `next_notification_id`.

```rust
self.pending_notifications = snapshot.notifications;
self.next_notification_id = self
    .pending_notifications
    .iter()
    .map(|event| event.id)
    .max()
    .map(|id| id + 1)
    .unwrap_or(1);
```

- [ ] **Step 6: Add the completed-path and duplicate-prevention tests**

Add tests in `tools/claudeBoard/src-tauri/tests/store_flow.rs` for:

```rust
#[test]
fn task_completed_enqueues_completed_notification_once() {
    let mut store = TaskStore::default();

    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-complete".into(),
        agent_id: None,
        pid: 222,
        title: "Done".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:20:00Z".into(),
    });
    store.apply(HookEvent {
        event_type: HookEventType::TaskCompleted,
        session_id: "session-complete".into(),
        agent_id: None,
        pid: 222,
        title: "Done".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T15:20:01Z".into(),
    });

    let snapshot = store.snapshot();
    assert_eq!(snapshot.notifications.len(), 1);
    assert_eq!(snapshot.notifications[0].sound_type, claude_board::model::NotificationSoundType::Completed);
}
```

- [ ] **Step 7: Run the store tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml store_flow -- --nocapture`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add tools/claudeBoard/src-tauri/src/store.rs tools/claudeBoard/src-tauri/src/model.rs tools/claudeBoard/src-tauri/tests/store_flow.rs
git commit -m "feat: queue notifications in task store"
```

### Task 3: Add notification acknowledgement to the backend API

**Files:**
- Modify: `tools/claudeBoard/src-tauri/src/store.rs`
- Modify: `tools/claudeBoard/src-tauri/src/server.rs`
- Test: `tools/claudeBoard/src-tauri/tests/http_api.rs`

- [ ] **Step 1: Write the failing HTTP ack test**

Add an API test in `tools/claudeBoard/src-tauri/tests/http_api.rs` that posts a hook event, fetches `/snapshot`, acks the returned notification id, then fetches `/snapshot` again and expects the task to remain while notifications become empty.

```rust
#[tokio::test]
async fn ack_removes_notification_without_changing_task_status() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-05T16:00:00Z".to_string());

    let event = serde_json::json!({
        "hook_event_name": "PermissionRequest",
        "session_id": "session-ack",
        "claude_board_pid": 3001,
        "claude_board_title": "Approve tool call",
        "claude_board_occurred_at": "2026-05-05T16:00:00Z",
        "cwd": "/workspace"
    });

    let response = app.clone().oneshot(
        Request::post("/events")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&event).unwrap()))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let snapshot = app.clone().oneshot(Request::get("/snapshot").body(Body::empty()).unwrap()).await.unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX).await.unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();
    let notification_id = snapshot.notifications[0].id;

    let ack = app.clone().oneshot(
        Request::post(format!("/notifications/{notification_id}/ack"))
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(ack.status(), StatusCode::ACCEPTED);

    let snapshot = app.oneshot(Request::get("/snapshot").body(Body::empty()).unwrap()).await.unwrap();
    let body = axum::body::to_bytes(snapshot.into_body(), usize::MAX).await.unwrap();
    let snapshot: TaskSnapshot = serde_json::from_slice(&body).unwrap();

    assert_eq!(snapshot.notifications.len(), 0);
    assert_eq!(snapshot.tasks[0].status, TaskStatus::NeedsUser);
}
```

- [ ] **Step 2: Run the HTTP test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ack_removes_notification_without_changing_task_status -- --exact`
Expected: FAIL because the ack route and store removal method do not exist.

- [ ] **Step 3: Add a store method for idempotent ack**

Add a method to `tools/claudeBoard/src-tauri/src/store.rs`.

```rust
pub fn acknowledge_notification(&mut self, id: u64) -> bool {
    let previous_len = self.pending_notifications.len();
    self.pending_notifications.retain(|event| event.id != id);
    previous_len != self.pending_notifications.len()
}
```

- [ ] **Step 4: Add the ack route and persistence**

Update `tools/claudeBoard/src-tauri/src/server.rs` to expose the route.

```rust
async fn post_ack_notification(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut store = state.store.lock().unwrap();
    store.acknowledge_notification(id);
    if let Some(path) = &state.state_path {
        if let Err(error) = save_snapshot(path, &store.persisted_snapshot()) {
            eprintln!("[claudeBoard] failed to save session state after notification ack: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    StatusCode::ACCEPTED
}
```

Register it:

```rust
.route("/notifications/:id/ack", post(post_ack_notification))
```

- [ ] **Step 5: Make repeated ack harmless**

Add a second API test asserting the same ack endpoint can be called twice and still returns `StatusCode::ACCEPTED`.

```rust
#[tokio::test]
async fn ack_is_idempotent_for_missing_notification_ids() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store, || Ok(Vec::new()), || "2026-05-05T16:05:00Z".to_string());

    let response = app.oneshot(
        Request::post("/notifications/999/ack")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
```

- [ ] **Step 6: Run the HTTP tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml http_api -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add tools/claudeBoard/src-tauri/src/store.rs tools/claudeBoard/src-tauri/src/server.rs tools/claudeBoard/src-tauri/tests/http_api.rs
git commit -m "feat: add notification ack api"
```

### Task 4: Move frontend sound playback to notification-event consumption

**Files:**
- Modify: `tools/claudeBoard/src/App.tsx`
- Modify: `tools/claudeBoard/src/lib/api.ts`
- Modify: `tools/claudeBoard/src/App.test.tsx`
- Optionally modify: `tools/claudeBoard/src/lib/use-snapshot.ts`

- [ ] **Step 1: Write the failing frontend playback test**

Add a test to `tools/claudeBoard/src/App.test.tsx` that renders a snapshot with one pending notification, verifies the matching sound function is called, and verifies the ack request helper is called with that id.

```tsx
import * as sound from "./lib/sound";
import { acknowledgeNotification, focusTask } from "./lib/api";

vi.mock("./lib/sound", () => ({
  playSound: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("./lib/api", () => ({
  focusTask: vi.fn(),
  acknowledgeNotification: vi.fn().mockResolvedValue(undefined),
}));

it("plays and acknowledges pending waiting notifications", async () => {
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
        startedAt: "2026-05-05T16:10:00Z",
        updatedAt: "2026-05-05T16:11:00Z",
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
        occurredAt: "2026-05-05T16:11:00Z",
      },
    ],
  });

  render(<App />);

  await waitFor(() => {
    expect(sound.playSound).toHaveBeenCalledWith("waiting");
    expect(acknowledgeNotification).toHaveBeenCalledWith(1);
  });
});
```

- [ ] **Step 2: Run the frontend test to verify it fails**

Run: `npm test -- src/App.test.tsx`
Expected: FAIL because the app does not yet consume `snapshot.notifications`.

- [ ] **Step 3: Add the frontend ack API helper**

Extend `tools/claudeBoard/src/lib/api.ts`.

```ts
export async function acknowledgeNotification(id: number): Promise<void> {
  const response = await fetch(`${BASE_URL}/notifications/${id}/ack`, {
    method: "POST",
  });

  if (!response.ok) {
    throw new Error(`Failed to acknowledge notification: ${response.status}`);
  }
}
```

- [ ] **Step 4: Consume pending notifications in App**

Add a React effect in `tools/claudeBoard/src/App.tsx` that tracks ids in flight and processes snapshot notifications in id order.

```tsx
import { useEffect, useMemo, useRef, useState } from "react";
import { acknowledgeNotification, focusTask } from "./lib/api";
import { playSound } from "./lib/sound";

const inFlightNotificationIds = useRef<Set<number>>(new Set());

useEffect(() => {
  const pending = [...snapshot.notifications]
    .filter((event) => !inFlightNotificationIds.current.has(event.id))
    .sort((left, right) => left.id - right.id);

  for (const event of pending) {
    inFlightNotificationIds.current.add(event.id);
    void (async () => {
      try {
        await playSound(event.soundType);
        await acknowledgeNotification(event.id);
      } finally {
        inFlightNotificationIds.current.delete(event.id);
      }
    })();
  }
}, [snapshot.notifications]);
```

- [ ] **Step 5: Prevent duplicate replay during rerender**

Add a second test in `tools/claudeBoard/src/App.test.tsx` that rerenders the same snapshot while the ack promise is unresolved and asserts `playSound` is still called only once.

```tsx
it("does not replay the same notification while ack is in flight", async () => {
  let resolveAck: (() => void) | undefined;
  vi.mocked(acknowledgeNotification).mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        resolveAck = resolve;
      }),
  );

  const snapshot = {
    counts: { total: 1, needsUser: 1, completed: 0, running: 0 },
    tasks: [runningTask],
    notifications: [
      {
        id: 8,
        sessionId: "session-1",
        taskId: "task-1",
        status: "needs_user" as const,
        soundType: "waiting" as const,
        occurredAt: "2026-05-05T16:15:00Z",
      },
    ],
  };

  vi.mocked(useSnapshot).mockReturnValue(snapshot);
  const rendered = render(<App />);

  await waitFor(() => expect(playSound).toHaveBeenCalledTimes(1));
  rendered.rerender(<App />);
  expect(playSound).toHaveBeenCalledTimes(1);
  resolveAck?.();
});
```

- [ ] **Step 6: Run the frontend tests to verify they pass**

Run: `npm test -- src/App.test.tsx`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add tools/claudeBoard/src/App.tsx tools/claudeBoard/src/lib/api.ts tools/claudeBoard/src/App.test.tsx
git commit -m "feat: consume queued notification sounds"
```

### Task 5: Remove backend direct playback for task-state sounds and verify recovery

**Files:**
- Modify: `tools/claudeBoard/src-tauri/src/server.rs`
- Modify: `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs`
- Modify: `tools/claudeBoard/src-tauri/tests/http_api.rs`
- Modify: `tools/claudeBoard/src-tauri/tests/store_flow.rs`
- Optionally modify: `tools/claudeBoard/src-tauri/src/session_state.rs`

- [ ] **Step 1: Write the failing recovery test**

Add a test in `tools/claudeBoard/src-tauri/tests/store_flow.rs` or `http_api.rs` that persists a snapshot with one pending notification, restores it into a fresh store, and asserts the notification remains pending.

```rust
#[test]
fn restore_snapshot_keeps_pending_notifications() {
    let mut store = TaskStore::default();
    store.apply(HookEvent {
        event_type: HookEventType::PermissionRequest,
        session_id: "session-restart".into(),
        agent_id: None,
        pid: 808,
        title: "Approve after restart".into(),
        conversation_content: None,
        occurred_at: "2026-05-05T16:20:00Z".into(),
    });

    let snapshot = store.persisted_snapshot();
    let mut restored = TaskStore::default();
    restored.restore_snapshot(snapshot);

    assert_eq!(restored.snapshot().notifications.len(), 1);
    assert_eq!(restored.snapshot().notifications[0].sound_type, claude_board::model::NotificationSoundType::Waiting);
}
```

- [ ] **Step 2: Run the recovery test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml restore_snapshot_keeps_pending_notifications -- --exact`
Expected: FAIL until notifications are restored and persisted correctly.

- [ ] **Step 3: Remove backend direct playback from event ingestion**

Delete the task-status sound side effects from `tools/claudeBoard/src-tauri/src/server.rs`.

```rust
if matches!(changed_status, Some(crate::model::TaskStatus::NeedsUser)) {
    let _ = play_sound_file("waiting".to_string());
}
if matches!(changed_status, Some(crate::model::TaskStatus::Completed)) {
    let _ = play_sound_file("completed".to_string());
}
```

Also remove the unused `play_sound_file` import from the same file once no task-state path uses it.

- [ ] **Step 4: Ensure daemon persistence includes notifications without special handling**

Confirm `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs` continues to save `store.persisted_snapshot()` after replay/refresh and does not need any sound-specific side effect.

```rust
if let Err(error) = claude_board::session_state::save_snapshot(
    &state_path,
    &store.lock().unwrap().persisted_snapshot(),
) {
    eprintln!("[claudeBoard] failed to save session state after replay: {error}");
}
```

No new code is required if the existing persisted snapshot path already serializes notifications; only update the file if compile errors or helper extraction are needed.

- [ ] **Step 5: Run the recovery and HTTP regression tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml http_api store_flow -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/claudeBoard/src-tauri/src/server.rs tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs tools/claudeBoard/src-tauri/tests/http_api.rs tools/claudeBoard/src-tauri/tests/store_flow.rs tools/claudeBoard/src-tauri/src/session_state.rs
git commit -m "refactor: remove direct task status sound playback"
```

### Task 6: Final verification

**Files:**
- Verify only: `tools/claudeBoard/src/App.tsx`
- Verify only: `tools/claudeBoard/src/lib/api.ts`
- Verify only: `tools/claudeBoard/src-tauri/src/model.rs`
- Verify only: `tools/claudeBoard/src-tauri/src/store.rs`
- Verify only: `tools/claudeBoard/src-tauri/src/server.rs`
- Verify only: `tools/claudeBoard/src-tauri/src/session_state.rs`
- Verify only: `tools/claudeBoard/src-tauri/tests/http_api.rs`
- Verify only: `tools/claudeBoard/src-tauri/tests/store_flow.rs`

- [ ] **Step 1: Run the frontend suite**

Run: `npm test`
Expected: PASS.

- [ ] **Step 2: Run the Rust suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

- [ ] **Step 3: Run the Tauri app for manual verification**

Run: `npm run tauri:dev`
Expected: the desktop overlay launches successfully.

- [ ] **Step 4: Verify the atomic behavior manually**

In the running app, trigger one `PermissionRequest` or `TaskCompleted` hook event and verify:
- the task list shows the new status;
- exactly one matching sound plays;
- refreshing or waiting for the next poll does not replay the same sound after ack.

- [ ] **Step 5: Commit**

```bash
git add tools/claudeBoard/src/App.tsx tools/claudeBoard/src/lib/api.ts tools/claudeBoard/src-tauri/src/model.rs tools/claudeBoard/src-tauri/src/store.rs tools/claudeBoard/src-tauri/src/server.rs tools/claudeBoard/src-tauri/src/session_state.rs tools/claudeBoard/src-tauri/tests/http_api.rs tools/claudeBoard/src-tauri/tests/store_flow.rs tools/claudeBoard/src/App.test.tsx tools/claudeBoard/src/lib/use-snapshot.ts
git commit -m "feat: make task status and sound notifications atomic"
```

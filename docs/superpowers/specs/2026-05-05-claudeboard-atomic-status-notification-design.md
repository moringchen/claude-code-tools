# claudeBoard Atomic Status and Notification Design

**Date:** 2026-05-05  
**Status:** Approved for planning

## Goal

Make claudeBoard treat task-list status changes and sound notifications as one atomic state transition so the UI never observes a new `NeedsUser` or `Completed` status without also observing the matching notification event from the same commit.

## Scope

### In scope
- Move notification creation into the backend store commit path.
- Represent notifications as persisted queue items attached to the snapshot model.
- Return pending notifications together with tasks/counts/sessions from `/snapshot`.
- Add an acknowledgement API so the frontend can mark notification events as consumed after playback.
- Make frontend playback consume backend-generated notification events instead of inferring sounds from count diffs or backend side effects.
- Persist pending notification events through snapshot save/load and daemon restart.
- Add regression tests for atomic store commits, snapshot exposure, ack behavior, and restart recovery.

### Out of scope
- Changing which task states are considered sound-worthy beyond the current `NeedsUser` and `Completed` transitions.
- Adding new sound types, debounce rules, batching rules, or notification preferences.
- Reworking polling into push transport such as WebSocket or SSE.
- Solving unrelated stale-running semantics beyond keeping status and sound events consistent.

## Problem statement

Today the state update and the sound side effect are split across layers. `TaskStore::apply` mutates task/session state, `server.rs` saves a snapshot, unlocks the store, and then immediately calls `play_sound_file(...)` for `NeedsUser` and `Completed`. The frontend separately polls `/snapshot` and renders the list from the returned task state.

That split allows mismatch windows:
- the task state can be committed and later observed by the frontend even if the sound side effect fails or is skipped;
- the sound side effect can happen before the frontend has polled the matching snapshot;
- restart recovery can restore task state without a durable record of whether the matching sound was already emitted or still needs playback.

The user requirement is stricter: status visibility and sound playback intent must come from one atomic commit so the frontend sees them as one unit.

## Recommended approach

Introduce a persisted notification-event queue inside `TaskStore`. Whenever a hook-driven status transition changes a task into `NeedsUser` or `Completed`, the store should enqueue a notification event in the same locked mutation that updates the task/session state. `/snapshot` should expose both the task list and the pending notification events. The frontend should play sounds only from snapshot notification events and acknowledge successful playback through a dedicated API.

This keeps the atomic boundary exactly where the authoritative state already lives: inside the store mutation guarded by the backend mutex.

## Alternatives considered

### Option 1: Keep backend immediate playback and persist a "played" flag
Persist whether a sound was emitted, but continue calling `play_sound_file(...)` directly inside the request path.

**Pros:** Smallest backend diff.  
**Cons:** Still couples atomic state to an immediate side effect; frontend can still observe status without a durable playback-intent record from the same commit.

### Option 2: Frontend derives sounds from task/count diffs
Let the frontend compare snapshots and decide when to play sounds.

**Pros:** Minimal backend work.  
**Cons:** No durable event identity, easy to miss or replay sounds across polling gaps, refreshes, and restart recovery.

### Option 3: Store-owned persisted notification events
Store generates notification events in the same mutation as task status changes and frontend consumes them by id.

**Pros:** Satisfies the atomicity requirement directly, survives restarts, avoids count-diff heuristics, and makes duplicate prevention testable.  
**Cons:** Requires model/API changes plus an ack flow.

## Detailed design

### Atomic commit boundary

The atomic unit is a single `TaskStore` mutation while the store mutex is held. A commit may update:
- `hook_tasks`
- `scanned_tasks` when relevant
- `session_records`
- `pending_notifications`
- notification sequence metadata

The store should return a `StoreCommit`-style result that describes what changed during the mutation, but notification creation itself must happen inside the mutation, not afterward in `server.rs`.

### Notification event model

Add a persisted notification record with stable identity. The record should include:
- `id: u64` — monotonically increasing unique id within the local snapshot
- `session_id: String`
- `task_id: String`
- `status: TaskStatus` limited to sound-producing states
- `sound_type: String` or enum with `waiting` / `completed`
- `occurred_at: String`

`TaskSnapshot` should gain a `notifications: Vec<NotificationEvent>` field. The snapshot file saved through `session_state.rs` should include this field so pending notifications survive restart.

### Notification enqueue rules

The store should enqueue exactly one event when a mutation causes a task status transition into one of the sound-producing statuses:
- transition to `NeedsUser` → enqueue `waiting`
- transition to `Completed` → enqueue `completed`

No event should be enqueued when:
- the status remains unchanged;
- a duplicate hook event repeats the same already-visible status;
- a scan refresh updates metadata without changing the user-visible status.

This preserves the current semantic rule that sounds follow real status transitions only.

### Snapshot contract

`GET /snapshot` should return one coherent payload containing:
- `counts`
- `tasks`
- `sessions`
- `notifications`

The frontend should treat one snapshot as one consistency boundary. Any task status visible in that payload is paired with the exact notification queue state that resulted from the same series of committed store mutations.

### Acknowledgement flow

Add a lightweight acknowledgement endpoint such as `POST /notifications/:id/ack`.

Behavior:
- if the notification id exists, remove it from `pending_notifications` and persist the updated snapshot;
- if it does not exist, return success/no-op semantics so duplicate ack retries are harmless.

This makes playback at-least-once from the backend’s perspective: until acked, the event remains pending and can be replayed after refresh or restart.

### Frontend behavior

The frontend should stop inferring sounds from count deltas or relying on backend immediate playback. Instead:
1. poll `/snapshot` as today;
2. render tasks from `snapshot.tasks`;
3. inspect `snapshot.notifications` in id order;
4. for each unprocessed id in the current browser session, play the mapped sound;
5. after successful playback, call the ack endpoint;
6. keep a local in-memory set of ids currently being played/acked to avoid duplicate concurrent playback during overlapping polls.

This preserves consistency: the list and the sound both originate from the same snapshot payload.

### Restart and recovery behavior

`session_state::save_snapshot` already writes the full snapshot atomically through a temp file and rename. Extending the snapshot model to include notifications is sufficient for persistence. On startup, `load_snapshot` should restore pending notification events unchanged. That means a notification created before app or daemon restart remains pending until the frontend successfully acks it.

### Backend sound responsibility

Remove request-path direct playback from `server.rs` for task-state sounds. The backend should no longer call `play_sound_file(...)` as the authoritative delivery path for task-list sounds. Playback becomes a consumer concern driven by pending notification events.

Other unrelated sound entry points can remain if they are not tied to task-list state transitions, but task-status sounds should have a single path.

## Data model changes

Additions:
- new `NotificationEvent` struct in `src-tauri/src/model.rs`
- new `notifications` field on `TaskSnapshot`
- new `pending_notifications` storage in `TaskStore`
- new `next_notification_id` counter in `TaskStore`

Possible supporting type:
- `StoreCommit` struct describing `status_changed`, `enqueued_notification_ids`, or similar metadata for server handlers and tests

## API changes

### Existing endpoint
- `GET /snapshot` returns `notifications` in addition to current fields.

### New endpoint
- `POST /notifications/:id/ack`
  - success when event is removed
  - also success when already absent

No other API shape changes are required for this design.

## Testing strategy

### Store tests
Add tests that prove one store mutation atomically produces both effects:
- `TaskCreated` → `PermissionRequest` changes task to `NeedsUser` and enqueues one `waiting` notification in the same snapshot state.
- `TaskCompleted` enqueues one `completed` notification and marks task completed in the same snapshot state.
- repeated `PermissionRequest` or repeated terminal events do not enqueue duplicates.
- ack removes only the notification event and does not alter task status.

### HTTP API tests
Add tests that:
- post a hook event, then fetch `/snapshot`, and assert the response contains both the new task status and exactly one matching notification event;
- ack a notification id and verify a subsequent snapshot no longer includes it;
- verify repeated ack is harmless.

### Recovery tests
Add tests for snapshot persistence and reload so a saved snapshot with pending notifications restores them after `load_snapshot` / `restore_snapshot`.

### Frontend tests
Add tests that:
- render from a snapshot containing pending notifications and verify the frontend plays the expected sound and calls ack;
- ensure rerendering the same pending id during an in-flight ack does not replay it immediately;
- ensure a snapshot with no notifications does not play anything even when counts change in unrelated ways.

## Success criteria

- Every sound-worthy task status transition creates a durable notification event in the same store commit as the status update.
- `/snapshot` always exposes a task list and notification queue that are mutually consistent.
- The frontend no longer derives task-list sounds from timing heuristics or backend direct side effects.
- Unacked notifications survive snapshot persistence and daemon/app restart.
- Acknowledging a notification removes only the event, not the task state.
- Regression tests cover atomic enqueue, duplicate prevention, ack behavior, and recovery.

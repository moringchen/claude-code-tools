# claudeBoard terminal focus and compact mode design

## Context

claudeBoard currently shows one visible task per main Claude Code session and supports click-to-focus through the existing `/tasks/:id/focus` flow. The current overlay only has two visual states driven by `expanded: boolean`, task titles do not consistently prefer the latest conversation topic, and tasks can outlive the real terminal/window lifecycle longer than desired. The requested feature set is:

1. task names should first use the Claude Code task topic when available
2. clicking a task in the dropdown should jump to the corresponding live terminal, regardless of terminal app when possible
3. tasks should disappear once their backing window, shell, process, or terminal is gone
4. double-clicking the overlay header should enter a compact mode that shows only `待 N / 完 N`, snaps to the nearest screen edge, and restores on single click

## Goals

- Preserve the existing product model of one visible task per main Claude Code session
- Improve visible task titles so they reflect the latest meaningful conversation topic
- Make focus behavior as accurate as possible for any still-live terminal host
- Remove dead tasks promptly instead of leaving stale visible rows
- Add a compact overlay mode without breaking current drag/expand behavior

## Non-goals

- Do not introduce a per-subagent visible task model
- Do not build a heavyweight global terminal registry service
- Do not guess focus targets when the backend cannot identify a single live terminal confidently
- Do not redesign the daemon/snapshot transport layer

## Recommended approach

Use the existing session-level snapshot and focus pipeline, but strengthen three layers around it:

1. **Title resolution**: use the latest session conversation topic as the first visible title source, then fall back to scan/session/window titles.
2. **Terminal targeting**: keep the existing `focusTask(taskId)` API and make backend focus execution attempt host-specific targeting first, then safe pid/session fallback.
3. **Compact overlay state**: replace the current two-state UI with an explicit three-state overlay mode model so compact behavior is isolated from expanded/collapsed list behavior.

This keeps the change local to the current claudeBoard architecture and avoids introducing an unrelated registry or a new product model.

## Existing architecture to reuse

- `src-tauri/src/store.rs` already persists `SessionRecord.last_conversation_content`, which can become the primary visible title source.
- `src-tauri/src/model.rs` already includes `window_target.host_kind`, `app`, `descriptor`, `tab_id`, and `pane_id`, which are the right backbone for terminal focus routing.
- `src-tauri/src/scan.rs` already rebuilds `TaskCard.window_target` for scanned sessions.
- `src/lib/api.ts` already exposes `focusTask(taskId)` and snapshot normalization.
- `src/App.tsx`, `src/components/OverlayBar.tsx`, and `src/components/TaskList.tsx` already own overlay mode, header interaction, and task click behavior.

## Detailed design

### 1. Task title priority

Visible task titles should resolve in this order:

1. the latest non-empty `SessionRecord.last_conversation_content`
2. the current `TaskCard.title`
3. any existing scan/session/window fallback already produced by the backend

Implementation-wise, the backend should keep storing the latest conversation content exactly once in the session record, then apply the title preference when building visible snapshots. That keeps title logic centralized and ensures both hook-derived and scan-recovered tasks converge on the same visible title rule.

This choice matches the requested behavior of preferring the latest conversation topic over generic terminal/window names, while still recovering gracefully when no topic has been captured.

### 2. Task removal when the backing terminal is gone

A visible task should remain only while its backing execution context is still alive.

The removal decision should follow a layered check:

1. **PID/liveness first**: continue using the current process-based liveness signal as the primary fast path.
2. **Window-target confirmation second**: if pid-based checks are ambiguous, use `window_target` host information to confirm whether the terminal window/tab/pane still exists.
3. **Immediate visibility removal on confirmed death**: once the process, shell, pane, tab, or host window is confirmed gone, remove the task from the visible snapshot instead of letting it linger.

This is intentionally stricter than long retention for visible UI. Persistence can still keep any internal state needed for debugging, but user-visible task rows should disappear as soon as the backing terminal context is known dead.

### 3. Click-to-focus routing

The frontend should keep calling `focusTask(taskId)` exactly as it does today. The backend focus handler should become a structured router:

1. Resolve the task from the current snapshot/store.
2. Attempt direct focus using `window_target` identity.
   - tmux-like hosts: focus the specific session/window/pane when identifiers exist
   - GUI terminals: focus the specific application window/tab when host metadata is sufficient
3. If host-specific routing cannot identify a single live target, try a safe fallback using the live pid/session relationship.
4. If the backend still cannot identify a single correct target, return an explicit failure instead of guessing.

The product requirement is broad host support, but the reliability rule is more important than breadth: prefer a precise failure over opening the wrong terminal.

### 4. Overlay interaction model

Replace the current `expanded: boolean` with an explicit overlay mode enum:

- `collapsed`
- `expanded`
- `compact`

Behavior by mode:

- **collapsed**: current normal summary bar behavior
- **expanded**: task list visible, current click-to-expand workflow retained
- **compact**: narrow width, compact summary only, snap-to-edge behavior enabled

This separation prevents compact mode from becoming an awkward overload of the existing expand/collapse flag and keeps interaction rules understandable.

### 5. Compact mode interactions

#### Entry
- Double-click the overlay header to enter compact mode.

#### Appearance
- Narrower width than the normal collapsed bar.
- Content shows only short counters in the requested format: `待 N / 完 N`.
- No task list is shown in compact mode.

#### Interaction
- Single click while compact restores normal collapsed mode.
- Dragging remains allowed.
- On drag end, the overlay snaps to the nearest screen edge.
- Compact mode does not expand directly into the task list on single click, which avoids accidental list expansion.

### 6. Snap-to-edge behavior

When entering compact mode or finishing a drag in compact mode:

1. measure the overlay position relative to the active display bounds
2. compare distance to the left and right edges
3. snap to whichever side is closer
4. preserve a small visual margin from the chosen edge

This implements the requested “auto snap to nearest side” behavior while keeping the rule deterministic and easy to test.

## Data and API impact

### Backend
- No breaking schema redesign is required.
- `TaskCard` can keep its current structure.
- `SessionRecord` already has the data needed for title priority.
- Focus endpoints remain `/tasks/:id/focus`.

### Frontend
- `App.tsx` should move from boolean expansion state to overlay mode state.
- `OverlayBar.tsx` should distinguish single click, drag, and double click.
- `TaskList.tsx` click behavior remains the same API-wise.
- `overlay-window.ts` should expose compact-mode sizing and, if needed, helpers for snap positioning.

## Error handling

- If a task is clicked after it has died but before the next UI refresh, the focus endpoint should return a clear “task not found or no longer alive” error.
- If host-specific focus fails, the backend should attempt only deterministic fallbacks.
- If no deterministic target exists, return failure rather than focusing the wrong terminal.
- Compact-mode snapping should fall back to the current position if screen bounds cannot be resolved, rather than moving unpredictably.

## Testing strategy

### Frontend
- verify overlay mode transitions: collapsed ↔ expanded and compact → collapsed
- verify double click enters compact mode
- verify single click in compact mode restores collapsed mode
- verify drag does not trigger click toggles
- verify compact summary renders `待 N / 完 N`
- verify compact sizing differs from normal collapsed sizing

### Backend
- verify visible title priority prefers latest conversation content
- verify focus routing selects the correct host-specific path when metadata exists
- verify fallback uses pid/session only when deterministic
- verify dead backing contexts are removed from visible snapshot

### Integration
- verify snapshot output title matches latest conversation topic when present
- verify clicking a task still calls the same frontend API contract
- verify focus failures are explicit and do not mutate unrelated task state
- verify compact mode does not interfere with existing notification playback or snapshot polling

## Trade-offs considered

### Recommended: extend the current architecture
Pros:
- minimal product-model churn
- reuses existing task, focus, and overlay structures
- easier to land incrementally

Cons:
- terminal focus support must be implemented host by host
- liveness confirmation may require a few backend-specific probes

### Rejected: build a dedicated terminal registry
Pros:
- potentially stronger long-term routing model

Cons:
- much higher implementation and maintenance cost
- unnecessary for the current scope
- adds architectural weight without proving present need

### Rejected: UI-only compact mode without backend lifecycle/focus improvements
Pros:
- fastest to demo visually

Cons:
- leaves the core terminal-jump and stale-task problems unsolved
- would make the feature feel incomplete

## Open decisions resolved in this design

- Terminal targeting priority: support any live terminal host when possible, not only Ghostty.
- Title priority: prefer latest conversation topic; otherwise fall back to existing scan/window title sources.
- Compact summary format: `待 N / 完 N`.
- Compact snap side: auto-select the nearest edge.

## Implementation slices

1. refactor overlay state to `collapsed | expanded | compact`
2. add compact header interactions and compact sizing
3. apply visible title priority from session record conversation content
4. harden liveness removal for dead terminal contexts
5. expand backend focus routing using `window_target`
6. add focused frontend/backend/integration regressions for the new behavior

## Acceptance criteria

- Visible task titles use the latest conversation topic when available.
- Clicking a task attempts to focus the correct live terminal and never intentionally jumps to an ambiguous target.
- Tasks disappear after their backing terminal context is confirmed gone.
- Double-clicking the header enters compact mode.
- Compact mode shows `待 N / 完 N`, snaps to the nearest side, and restores to normal collapsed mode on single click.
- Existing expanded task list behavior continues to work outside compact mode.

# claudeBoard Sound and Z-Order Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make claudeBoard play completion/waiting sounds directly on hook-driven status changes and drop behind other apps when unfocused.

**Architecture:** Keep snapshot-driven status detection in React, but move sound playback to a Tauri command so playback does not depend on browser user interaction. Extract macOS window level constants into testable library code and use a normal-window level when the overlay loses focus.

**Tech Stack:** React, TypeScript, Vitest, Tauri 2, Rust, macOS Cocoa window levels.

---

### Task 1: Frontend sound trigger command

**Files:**
- Modify: `src/App.test.tsx`
- Modify: `src/lib/sound.ts`
- Modify: `src-tauri/src/sound.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write the failing test**

Add a Vitest test in `src/App.test.tsx` that mocks `./lib/sound`, renders with running counts, rerenders with `needsUser` or `completed` incremented, and expects `playSound("waiting")` / `playSound("completed")` to be called without any mouse interaction.

- [ ] **Step 2: Run frontend test to verify it fails**

Run: `npm test -- src/App.test.tsx`
Expected: FAIL because current test mock is not wired or current sound implementation still exposes interaction-dependent behavior.

- [ ] **Step 3: Implement minimal frontend change**

Change `src/lib/sound.ts` so `playSound(type)` invokes a backend command such as `play_sound_file` with `waiting` or `completed`; remove user-interaction gating from production playback.

- [ ] **Step 4: Implement backend command**

Add `play_sound_file(sound_type: String)` in `src-tauri/src/sound.rs` that maps `waiting` to `/Users/moringchen/Downloads/待回复.mp3` and `completed` to `/Users/moringchen/Downloads/任务完成.mp3`, then launches macOS `afplay` asynchronously. Register it in `src-tauri/src/main.rs`.

- [ ] **Step 5: Run tests**

Run: `npm test -- src/App.test.tsx`
Expected: PASS.

### Task 2: macOS z-order lowering

**Files:**
- Modify: `src-tauri/src/macos_window_behavior.rs`
- Modify: `src-tauri/src/main.rs`
- Create or modify: `src-tauri/tests/macos_window_behavior.rs`

- [ ] **Step 1: Write the failing test**

Add Rust tests asserting foreground level is floating (`5`) and background level is normal (`0`), not torn-off-menu (`3`).

- [ ] **Step 2: Run Rust test to verify it fails**

Run: `cargo test macos_window_behavior --manifest-path src-tauri/Cargo.toml`
Expected: FAIL because background level is not exposed or still effectively level `3`.

- [ ] **Step 3: Implement minimal z-order change**

Expose `macos_window_level_for_mode(OverlayZOrderMode)` returning `5` for foreground and `0` for background, and use it in `src-tauri/src/main.rs` for focus/unfocus.

- [ ] **Step 4: Run Rust tests**

Run: `cargo test macos_window_behavior --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

### Task 3: Final verification

- [ ] Run: `npm test`
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] If feasible, run the Tauri dev app and manually verify sound on status changes and overlay lowering behind another app.

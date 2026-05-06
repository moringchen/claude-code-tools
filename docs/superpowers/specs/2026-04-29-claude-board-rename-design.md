# claudeBoard Rename Design

**Date:** 2026-04-29  
**Status:** Approved for implementation

## Goal

Rename the newly added Claude Task Window tool to `claudeBoard` consistently across the repository before public release.

## Scope

### In scope
- Rename the tool directory from `tools/claudeTaskWindow/` to `tools/claudeBoard/`.
- Rename user-facing product text from `Claude Task Window` to `claudeBoard`.
- Rename npm package metadata from `claude-task-window` to `claude-board`.
- Rename Rust crate and binary identifiers from `claude_task_window` / `claude_task_windowd` to `claude_board` / `claude_boardd`.
- Rename Tauri product and bundle identifier to `claudeBoard` and `com.moringchen.claude-board`.
- Rename local install/config paths from `~/.claude-task-window` to `~/.claude-board`.
- Rename environment variables from `CLAUDE_TASK_WINDOW_*` to `CLAUDE_BOARD_*`.
- Update README files, docs check script, install/uninstall scripts, tests, and imports.
- Run the existing documentation, frontend, Rust, and build verification commands.

### Out of scope
- Changing product behavior or UI layout.
- Adding compatibility shims for the old `claudeTaskWindow` path, old package name, old Rust crate name, old local directory, or old environment variables.
- Publishing release artifacts.

## Recommended approach

Perform a complete consistent rename. This avoids shipping a public project where the visible name, directory name, package name, Rust crate, Tauri identifier, local storage path, and environment variables disagree.

## Alternatives considered

### Option 1: Display-only rename
Change only README and UI strings.

**Pros:** Smallest diff.  
**Cons:** Internal names remain confusing and unsuitable for public release.

### Option 2: Directory and docs rename only
Rename the tool directory and documentation, but keep package/crate/env names.

**Pros:** Moderate diff.  
**Cons:** Users and maintainers still see `task-window` names in package metadata, binaries, and scripts.

### Option 3: Complete consistent rename
Rename all public and internal identifiers to `claudeBoard`-aligned forms.

**Pros:** Clean public release surface and simpler future maintenance.  
**Cons:** Largest diff and requires thorough test updates.

## Detailed design

### Naming map

- Directory: `tools/claudeTaskWindow/` → `tools/claudeBoard/`
- Product text: `Claude Task Window` → `claudeBoard`
- npm package: `claude-task-window` → `claude-board`
- Storage key: `claude-task-window.preferences` → `claude-board.preferences`
- Local install directory: `.claude-task-window` → `.claude-board`
- Environment variables:
  - `CLAUDE_TASK_WINDOW_BUFFER_PATH` → `CLAUDE_BOARD_BUFFER_PATH`
  - `CLAUDE_TASK_WINDOW_DEBUG` → `CLAUDE_BOARD_DEBUG`
- Rust crate/lib: `claude_task_window` → `claude_board`
- Rust daemon binary: `claude_task_windowd` → `claude_boardd`
- Tauri product name: `Claude Task Window` → `claudeBoard`
- Tauri identifier: `com.moringchen.claude-task-window` → `com.moringchen.claude-board`

### Files to update

Update all references in:
- root `README.md`
- root `README_CN.md`
- tool `README.md` and `README_CN.md`
- `package.json` and `package-lock.json`
- `index.html`
- `src/lib/settings.ts` and related tests
- `scripts/*.sh`, `scripts/*.ps1`, and `scripts/check-docs.py`
- `src-tauri/Cargo.toml`, `Cargo.lock`, bin filename, imports, and tests
- `src-tauri/tauri.conf.json`

### Verification

After implementation, run:
- `python3 tools/claudeBoard/scripts/check-docs.py`
- `cd tools/claudeBoard && npm test`
- `cd tools/claudeBoard && npm run build`
- `cd tools/claudeBoard && cargo test --manifest-path src-tauri/Cargo.toml`
- grep for old names and confirm no live source/docs references remain, excluding git history and dependency caches.

## Success criteria

- No source, test, script, or documentation path depends on `claudeTaskWindow`.
- No user-facing text still calls the tool `Claude Task Window`.
- No runtime package/crate/binary/env/storage identifiers still use `task-window` or `task_window`.
- Existing docs, frontend tests, Rust tests, and frontend build pass after the rename.

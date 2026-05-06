# claudeBoard Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the existing `Claude Task Window` tool to `claudeBoard` consistently across paths, product text, package metadata, Rust identifiers, scripts, docs, and tests.

**Architecture:** This is a mechanical rename with no behavior changes. First move the tool directory and update documentation/check scripts, then update frontend package/UI metadata, then update Rust crate/binary imports and script/env identifiers, and finally verify no old live references remain.

**Tech Stack:** Markdown, Python docs check, npm/Vite/React/TypeScript/Vitest, Rust/Cargo/Tauri, shell scripts, PowerShell scripts

---

## File structure

- Move: `tools/claudeTaskWindow/` -> `tools/claudeBoard/`
- Modify: `README.md` — tool link, title, quick start path, structure tree
- Modify: `README_CN.md` — tool link, title, quick start path, structure tree
- Modify: `tools/claudeBoard/README.md` — product name and paths
- Modify: `tools/claudeBoard/README_CN.md` — product name and paths
- Modify: `tools/claudeBoard/scripts/check-docs.py` — checked paths/fragments
- Modify: `tools/claudeBoard/package.json` — npm package name
- Modify: `tools/claudeBoard/package-lock.json` — npm package name
- Modify: `tools/claudeBoard/index.html` — page title
- Modify: `tools/claudeBoard/src/lib/settings.ts` — storage key
- Modify: `tools/claudeBoard/src/lib/settings.test.ts` — any expected persisted key behavior if exposed
- Modify: `tools/claudeBoard/scripts/*.sh` — install dir, output text, env names
- Modify: `tools/claudeBoard/scripts/*.ps1` — install dir, output text, env names
- Modify: `tools/claudeBoard/src-tauri/Cargo.toml` — crate/lib/bin names and bin path
- Modify: `tools/claudeBoard/src-tauri/Cargo.lock` — crate name
- Move: `tools/claudeBoard/src-tauri/src/bin/claude_task_windowd.rs` -> `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs`
- Modify: `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs` — crate import
- Modify: `tools/claudeBoard/src-tauri/tests/*.rs` — crate imports and expected paths
- Modify: `tools/claudeBoard/src-tauri/tauri.conf.json` — productName, identifier, title

No `.specify/memory/constitution.md` file exists in this repository, so there are no additional Spec-Kit constitution constraints to apply.

### Task 1: Move the tool directory and update documentation checks

**Files:**
- Move: `tools/claudeTaskWindow/` -> `tools/claudeBoard/`
- Modify: `tools/claudeBoard/scripts/check-docs.py`
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `tools/claudeBoard/README.md`
- Modify: `tools/claudeBoard/README_CN.md`

- [ ] **Step 1: Run the old docs check to establish baseline**

Run: `python3 tools/claudeTaskWindow/scripts/check-docs.py`
Expected: PASS with `Documentation checks passed.`

- [ ] **Step 2: Move the directory**

Run:
```bash
git mv tools/claudeTaskWindow tools/claudeBoard
```

- [ ] **Step 3: Update docs checker**

Replace `tools/claudeBoard/scripts/check-docs.py` with:

```python
from pathlib import Path

root = Path(__file__).resolve().parents[3]
checks = {
    root / "tools" / "claudeBoard" / "README.md": [
        "claudeBoard",
        "macOS",
        "Windows 10",
        "global hooks",
        "当前无任务",
    ],
    root / "tools" / "claudeBoard" / "README_CN.md": [
        "claudeBoard",
        "macOS",
        "Windows 11",
        "全局 hooks",
        "当前无任务",
    ],
    root / "README.md": ["claudeBoard", "tools/claudeBoard/"],
    root / "README_CN.md": ["claudeBoard", "tools/claudeBoard/"],
}

for path, fragments in checks.items():
    content = path.read_text(encoding="utf-8")
    for fragment in fragments:
        assert fragment in content, f"{fragment!r} missing from {path}"

print("Documentation checks passed.")
```

- [ ] **Step 4: Update English root README**

In `README.md`, replace the Claude Task Window section and tree entries so they contain:

```md
### 3. [claudeBoard](tools/claudeBoard/)

A desktop overlay for Claude Code task activity with user-level global hooks, an always-visible top bar, task counts, click-to-focus actions, and notification or voice toggles.

**Quick Start:**
```bash
cd tools/claudeBoard
npm install
npm test
```
```

And the structure entry:

```md
    └── claudeBoard/                 # claudeBoard desktop overlay
        ├── README.md                # Tool documentation (EN)
        ├── README_CN.md             # Tool documentation (CN)
        ├── package.json
        ├── src/
        ├── scripts/
        └── src-tauri/
```

- [ ] **Step 5: Update Chinese root README**

In `README_CN.md`, replace the Claude Task Window section and tree entries so they contain:

```md
### 3. [claudeBoard](tools/claudeBoard/)

用于展示 Claude Code 任务活动的桌面悬浮层，提供用户级全局 hooks、顶部常驻条、任务统计、点击聚焦，以及通知与语音播报开关。

**快速开始：**
```bash
cd tools/claudeBoard
npm install
npm test
```
```

And the structure entry:

```md
    └── claudeBoard/                 # claudeBoard 桌面悬浮工具
        ├── README.md                # 工具文档（英文）
        ├── README_CN.md             # 工具文档（中文）
        ├── package.json
        ├── src/
        ├── scripts/
        └── src-tauri/
```

- [ ] **Step 6: Update tool READMEs**

In `tools/claudeBoard/README.md` and `tools/claudeBoard/README_CN.md`, replace:
- `Claude Task Window` with `claudeBoard`
- `tools/claudeTaskWindow` with `tools/claudeBoard`

- [ ] **Step 7: Run docs check**

Run: `python3 tools/claudeBoard/scripts/check-docs.py`
Expected: PASS with `Documentation checks passed.`

- [ ] **Step 8: Commit**

```bash
git add README.md README_CN.md tools/claudeBoard
git commit -m "rename Claude Task Window docs to claudeBoard"
```

### Task 2: Rename frontend package, UI metadata, storage keys, and scripts

**Files:**
- Modify: `tools/claudeBoard/package.json`
- Modify: `tools/claudeBoard/package-lock.json`
- Modify: `tools/claudeBoard/index.html`
- Modify: `tools/claudeBoard/src/lib/settings.ts`
- Modify: `tools/claudeBoard/scripts/hook-dispatch.sh`
- Modify: `tools/claudeBoard/scripts/hook-dispatch.ps1`
- Modify: `tools/claudeBoard/scripts/install-macos.sh`
- Modify: `tools/claudeBoard/scripts/uninstall-macos.sh`
- Modify: `tools/claudeBoard/scripts/install-windows.ps1`
- Modify: `tools/claudeBoard/scripts/uninstall-windows.ps1`

- [ ] **Step 1: Write the failing verification search**

Run:
```bash
grep -R "claude-task-window\|Claude Task Window\|claudeTaskWindow\|CLAUDE_TASK_WINDOW\|\.claude-task-window" tools/claudeBoard README.md README_CN.md
```
Expected: FAIL by printing old frontend/script references.

- [ ] **Step 2: Update package metadata and UI title**

Set `tools/claudeBoard/package.json` name:

```json
"name": "claude-board"
```

Set the same package name in the root package entries of `tools/claudeBoard/package-lock.json`.

Set `tools/claudeBoard/index.html` title:

```html
<title>claudeBoard</title>
```

- [ ] **Step 3: Update frontend storage key**

In `tools/claudeBoard/src/lib/settings.ts`, set:

```ts
const STORAGE_KEY = "claude-board.preferences";
```

- [ ] **Step 4: Update POSIX scripts**

In shell scripts under `tools/claudeBoard/scripts/`, replace:
- `.claude-task-window` with `.claude-board`
- `CLAUDE_TASK_WINDOW_BUFFER_PATH` with `CLAUDE_BOARD_BUFFER_PATH`
- `Claude Task Window` with `claudeBoard`

The dispatch buffer line should be:

```bash
buffer_path="${CLAUDE_BOARD_BUFFER_PATH:-$HOME/.claude-board/events.jsonl}"
```

- [ ] **Step 5: Update PowerShell scripts**

In PowerShell scripts under `tools/claudeBoard/scripts/`, replace:
- `.claude-task-window` with `.claude-board`
- `CLAUDE_TASK_WINDOW_BUFFER_PATH` with `CLAUDE_BOARD_BUFFER_PATH`
- `CLAUDE_TASK_WINDOW_DEBUG` with `CLAUDE_BOARD_DEBUG`
- `Claude Task Window` with `claudeBoard`

The dispatch buffer selection should use:

```powershell
$bufferPath = if ($env:CLAUDE_BOARD_BUFFER_PATH) { $env:CLAUDE_BOARD_BUFFER_PATH } else { Join-Path $HOME '.claude-board/events.jsonl' }
$debugEnabled = -not [string]::IsNullOrEmpty($env:CLAUDE_BOARD_DEBUG)
```

- [ ] **Step 6: Run frontend tests and docs check**

Run:
```bash
python3 tools/claudeBoard/scripts/check-docs.py
cd tools/claudeBoard && npm test && npm run build
```
Expected: docs check passes, 6 Vitest files pass, Vite build succeeds.

- [ ] **Step 7: Confirm frontend/script old names are gone**

Run:
```bash
grep -R "claude-task-window\|Claude Task Window\|claudeTaskWindow\|CLAUDE_TASK_WINDOW\|\.claude-task-window" tools/claudeBoard README.md README_CN.md --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=target
```
Expected: no output.

- [ ] **Step 8: Commit**

```bash
git add tools/claudeBoard README.md README_CN.md
git commit -m "rename frontend and scripts to claudeBoard"
```

### Task 3: Rename Rust crate, binary, imports, and Tauri metadata

**Files:**
- Modify: `tools/claudeBoard/src-tauri/Cargo.toml`
- Modify: `tools/claudeBoard/src-tauri/Cargo.lock`
- Move: `tools/claudeBoard/src-tauri/src/bin/claude_task_windowd.rs` -> `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs`
- Modify: `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs`
- Modify: `tools/claudeBoard/src-tauri/tests/*.rs`
- Modify: `tools/claudeBoard/src-tauri/tauri.conf.json`

- [ ] **Step 1: Run the failing Rust old-name search**

Run:
```bash
grep -R "claude_task_window\|claude_task_windowd\|claude-task-window\|Claude Task Window" tools/claudeBoard/src-tauri --exclude-dir=target
```
Expected: FAIL by printing old Rust/Tauri names.

- [ ] **Step 2: Update Cargo manifest**

In `tools/claudeBoard/src-tauri/Cargo.toml`, set:

```toml
[package]
name = "claude_board"

[lib]
name = "claude_board"
path = "src/lib.rs"

[[bin]]
name = "claude_boardd"
path = "src/bin/claude_boardd.rs"
```

Keep existing version, edition, and dependencies unchanged.

- [ ] **Step 3: Move daemon binary and update import**

Run:
```bash
git mv tools/claudeBoard/src-tauri/src/bin/claude_task_windowd.rs tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs
```

In `tools/claudeBoard/src-tauri/src/bin/claude_boardd.rs`, replace:

```rust
use claude_task_window::{server::build_router, store::TaskStore};
```

with:

```rust
use claude_board::{server::build_router, store::TaskStore};
```

- [ ] **Step 4: Update Rust test imports**

In every file under `tools/claudeBoard/src-tauri/tests/`, replace `claude_task_window` with `claude_board`.

Also update hook path expectations from `~/.claude-task-window/hook-dispatch.sh` to:

```rust
"~/.claude-board/hook-dispatch.sh"
```

- [ ] **Step 5: Update Tauri config**

In `tools/claudeBoard/src-tauri/tauri.conf.json`, set:

```json
"productName": "claudeBoard",
"identifier": "com.moringchen.claude-board"
```

And window title:

```json
"title": "claudeBoard"
```

- [ ] **Step 6: Regenerate Cargo lock metadata**

Run:
```bash
cd tools/claudeBoard && cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass and `Cargo.lock` crate metadata is updated to `claude_board`.

- [ ] **Step 7: Confirm Rust old names are gone**

Run:
```bash
grep -R "claude_task_window\|claude_task_windowd\|claude-task-window\|Claude Task Window" tools/claudeBoard/src-tauri --exclude-dir=target
```
Expected: no output.

- [ ] **Step 8: Commit**

```bash
git add tools/claudeBoard
git commit -m "rename Rust crate and Tauri app to claudeBoard"
```

### Task 4: Final old-reference sweep and verification

**Files:**
- Modify only if old references are found in live files.

- [ ] **Step 1: Run complete old-name sweep**

Run:
```bash
grep -R "claudeTaskWindow\|Claude Task Window\|claude-task-window\|claude_task_window\|claude_task_windowd\|CLAUDE_TASK_WINDOW\|\.claude-task-window" README.md README_CN.md tools/claudeBoard --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=target
```
Expected: no output.

- [ ] **Step 2: Run full verification**

Run:
```bash
python3 tools/claudeBoard/scripts/check-docs.py
cd tools/claudeBoard && npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml
```
Expected:
- docs check passes
- 6 Vitest files pass
- frontend build succeeds
- all Rust tests pass

- [ ] **Step 3: Check git status**

Run:
```bash
git status --short
```

Expected: only intentional tracked changes for the rename plus unrelated pre-existing untracked docs files if they were already present.

- [ ] **Step 4: Commit final cleanup if needed**

If Step 1 or Step 2 required fixes, commit them:

```bash
git add README.md README_CN.md tools/claudeBoard
git commit -m "verify claudeBoard rename"
```

If there were no fixes after Task 3, skip this commit.

## Plan self-review

**Spec coverage:**
- Directory rename is covered by Task 1.
- User-facing product text and README updates are covered by Task 1 and Task 2.
- npm package metadata and storage key are covered by Task 2.
- local directory and environment variable rename are covered by Task 2.
- Rust crate, binary, imports, and Tauri metadata are covered by Task 3.
- final grep and verification commands are covered by Task 4.

**Placeholder scan:**
- No TBD/TODO placeholders remain.
- Every step includes exact commands or exact replacement content.

**Type consistency:**
- User-facing name: `claudeBoard`.
- directory: `tools/claudeBoard`.
- npm package: `claude-board`.
- Rust crate: `claude_board`.
- Rust daemon: `claude_boardd`.
- local directory: `.claude-board`.
- environment prefix: `CLAUDE_BOARD`.

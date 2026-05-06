# cliTitleName README Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate terminal-title documentation under `tools/cliTitleName/`, standardize quick-install snippets to `curl -fsSL ... | bash` plus `source ~/.zshrc`, and remove the old `tools/current-window-title-lock/` tool.

**Architecture:** Keep the functional implementation centered on `tools/cliTitleName/` and treat this as a documentation-and-structure cleanup. Use lightweight shell-based doc assertions so the change is verified automatically, then remove the superseded `current-window-title-lock` directory once every public README points to `cliTitleName` only.

**Tech Stack:** Markdown README files, Bash/zsh install snippets, zsh-based verification scripts, git file deletion.

---

## File Structure

- `tools/cliTitleName/README.md`
  - English tool README; update install block to include the GitHub raw URL and `source ~/.zshrc`.
- `tools/cliTitleName/README_CN.md`
  - Chinese tool README; same install-block update.
- `tools/cliTitleName/test_cliTitleName.zsh`
  - Existing regression suite; extend it to assert the updated install snippets.
- `tools/zsh-completion/README.md`
  - English completion README; replace local setup invocation with a curl-based quick-install block.
- `tools/zsh-completion/README_CN.md`
  - Chinese completion README; same quick-install update.
- `tools/zsh-completion/test_zsh_completion_docs.zsh`
  - New lightweight doc-check script for the completion README files.
- `README.md`
  - Top-level English README; replace the old title-lock tool section with `cliTitleName` and update the structure/install sections.
- `README_CN.md`
  - Top-level Chinese README; same consolidation.
- `tools/current-window-title-lock/`
  - Delete this directory entirely after the README updates are in place.

### Task 1: Update cliTitleName tool docs and assertions

**Files:**
- Modify: `tools/cliTitleName/README.md`
- Modify: `tools/cliTitleName/README_CN.md`
- Modify: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `tools/cliTitleName/test_cliTitleName.zsh`

- [ ] **Step 1: Write the failing test**

Add four assertions to `tools/cliTitleName/test_cliTitleName.zsh` immediately after the existing install-path assertions near the README checks:

```zsh
assert_contains "$README_EN_CONTENTS" 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash' 'README.md should document the GitHub raw installer command'
assert_contains "$README_EN_CONTENTS" 'source ~/.zshrc' 'README.md should tell users to source ~/.zshrc in the install snippet'
assert_contains "$README_CN_CONTENTS" 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash' 'README_CN.md should document the GitHub raw installer command'
assert_contains "$README_CN_CONTENTS" 'source ~/.zshrc' 'README_CN.md should tell users to source ~/.zshrc in the install snippet'
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: FAIL because the current README install blocks do not yet contain `source ~/.zshrc`.

- [ ] **Step 3: Write the minimal implementation**

Replace the install blocks in `tools/cliTitleName/README.md` and `tools/cliTitleName/README_CN.md` with the exact text below.

`tools/cliTitleName/README.md`

```md
## Install

```sh
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

After installation, interactive zsh shells load `titlename` as a shell function from `~/.config/cliTitleName/titlename.zsh`. The executable at `~/.local/bin/titlename` remains available as the fallback outside that shell-function context.
```

`tools/cliTitleName/README_CN.md`

```md
## 安装

```sh
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

安装后，交互式 zsh 会从 `~/.config/cliTitleName/titlename.zsh` 加载 `titlename` shell function。`~/.local/bin/titlename` 这个可执行文件仍然会保留，用作非 shell-function 场景下的回退入口。
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: PASS with output `ok`.

- [ ] **Step 5: Commit**

```bash
git add tools/cliTitleName/README.md tools/cliTitleName/README_CN.md tools/cliTitleName/test_cliTitleName.zsh
git commit -m "docs: align cliTitleName install instructions"
```

### Task 2: Standardize zsh-completion quick-install docs

**Files:**
- Modify: `tools/zsh-completion/README.md`
- Modify: `tools/zsh-completion/README_CN.md`
- Create: `tools/zsh-completion/test_zsh_completion_docs.zsh`
- Test: `tools/zsh-completion/test_zsh_completion_docs.zsh`

- [ ] **Step 1: Write the failing test**

Create `tools/zsh-completion/test_zsh_completion_docs.zsh` with exactly this content:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
README_EN_CONTENTS=$(<"$SCRIPT_DIR/README.md")
README_CN_CONTENTS=$(<"$SCRIPT_DIR/README_CN.md")

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" != *"$needle"* ]]; then
    print -u2 -- "$message"
    print -u2 -- "missing: $needle"
    exit 1
  fi
}

assert_contains "$README_EN_CONTENTS" 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash' 'README.md should document the GitHub raw installer command'
assert_contains "$README_EN_CONTENTS" 'source ~/.zshrc' 'README.md should include source ~/.zshrc in the install snippet'
assert_contains "$README_CN_CONTENTS" 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash' 'README_CN.md should document the GitHub raw installer command'
assert_contains "$README_CN_CONTENTS" 'source ~/.zshrc' 'README_CN.md should include source ~/.zshrc in the install snippet'

print 'ok'
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
zsh tools/zsh-completion/test_zsh_completion_docs.zsh
```

Expected: FAIL because the current README files still tell the user to run `zsh setup_claude_completion.sh` locally.

- [ ] **Step 3: Write the minimal implementation**

Replace the installation sections in the two README files with the exact text below.

`tools/zsh-completion/README.md`

```md
## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```
```

`tools/zsh-completion/README_CN.md`

```md
## 安装方法

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
zsh tools/zsh-completion/test_zsh_completion_docs.zsh
```

Expected: PASS with output `ok`.

- [ ] **Step 5: Commit**

```bash
git add tools/zsh-completion/README.md tools/zsh-completion/README_CN.md tools/zsh-completion/test_zsh_completion_docs.zsh
git commit -m "docs: standardize zsh completion install snippet"
```

### Task 3: Consolidate top-level READMEs and remove the old title-lock tool

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Delete: `tools/current-window-title-lock/current_window_title_lock.zsh`
- Delete: `tools/current-window-title-lock/README.md`
- Delete: `tools/current-window-title-lock/README_CN.md`
- Delete: `tools/current-window-title-lock/test_current_window_title_lock.zsh`
- Test: `README.md`
- Test: `README_CN.md`

- [ ] **Step 1: Write the failing verification command**

Run this command before editing the top-level READMEs:

```bash
! grep -q 'current-window-title-lock' README.md && \
! grep -q 'current-window-title-lock' README_CN.md && \
grep -q 'tools/cliTitleName/' README.md && \
grep -q 'tools/cliTitleName/' README_CN.md && \
! test -e tools/current-window-title-lock
```

Expected: FAIL because both READMEs still reference `current-window-title-lock` and the directory still exists.

- [ ] **Step 2: Write the minimal implementation**

Replace the existing tool-2 sections in the top-level README files with the exact text below.

`README.md`

```md
### 2. [cliTitleName](tools/cliTitleName/)

Set the current terminal window title with a single `titlename` command.

**Key Features:**
- Set the current terminal window title immediately
- In Ghostty + interactive zsh, disable later title rewrites for the current shell session
- Keep the command surface to a single `titlename "..."` invocation
- Restore normal Ghostty title automation automatically in a new shell

**Quick Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```
```

Update the project-structure block in `README.md` so the `tools/` section becomes:

```md
└── tools/
    ├── cliTitleName/                # Terminal title tool
    │   ├── titlename
    │   ├── titlename.zsh
    │   ├── install.sh
    │   ├── test_cliTitleName.zsh
    │   ├── README.md                # Tool documentation (EN)
    │   └── README_CN.md             # Tool documentation (CN)
    └── zsh-completion/              # Zsh completion tool
        ├── setup_claude_completion.sh
        ├── README.md                # Tool documentation (EN)
        └── README_CN.md             # Tool documentation (CN)
```

Replace the install example in the `## Installation` section of `README.md` with:

```md
```bash
# Claude CLI Zsh completion
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc

# cliTitleName
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```
```

`README_CN.md`

```md
### 2. [cliTitleName](tools/cliTitleName/)

使用单个 `titlename` 命令设置当前终端窗口标题。

**主要功能：**
- 立即设置当前终端窗口标题
- 在 Ghostty + 交互式 zsh 中，禁用当前 shell 会话后续的标题改写
- 保持单一命令入口：`titlename "..."`
- 新开一个 shell 后自动恢复 Ghostty 默认标题行为

**快速安装：**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```
```

Update the project-structure block in `README_CN.md` so the `tools/` section becomes:

```md
└── tools/
    ├── cliTitleName/                # 终端标题工具
    │   ├── titlename
    │   ├── titlename.zsh
    │   ├── install.sh
    │   ├── test_cliTitleName.zsh
    │   ├── README.md                # 工具文档（英文）
    │   └── README_CN.md             # 工具文档（中文）
    └── zsh-completion/              # Zsh 自动补齐工具
        ├── setup_claude_completion.sh
        ├── README.md                # 工具文档（英文）
        └── README_CN.md             # 工具文档（中文）
```

Replace the install example in the `## 安装方法` section of `README_CN.md` with:

```md
```bash
# Claude CLI Zsh 参数自动补齐
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc

# cliTitleName
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```
```

Then delete the entire `tools/current-window-title-lock/` directory.

- [ ] **Step 3: Run verification to prove the consolidation works**

Run:

```bash
! grep -q 'current-window-title-lock' README.md && \
! grep -q 'current-window-title-lock' README_CN.md && \
grep -q '### 2. \[cliTitleName\](tools/cliTitleName/)' README.md && \
grep -q '### 2. \[cliTitleName\](tools/cliTitleName/)' README_CN.md && \
grep -q 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash' README.md && \
grep -q 'curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash' README_CN.md && \
! test -e tools/current-window-title-lock
```

Expected: PASS with exit code 0 and no output.

- [ ] **Step 4: Commit**

```bash
git add README.md README_CN.md
git add -u tools/current-window-title-lock
git commit -m "docs: consolidate title tool references"
```

### Task 4: Run final verification for the whole change

**Files:**
- Test: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `tools/zsh-completion/test_zsh_completion_docs.zsh`
- Test: `README.md`
- Test: `README_CN.md`

- [ ] **Step 1: Run the tool README checks**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh && zsh tools/zsh-completion/test_zsh_completion_docs.zsh
```

Expected: PASS with two `ok` lines.

- [ ] **Step 2: Run the repository consolidation check**

Run:

```bash
! grep -q 'current-window-title-lock' README.md && \
! grep -q 'current-window-title-lock' README_CN.md && \
! test -e tools/current-window-title-lock
```

Expected: PASS with exit code 0 and no output.

- [ ] **Step 3: Inspect final git state**

Run:

```bash
git status --short
```

Expected: clean working tree.
```

## Self-Review

- **Spec coverage:** Task 1 covers the `cliTitleName` GitHub raw URL and `source ~/.zshrc`; Task 2 covers the zsh-completion curl-based quick install; Task 3 replaces the top-level title-lock documentation and removes the old directory; Task 4 verifies the consolidated state.
- **Placeholder scan:** No `TODO`, `TBD`, or “similar to above” placeholders remain.
- **Type consistency:** All path references use the same canonical names: `tools/cliTitleName/`, `tools/zsh-completion/`, and `tools/current-window-title-lock/` only as the directory being removed.

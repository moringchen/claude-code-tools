# cliTitleName Session Title Disable Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep `titlename` as a single user-facing command while making it disable Ghostty's later title rewrites for the current zsh shell session.

**Architecture:** Preserve the existing executable `titlename` script as the non-interactive fallback, and add a zsh function implementation that can mutate the current shell session. Install that function into interactive zsh via a managed `~/.zshrc` source block, and strip only Ghostty's OSC 2 title writes from `_ghostty_precmd` and `_ghostty_preexec` so prompt markers, cwd reporting, and other shell integration behavior stay intact.

**Tech Stack:** Bash installer, zsh function integration, Ghostty shell integration hooks, zsh-based regression tests.

---

## File Structure

- `tools/cliTitleName/titlename`
  - Keep as the executable fallback for non-function contexts.
- `tools/cliTitleName/titlename.zsh`
  - New authoritative zsh function implementation for session-level title disabling.
- `tools/cliTitleName/install.sh`
  - Install both the fallback executable and the zsh function file; add a managed source block to `~/.zshrc`.
- `tools/cliTitleName/test_cliTitleName.zsh`
  - Extend tests for function-mode Ghostty behavior, installer output, installer idempotence, and updated docs.
- `tools/cliTitleName/README.md`
  - Update English docs for the new Ghostty + interactive zsh behavior.
- `tools/cliTitleName/README_CN.md`
  - Update Chinese docs for the new Ghostty + interactive zsh behavior.

### Task 1: Add the zsh function implementation and Ghostty hook tests

**Files:**
- Create: `tools/cliTitleName/titlename.zsh`
- Modify: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `tools/cliTitleName/test_cliTitleName.zsh`

- [ ] **Step 1: Write the failing test**

Replace `tools/cliTitleName/test_cliTitleName.zsh` with the version below so the suite starts asserting function-mode behavior before `titlename.zsh` exists:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
COMMAND="$SCRIPT_DIR/titlename"
FUNCTION_FILE="$SCRIPT_DIR/titlename.zsh"
INSTALLER="$SCRIPT_DIR/install.sh"

assert_eq() {
  local expected="$1"
  local actual="$2"
  local message="$3"

  if [[ "$actual" != "$expected" ]]; then
    print -u2 -- "$message"
    print -u2 -- "expected: $expected"
    print -u2 -- "actual:   $actual"
    exit 1
  fi
}

assert_true() {
  local condition="$1"
  local message="$2"

  if ! eval "$condition"; then
    print -u2 -- "$message"
    exit 1
  fi
}

run_capture() {
  local stdout_file stderr_file
  stdout_file=$(mktemp)
  stderr_file=$(mktemp)

  set +e
  "$@" >"$stdout_file" 2>"$stderr_file"
  RUN_STATUS=$?
  set -e

  RUN_STDOUT=$(<"$stdout_file")
  RUN_STDERR=$(<"$stderr_file")
  rm -f "$stdout_file" "$stderr_file"
}

run_ghostty_function_scenario() {
  local script_file state_file
  script_file=$(mktemp)
  state_file=$(mktemp)

  cat >"$script_file" <<'EOF'
#!/usr/bin/env zsh
set -euo pipefail

source "__FUNCTION_FILE__"
export TERM_PROGRAM=ghostty

functions[_ghostty_precmd]=$'builtin print -rnu $_ghostty_fd $\'\\e]133;A;cl=line\\a\'\n          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${(%):-%(4~|…/%3~|%~)}"$\'\\a\'\n          builtin print -rnu $_ghostty_fd $\'\\e]133;B\\a\''
functions[_ghostty_preexec]=$'builtin print -rnu $_ghostty_fd $\'\\e]133;C\\a\'\n          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${1//[[:cntrl:]]}"$\'\\a\''

titlename "Claude Window"

{
  print -r -- '__PRECMD__'
  print -r -- "$functions[_ghostty_precmd]"
  print -r -- '__PREEXEC__'
  print -r -- "$functions[_ghostty_preexec]"
} >"__STATE_FILE__"
EOF

  python3 - <<'PY' "$script_file" "$FUNCTION_FILE" "$state_file"
from pathlib import Path
import sys
script_path = Path(sys.argv[1])
function_path = sys.argv[2]
state_path = sys.argv[3]
text = script_path.read_text()
text = text.replace('__FUNCTION_FILE__', function_path)
text = text.replace('__STATE_FILE__', state_path)
script_path.write_text(text)
PY

  chmod +x "$script_file"
  run_capture zsh "$script_file"
  FUNCTION_STATE=$(<"$state_file")
  rm -f "$script_file" "$state_file"
}

typeset -g RUN_STATUS=0
TYPESSET_GUARD=1
unset TYPESSET_GUARD

typeset -g RUN_STDOUT=''
typeset -g RUN_STDERR=''
typeset -g FUNCTION_STATE=''

assert_true '[[ -f "$COMMAND" ]]' 'titlename should exist'
assert_true '[[ -x "$COMMAND" ]]' 'titlename should be executable'
assert_true '[[ -f "$FUNCTION_FILE" ]]' 'titlename.zsh should exist'

run_capture "$COMMAND" 'Claude Window'
assert_eq '0' "$RUN_STATUS" 'titlename should succeed with one non-empty argument'
assert_eq $'\e]2;Claude Window\a' "$RUN_STDOUT" 'titlename should emit the expected OSC title sequence'
assert_eq '' "$RUN_STDERR" 'titlename should not write stderr on success'

run_ghostty_function_scenario
assert_eq '0' "$RUN_STATUS" 'function-mode titlename should succeed in Ghostty mode'
assert_true '[[ "$RUN_STDOUT" == $'"'"'\e]2;Claude Window\a'"'"' ]]' 'function-mode titlename should emit the requested title sequence'
assert_true '[[ "$FUNCTION_STATE" == *"__PRECMD__"* ]]' 'function-mode scenario should capture prompt state'
assert_true '[[ "$FUNCTION_STATE" != *$'"'"'\e]2;'"'"'* ]]' 'Ghostty title writes should be removed from prompt hooks'
assert_true '[[ "$FUNCTION_STATE" == *$'"'"'\e]133;A;cl=line\a'"'"'* ]]' 'Ghostty semantic prompt markers should remain'
assert_true '[[ "$FUNCTION_STATE" == *$'"'"'\e]133;C\a'"'"'* ]]' 'Ghostty preexec markers should remain'

run_capture "$COMMAND"
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail without arguments'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on usage failure'
assert_eq 'usage: titlename "Window Title"' "$RUN_STDERR" 'titlename should print the exact usage message without arguments'

run_capture "$COMMAND" ''
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail with an empty string argument'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on empty-string failure'
assert_eq 'usage: titlename "Window Title"' "$RUN_STDERR" 'titlename should print the exact usage message for an empty string argument'

fake_bin_dir=$(mktemp -d)
cat >"$fake_bin_dir/uname" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' 'Linux'
EOF
chmod +x "$fake_bin_dir/uname"

run_capture env PATH="$fake_bin_dir:$PATH" "$COMMAND" 'Claude Window'
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail on non-macOS platforms'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on non-macOS failure'
assert_eq 'titlename supports macOS only' "$RUN_STDERR" 'titlename should print the exact macOS-only error'

rm -rf "$fake_bin_dir"

print 'ok'
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: FAIL with `titlename.zsh should exist`.

- [ ] **Step 3: Write minimal implementation**

Create `tools/cliTitleName/titlename.zsh` with exactly this content:

```zsh
# shellcheck shell=zsh

_titlename_disable_ghostty_title_updates() {
  emulate -L zsh

  local prompt_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${(%):-%(4~|…/%3~|%~)}"$\'\\a\''
  local preexec_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${1//[[:cntrl:]]}"$\'\\a\''

  if (( $+functions[_ghostty_precmd] )); then
    functions[_ghostty_precmd]=${functions[_ghostty_precmd]//$'\n'"$prompt_title_line"/}
  fi

  if (( $+functions[_ghostty_preexec] )); then
    functions[_ghostty_preexec]=${functions[_ghostty_preexec]//$'\n'"$preexec_title_line"/}
  fi
}

titlename() {
  emulate -L zsh

  if [[ "$(uname -s)" != "Darwin" ]]; then
    print -u2 -- 'titlename supports macOS only'
    return 1
  fi

  if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
    print -u2 -- 'usage: titlename "Window Title"'
    return 1
  fi

  printf '\033]2;%s\a' "$1"

  if [[ "${TERM_PROGRAM:-}" == 'ghostty' ]]; then
    _titlename_disable_ghostty_title_updates
  fi
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: PASS with output `ok`.

- [ ] **Step 5: Commit**

```bash
git add tools/cliTitleName/titlename.zsh tools/cliTitleName/test_cliTitleName.zsh
git commit -m "feat: add session-scoped Ghostty title disable"
```

### Task 2: Install the zsh function and managed zshrc source block

**Files:**
- Modify: `tools/cliTitleName/install.sh`
- Modify: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `tools/cliTitleName/test_cliTitleName.zsh`

- [ ] **Step 1: Write the failing installer test**

Extend `tools/cliTitleName/test_cliTitleName.zsh` by replacing the old installer section with the block below:

```zsh
installer_home=$(mktemp -d)
installed_path="$installer_home/.local/bin/titlename"
installed_function_path="$installer_home/.config/cliTitleName/titlename.zsh"
installed_zshrc="$installer_home/.zshrc"
expected_install_stdout=$'Installed titlename to '"$installed_path"$'\nInstalled zsh integration to '"$installed_function_path"$'\nLoaded titlename from '"$installed_zshrc"

run_capture zsh -c 'cat "$1" | env HOME="$2" PATH="/usr/bin:/bin:/usr/sbin:/sbin" bash' -- "$INSTALLER" "$installer_home"
assert_eq '0' "$RUN_STATUS" 'streamed install.sh should succeed on macOS'
assert_true '[[ -f "$installed_path" ]]' 'streamed install.sh should install titlename into ~/.local/bin'
assert_true '[[ -x "$installed_path" ]]' 'streamed install.sh should make the installed titlename executable'
assert_true '[[ -f "$installed_function_path" ]]' 'streamed install.sh should install the zsh function file'
assert_true '[[ -f "$installed_zshrc" ]]' 'streamed install.sh should create ~/.zshrc when needed'
assert_eq "$expected_install_stdout" "$RUN_STDOUT" 'streamed install.sh should print installed paths for the script, zsh integration, and zshrc loader'
assert_eq '' "$RUN_STDERR" 'streamed install.sh should not write stderr on success'
assert_eq "$(<"$COMMAND")" "$(<"$installed_path")" 'streamed install.sh should install the expected fallback executable contents'
assert_eq "$(<"$FUNCTION_FILE")" "$(<"$installed_function_path")" 'streamed install.sh should install the expected zsh function contents'
assert_true '[[ "$(<"$installed_zshrc")" == *"# >>> cliTitleName >>>"* ]]' 'streamed install.sh should add the managed start marker to ~/.zshrc'
assert_true '[[ "$(<"$installed_zshrc")" == *"source \"$HOME/.config/cliTitleName/titlename.zsh\""* ]]' 'streamed install.sh should source the installed zsh function file from ~/.zshrc'
assert_true '[[ "$(<"$installed_zshrc")" == *"# <<< cliTitleName <<<"* ]]' 'streamed install.sh should add the managed end marker to ~/.zshrc'

run_capture zsh -c 'cat "$1" | env HOME="$2" PATH="/usr/bin:/bin:/usr/sbin:/sbin" bash' -- "$INSTALLER" "$installer_home"
assert_eq '0' "$RUN_STATUS" 'streamed install.sh should remain idempotent on the second run'
managed_block_count=$(grep -c '^# >>> cliTitleName >>>$' "$installed_zshrc")
assert_eq '1' "$managed_block_count" 'streamed install.sh should not duplicate the managed ~/.zshrc block'

rm -rf "$installer_home"
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: FAIL because `install.sh` does not yet install `~/.config/cliTitleName/titlename.zsh` or manage `~/.zshrc`.

- [ ] **Step 3: Write minimal implementation**

Replace `tools/cliTitleName/install.sh` with exactly this content:

```bash
#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

INSTALL_DIR="${HOME}/.local/bin"
INSTALL_PATH="$INSTALL_DIR/titlename"
CONFIG_DIR="${HOME}/.config/cliTitleName"
FUNCTION_PATH="$CONFIG_DIR/titlename.zsh"
ZSHRC_PATH="${HOME}/.zshrc"
START_MARK='# >>> cliTitleName >>>'
END_MARK='# <<< cliTitleName <<<'
SOURCE_LINE='[[ -r "$HOME/.config/cliTitleName/titlename.zsh" ]] && source "$HOME/.config/cliTitleName/titlename.zsh"'

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"

cat >"$INSTALL_PATH" <<'EOF'
#!/usr/bin/env bash

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
  printf '%s\n' 'usage: titlename "Window Title"' >&2
  exit 1
fi

printf '\033]2;%s\a' "$1"
EOF
chmod +x "$INSTALL_PATH"

cat >"$FUNCTION_PATH" <<'EOF'
# shellcheck shell=zsh

_titlename_disable_ghostty_title_updates() {
  emulate -L zsh

  local prompt_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${(%):-%(4~|…/%3~|%~)}"$\'\\a\''
  local preexec_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${1//[[:cntrl:]]}"$\'\\a\''

  if (( $+functions[_ghostty_precmd] )); then
    functions[_ghostty_precmd]=${functions[_ghostty_precmd]//$'\n'"$prompt_title_line"/}
  fi

  if (( $+functions[_ghostty_preexec] )); then
    functions[_ghostty_preexec]=${functions[_ghostty_preexec]//$'\n'"$preexec_title_line"/}
  fi
}

titlename() {
  emulate -L zsh

  if [[ "$(uname -s)" != "Darwin" ]]; then
    print -u2 -- 'titlename supports macOS only'
    return 1
  fi

  if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
    print -u2 -- 'usage: titlename "Window Title"'
    return 1
  fi

  printf '\033]2;%s\a' "$1"

  if [[ "${TERM_PROGRAM:-}" == 'ghostty' ]]; then
    _titlename_disable_ghostty_title_updates
  fi
}
EOF

: >"$ZSHRC_PATH"
if [[ -f "$ZSHRC_PATH" ]] && [[ -s "$ZSHRC_PATH" ]]; then
  :
fi
if ! grep -Fqx "$START_MARK" "$ZSHRC_PATH" 2>/dev/null; then
  {
    printf '\n%s\n' "$START_MARK"
    printf '%s\n' "$SOURCE_LINE"
    printf '%s\n' "$END_MARK"
  } >>"$ZSHRC_PATH"
fi

printf 'Installed titlename to %s\n' "$INSTALL_PATH"
printf 'Installed zsh integration to %s\n' "$FUNCTION_PATH"
printf 'Loaded titlename from %s\n' "$ZSHRC_PATH"
```

Then immediately fix the `: >"$ZSHRC_PATH"` truncation bug by changing that section to:

```bash
touch "$ZSHRC_PATH"
if ! grep -Fqx "$START_MARK" "$ZSHRC_PATH" 2>/dev/null; then
  {
    printf '\n%s\n' "$START_MARK"
    printf '%s\n' "$SOURCE_LINE"
    printf '%s\n' "$END_MARK"
  } >>"$ZSHRC_PATH"
fi
```

The final `install.sh` content after both edits must be:

```bash
#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

INSTALL_DIR="${HOME}/.local/bin"
INSTALL_PATH="$INSTALL_DIR/titlename"
CONFIG_DIR="${HOME}/.config/cliTitleName"
FUNCTION_PATH="$CONFIG_DIR/titlename.zsh"
ZSHRC_PATH="${HOME}/.zshrc"
START_MARK='# >>> cliTitleName >>>'
END_MARK='# <<< cliTitleName <<<'
SOURCE_LINE='[[ -r "$HOME/.config/cliTitleName/titlename.zsh" ]] && source "$HOME/.config/cliTitleName/titlename.zsh"'

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"

cat >"$INSTALL_PATH" <<'EOF'
#!/usr/bin/env bash

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
  printf '%s\n' 'usage: titlename "Window Title"' >&2
  exit 1
fi

printf '\033]2;%s\a' "$1"
EOF
chmod +x "$INSTALL_PATH"

cat >"$FUNCTION_PATH" <<'EOF'
# shellcheck shell=zsh

_titlename_disable_ghostty_title_updates() {
  emulate -L zsh

  local prompt_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${(%):-%(4~|…/%3~|%~)}"$\'\\a\''
  local preexec_title_line=$'          builtin print -rnu $_ghostty_fd $\'\\e]2;\'"${1//[[:cntrl:]]}"$\'\\a\''

  if (( $+functions[_ghostty_precmd] )); then
    functions[_ghostty_precmd]=${functions[_ghostty_precmd]//$'\n'"$prompt_title_line"/}
  fi

  if (( $+functions[_ghostty_preexec] )); then
    functions[_ghostty_preexec]=${functions[_ghostty_preexec]//$'\n'"$preexec_title_line"/}
  fi
}

titlename() {
  emulate -L zsh

  if [[ "$(uname -s)" != "Darwin" ]]; then
    print -u2 -- 'titlename supports macOS only'
    return 1
  fi

  if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
    print -u2 -- 'usage: titlename "Window Title"'
    return 1
  fi

  printf '\033]2;%s\a' "$1"

  if [[ "${TERM_PROGRAM:-}" == 'ghostty' ]]; then
    _titlename_disable_ghostty_title_updates
  fi
}
EOF

touch "$ZSHRC_PATH"
if ! grep -Fqx "$START_MARK" "$ZSHRC_PATH" 2>/dev/null; then
  {
    printf '\n%s\n' "$START_MARK"
    printf '%s\n' "$SOURCE_LINE"
    printf '%s\n' "$END_MARK"
  } >>"$ZSHRC_PATH"
fi

printf 'Installed titlename to %s\n' "$INSTALL_PATH"
printf 'Installed zsh integration to %s\n' "$FUNCTION_PATH"
printf 'Loaded titlename from %s\n' "$ZSHRC_PATH"
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: PASS with output `ok`.

- [ ] **Step 5: Commit**

```bash
git add tools/cliTitleName/install.sh tools/cliTitleName/test_cliTitleName.zsh
git commit -m "feat: install titlename zsh integration"
```

### Task 3: Update the English and Chinese documentation

**Files:**
- Modify: `tools/cliTitleName/README.md`
- Modify: `tools/cliTitleName/README_CN.md`
- Modify: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `tools/cliTitleName/test_cliTitleName.zsh`

- [ ] **Step 1: Write the failing documentation assertions**

Append these assertions near the end of `tools/cliTitleName/test_cliTitleName.zsh`, just before the final `print 'ok'`:

```zsh
assert_true '[[ "$(<"$SCRIPT_DIR/README.md")" == *"Ghostty + interactive zsh"* ]]' 'README.md should document the Ghostty + interactive zsh scope'
assert_true '[[ "$(<"$SCRIPT_DIR/README.md")" == *"disables later title rewrites for the current shell session"* ]]' 'README.md should document session-scoped title disabling'
assert_true '[[ "$(<"$SCRIPT_DIR/README.md")" == *"does not change Ghostty globally"* ]]' 'README.md should document that the change is not global'
assert_true '[[ "$(<"$SCRIPT_DIR/README_CN.md")" == *"Ghostty + 交互式 zsh"* ]]' 'README_CN.md should document the Ghostty + 交互式 zsh scope'
assert_true '[[ "$(<"$SCRIPT_DIR/README_CN.md")" == *"禁用当前 shell 会话后续的标题改写"* ]]' 'README_CN.md should document session-scoped title disabling'
assert_true '[[ "$(<"$SCRIPT_DIR/README_CN.md")" == *"不会修改 Ghostty 的全局配置"* ]]' 'README_CN.md should document that the change is not global'
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: FAIL on the first new README assertion because the current docs still describe one-shot behavior only.

- [ ] **Step 3: Write minimal documentation updates**

Replace `tools/cliTitleName/README.md` with:

```md
# cliTitleName

cliTitleName is a tiny macOS-only utility with one command: `titlename`.

## macOS only

This tool is supported only on macOS.

## Install

```sh
curl -fsSL xxx | bash
```

After installation, interactive zsh shells load `titlename` as a shell function from `~/.config/cliTitleName/titlename.zsh`. The executable at `~/.local/bin/titlename` remains available as the fallback outside that shell-function context.

## Usage

```sh
titlename "My Window"
```

## What it does

- It sets the current terminal window title immediately.
- In Ghostty + interactive zsh, it also disables later title rewrites for the current shell session.
- The disable behavior is session-scoped: opening a new shell restores Ghostty's normal title automation.

## What it does not do

- It does not change Ghostty globally.
- It does not modify Ghostty config files.
- It does not promise session-level title disabling outside Ghostty + interactive zsh.
- Outside Ghostty + interactive zsh, it behaves like a one-shot title setter only.
- It does not support non-macOS systems.

## Files

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`
```

Replace `tools/cliTitleName/README_CN.md` with:

```md
# cliTitleName

cliTitleName 是一个极简的、仅支持 macOS 的工具，只有一个命令 `titlename`。

## 仅支持 macOS

这个工具只支持 macOS。

## 安装

```sh
curl -fsSL xxx | bash
```

安装后，交互式 zsh 会从 `~/.config/cliTitleName/titlename.zsh` 加载 `titlename` shell function。`~/.local/bin/titlename` 这个可执行文件仍然会保留，用作非 shell-function 场景下的回退入口。

## 使用方式

```sh
titlename "My Window"
```

## 它会做什么

- 它会立即设置当前终端窗口标题。
- 在 Ghostty + 交互式 zsh 中，它还会禁用当前 shell 会话后续的标题改写。
- 这个禁用行为只作用于当前会话；新开一个 shell 后，Ghostty 的默认标题自动更新会恢复。

## 它不会做什么

- 它不会修改 Ghostty 的全局配置。
- 它不会改写 Ghostty 配置文件。
- 它不承诺在 Ghostty + 交互式 zsh 之外提供会话级标题禁用能力。
- 在 Ghostty + 交互式 zsh 之外，它仍然只是一次性设置标题。
- 它不支持非 macOS 系统。

## 文件

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`
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
git commit -m "docs: describe session-scoped title disabling"
```

### Task 4: Run final automated and manual verification

**Files:**
- Test: `tools/cliTitleName/test_cliTitleName.zsh`
- Test: `docs/superpowers/specs/2026-04-27-cli-title-name-session-title-disable-design.md`

- [ ] **Step 1: Run the full automated regression suite**

Run:

```bash
zsh tools/cliTitleName/test_cliTitleName.zsh
```

Expected: PASS with output `ok`.

- [ ] **Step 2: Re-read the spec and verify coverage**

Check that the implementation satisfies:

- the single `titlename` command surface
- session-scoped disabling only
- no global Ghostty config edits
- Ghostty + interactive zsh as the only guaranteed environment
- preserved one-shot fallback outside the shell-function context

- [ ] **Step 3: Perform manual Ghostty verification**

Run the installer in a fresh shell:

```bash
bash tools/cliTitleName/install.sh
```

Then open a fresh Ghostty zsh session and run:

```zsh
which titlename
whence -f titlename
titlename "命名窗口工具"
print -r -- $functions[_ghostty_precmd]
print -r -- $functions[_ghostty_preexec]
```

Expected:

- `which titlename` still resolves a usable command name
- `whence -f titlename` shows a shell function definition
- the window title changes to `命名窗口工具`
- `_ghostty_precmd` and `_ghostty_preexec` no longer contain `\e]2;`
- subsequent Enter presses and subsequent commands do not overwrite the title
- opening a brand-new shell restores normal Ghostty title automation

- [ ] **Step 4: Commit the final verification-ready state**

```bash
git add tools/cliTitleName/titlename.zsh tools/cliTitleName/install.sh tools/cliTitleName/test_cliTitleName.zsh tools/cliTitleName/README.md tools/cliTitleName/README_CN.md
git commit -m "feat: disable Ghostty title rewrites after titlename"
```

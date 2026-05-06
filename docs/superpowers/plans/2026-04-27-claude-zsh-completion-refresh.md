# Claude Zsh Completion Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the stale static `_claude` installer output with help-derived completions and a Claude-scoped zsh completion block that preserves the user's global completion behavior.

**Architecture:** Keep all behavior inside `tools/zsh-completion/setup_claude_completion.sh`. The script should validate that `claude` exists, capture `claude --help`, parse only the stable top-level `Options:` and `Commands:` sections, and render a generated `_claude` function from those parsed tables. The installer should manage its own marked `~/.zshrc` block that adds `~/.zsh/completions` to `fpath`, runs `compinit` only when needed, and applies a narrow `zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'` without touching any unrelated completion settings.

**Tech Stack:** Bash installer script, zsh completion syntax, Python 3 for fixture extraction/verification helpers in tests, temporary HOME-based shell verification.

---

## File Structure

- `tools/zsh-completion/setup_claude_completion.sh`
  - Replace the static heredoc completion payload with generator logic.
  - Add parsing helpers for `claude --help`.
  - Add managed zshrc block update logic.
  - Add clear failure paths for missing `claude` or parse failures.
- `tools/zsh-completion/test_zsh_completion_installer.py`
  - New regression test file for help parsing, generated `_claude` contents, and managed `~/.zshrc` block behavior.
- `tools/zsh-completion/test_zsh_completion_docs.zsh`
  - Keep the existing README install-snippet assertions.
- `tools/zsh-completion/README.md`
  - Update feature/behavior descriptions so they no longer promise stale hard-coded subcommand detail.
  - Document that completions are generated from the installed Claude CLI.
- `tools/zsh-completion/README_CN.md`
  - Chinese counterpart to the README updates above.
- `tools/zsh-completion/testdata/claude-help.txt`
  - New captured top-level `claude --help` fixture used by parser tests.

---

### Task 1: Add parser fixture and failing generator tests

**Files:**
- Create: `tools/zsh-completion/test_zsh_completion_installer.py`
- Create: `tools/zsh-completion/testdata/claude-help.txt`
- Modify: `tools/zsh-completion/setup_claude_completion.sh:1-390`

- [ ] **Step 1: Create the help fixture**

Write `tools/zsh-completion/testdata/claude-help.txt` with the current top-level CLI help output captured from `claude --help`:

```text
Usage: claude [options] [command] [prompt]

Claude Code - starts an interactive session by default, use -p/--print for
non-interactive output

Arguments:
  prompt                                            Your prompt

Options:
  --add-dir <directories...>                        Additional directories to allow tool access to
  --agent <agent>                                   Agent for the current session. Overrides the 'agent' setting.
  --agents <json>                                   JSON object defining custom agents (e.g. '{"reviewer": {"description": "Reviews code", "prompt": "You are a code reviewer"}}')
  --allow-dangerously-skip-permissions              Enable bypassing all permission checks as an option, without it being enabled by default. Recommended only for sandboxes with no internet access.
  --allowedTools, --allowed-tools <tools...>        Comma or space-separated list of tool names to allow (e.g. "Bash(git *) Edit")
  --append-system-prompt <prompt>                   Append a system prompt to the default system prompt
  --bare                                            Minimal mode: skip hooks, LSP, plugin sync, attribution, auto-memory, background prefetches, keychain reads, and CLAUDE.md auto-discovery. Sets CLAUDE_CODE_SIMPLE=1. Anthropic auth is strictly ANTHROPIC_API_KEY or apiKeyHelper via --settings (OAuth and keychain are never read). 3P providers (Bedrock/Vertex/Foundry) use their own credentials. Skills still resolve via /skill-name. Explicitly provide context via: --system-prompt[-file], --append-system-prompt[-file], --add-dir (CLAUDE.md dirs), --mcp-config, --settings, --agents, --plugin-dir.
  --betas <betas...>                                Beta headers to include in API requests (API key users only)
  --brief                                           Enable SendUserMessage tool for agent-to-user communication
  --chrome                                          Enable Claude in Chrome integration
  -c, --continue                                    Continue the most recent conversation in the current directory
  --dangerously-skip-permissions                    Bypass all permission checks. Recommended only for sandboxes with no internet access.
  -d, --debug [filter]                              Enable debug mode with optional category filtering (e.g., "api,hooks" or "!1p,!file")
  --debug-file <path>                               Write debug logs to a specific file path (implicitly enables debug mode)
  --disable-slash-commands                          Disable all skills
  --disallowedTools, --disallowed-tools <tools...>  Comma or space-separated list of tool names to deny (e.g. "Bash(git *) Edit")
  --effort <level>                                  Effort level for the current session (low, medium, high, xhigh, max)
  --exclude-dynamic-system-prompt-sections          Move per-machine sections (cwd, env info, memory paths, git status) from the system prompt into the first user message. Improves cross-user prompt-cache reuse. Only applies with the default system prompt (ignored with --system-prompt). (default: false)
  --fallback-model <model>                          Enable automatic fallback to specified model when default model is overloaded (only works with --print)
  --file <specs...>                                 File resources to download at startup. Format: file_id:relative_path (e.g., --file file_abc:doc.txt file_def:img.png)
  --fork-session                                    When resuming, create a new session ID instead of reusing the original (use with --resume or --continue)
  --from-pr [value]                                 Resume a session linked to a PR by PR number/URL, or open interactive picker with optional search term
  -h, --help                                        Display help for command
  --ide                                             Automatically connect to IDE on startup if exactly one valid IDE is available
  --include-hook-events                             Include all hook lifecycle events in the output stream (only works with --output-format=stream-json)
  --include-partial-messages                        Include partial message chunks as they arrive (only works with --print and --output-format=stream-json)
  --input-format <format>                           Input format (only works with --print): "text" (default), or "stream-json" (realtime streaming input) (choices: "text", "stream-json")
  --json-schema <schema>                            JSON Schema for structured output validation. Example: {"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}
  --max-budget-usd <amount>                         Maximum dollar amount to spend on API calls (only works with --print)
  --mcp-config <configs...>                         Load MCP servers from JSON files or strings (space-separated)
  --mcp-debug                                       [DEPRECATED. Use --debug instead] Enable MCP debug mode (shows MCP server errors)
  --model <model>                                   Model for the current session. Provide an alias for the latest model (e.g. 'sonnet' or 'opus') or a model's full name (e.g. 'claude-sonnet-4-6').
  -n, --name <name>                                 Set a display name for this session (shown in the prompt box, /resume picker, and terminal title)
  --no-chrome                                       Disable Claude in Chrome integration
  --no-session-persistence                          Disable session persistence - sessions will not be saved to disk and cannot be resumed (only works with --print)
  --output-format <format>                          Output format (only works with --print): "text" (default), "json" (single result), or "stream-json" (realtime streaming) (choices: "text", "json", "stream-json")
  --permission-mode <mode>                          Permission mode to use for the session (choices: "acceptEdits", "auto", "bypassPermissions", "default", "dontAsk", "plan")
  --plugin-dir <path>                               Load plugins from a directory for this session only (repeatable: --plugin-dir A --plugin-dir B) (default: [])
  -p, --print                                       Print response and exit (useful for pipes). Note: The workspace trust dialog is skipped when Claude is run with the -p mode. Only use this flag in directories you trust.
  --remote-control-session-name-prefix <prefix>     Prefix for auto-generated Remote Control session names (default: hostname)
  --replay-user-messages                            Re-emit user messages from stdin back on stdout for acknowledgment (only works with --input-format=stream-json and --output-format=stream-json)
  -r, --resume [value]                              Resume a conversation by session ID, or open interactive picker with optional search term
  --session-id <uuid>                               Use a specific session ID for the conversation (must be a valid UUID)
  --setting-sources <sources>                       Comma-separated list of setting sources to load (user, project, local).
  --settings <file-or-json>                         Path to a settings JSON file or a JSON string to load additional settings from
  --strict-mcp-config                               Only use MCP servers from --mcp-config, ignoring all other MCP configurations
  --system-prompt <prompt>                          System prompt to use for the session
  --tmux                                            Create a tmux session for the worktree (requires --worktree). Uses iTerm2 native panes when available; use --tmux=classic for traditional tmux.
  --tools <tools...>                                Specify the list of available tools from the built-in set. Use "" to disable all tools, "default" to use all tools, or specify tool names (e.g. "Bash,Edit,Read").
  --verbose                                         Override verbose mode setting from config
  -v, --version                                     Output the version number
  -w, --worktree [name]                             Create a new git worktree for this session (optionally specify a name)

Commands:
  agents [options]                                  List configured agents
  auth                                              Manage authentication
  auto-mode                                         Inspect auto mode classifier configuration
  doctor                                            Check the health of your Claude Code auto-updater. Note: The workspace trust dialog is skipped and stdio servers from .mcp.json are spawned for health checks. Only use this command in directories you trust.
  install [options] [target]                        Install Claude Code native build. Use [target] to specify version (stable, latest, or specific version)
  mcp                                               Configure and manage MCP servers
  plugin|plugins                                    Manage Claude Code plugins
  setup-token                                       Set up a long-lived authentication token (requires Claude subscription)
  update|upgrade                                    Check for updates and install if available
```

- [ ] **Step 2: Write failing parser and installer tests**

Create `tools/zsh-completion/test_zsh_completion_installer.py`:

```python
from __future__ import annotations

import os
import pathlib
import shutil
import subprocess
import tempfile
import textwrap
import unittest

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
SETUP_SCRIPT = SCRIPT_DIR / "setup_claude_completion.sh"
HELP_FIXTURE = SCRIPT_DIR / "testdata" / "claude-help.txt"


class ZshCompletionInstallerTests(unittest.TestCase):
    maxDiff = None

    def run_installer(self, home: pathlib.Path) -> subprocess.CompletedProcess[str]:
        fake_bin = home / "bin"
        fake_bin.mkdir(parents=True, exist_ok=True)
        claude_path = fake_bin / "claude"
        claude_path.write_text(
            textwrap.dedent(
                f"""\
                #!/bin/sh
                if [ "$1" = "--help" ]; then
                  cat <<'EOF'
                {HELP_FIXTURE.read_text()}
                EOF
                  exit 0
                fi
                echo "unexpected args: $*" >&2
                exit 1
                """
            ),
            encoding="utf-8",
        )
        claude_path.chmod(0o755)

        env = os.environ.copy()
        env["HOME"] = str(home)
        env["PATH"] = f"{fake_bin}:{env['PATH']}"

        return subprocess.run(
            ["bash", str(SETUP_SCRIPT)],
            cwd=SCRIPT_DIR,
            env=env,
            text=True,
            capture_output=True,
        )

    def test_installer_generates_completion_from_help_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            completion = (home / ".zsh" / "completions" / "_claude").read_text(encoding="utf-8")
            self.assertIn("--bare", completion)
            self.assertIn("--name", completion)
            self.assertIn("xhigh", completion)
            self.assertIn('acceptEdits auto bypassPermissions default dontAsk plan', completion)
            self.assertNotIn('acceptEdits bypassPermissions default dontAsk plan auto', completion)

    def test_installer_writes_claude_scoped_zshrc_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            zshrc = home / ".zshrc"
            zshrc.write_text(
                textwrap.dedent(
                    """\
                    # OPENSPEC:START
                    fpath=("/Users/example/.zsh/completions" $fpath)
                    zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'
                    autoload -Uz compinit
                    compinit
                    # OPENSPEC:END
                    """
                ),
                encoding="utf-8",
            )

            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            updated = zshrc.read_text(encoding="utf-8")
            self.assertIn("# >>> Claude CLI zsh completion >>>", updated)
            self.assertIn("zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'", updated)
            self.assertIn("fpath=(\"$HOME/.zsh/completions\" $fpath)", updated)
            self.assertIn("# OPENSPEC:START", updated)
            self.assertIn("zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'", updated)
            self.assertEqual(updated.count("# >>> Claude CLI zsh completion >>>"), 1)

    def test_installer_fails_when_claude_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            env = os.environ.copy()
            env["HOME"] = str(home)
            env["PATH"] = "/usr/bin:/bin"

            result = subprocess.run(
                ["bash", str(SETUP_SCRIPT)],
                cwd=SCRIPT_DIR,
                env=env,
                text=True,
                capture_output=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("claude was not found in PATH", result.stderr)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 3: Run the new test file and confirm it fails**

Run:

```bash
python3 tools/zsh-completion/test_zsh_completion_installer.py
```

Expected: FAIL because `setup_claude_completion.sh` still writes the old static heredoc and does not emit the managed Claude-scoped zshrc block.

- [ ] **Step 4: Commit the failing tests and fixture**

```bash
git add tools/zsh-completion/test_zsh_completion_installer.py tools/zsh-completion/testdata/claude-help.txt
git commit -m "test: add zsh completion installer coverage"
```

---

### Task 2: Replace the static completion heredoc with help-derived generation

**Files:**
- Modify: `tools/zsh-completion/setup_claude_completion.sh:1-390`
- Test: `tools/zsh-completion/test_zsh_completion_installer.py`

- [ ] **Step 1: Replace the script body with generator-friendly structure**

Rewrite `tools/zsh-completion/setup_claude_completion.sh` to this shape:

```bash
#!/usr/bin/env bash

set -euo pipefail

COMPLETIONS_DIR="$HOME/.zsh/completions"
COMPLETION_FILE="$COMPLETIONS_DIR/_claude"
ZSHRC_PATH="$HOME/.zshrc"
BLOCK_START='# >>> Claude CLI zsh completion >>>'
BLOCK_END='# <<< Claude CLI zsh completion <<<'

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

require_claude() {
  command -v claude >/dev/null 2>&1 || fail 'claude was not found in PATH'
}

capture_help() {
  claude --help || fail 'failed to run claude --help'
}

ensure_completions_dir() {
  mkdir -p "$COMPLETIONS_DIR"
  [[ -w "$COMPLETIONS_DIR" ]] || fail "directory is not writable: $COMPLETIONS_DIR"
}
```

- [ ] **Step 2: Add help parsing and zsh escaping helpers**

Append these functions inside `tools/zsh-completion/setup_claude_completion.sh`:

```bash
escape_zsh_text() {
  python3 - <<'PY' "$1"
import sys
print(sys.argv[1].replace('\\', '\\\\').replace("'", "'\\''"))
PY
}

parse_help_to_json() {
  python3 - <<'PY'
import json
import re
import sys

help_text = sys.stdin.read().splitlines()
section = None
options = []
commands = []

def parse_option_names(raw):
    flag_part = re.split(r'\s{2,}', raw, maxsplit=1)[0]
    return [part.strip() for part in flag_part.split(', ')]

for line in help_text:
    if line == 'Options:':
        section = 'options'
        continue
    if line == 'Commands:':
        section = 'commands'
        continue
    if not line.strip():
        continue
    if section == 'options' and line.startswith('  -'):
        raw, description = re.split(r'\s{2,}', line.strip(), maxsplit=1)
        options.append({
            'raw': raw,
            'names': parse_option_names(raw),
            'description': description,
        })
    elif section == 'commands' and line.startswith('  '):
        raw, description = re.split(r'\s{2,}', line.strip(), maxsplit=1)
        commands.append({
            'raw': raw,
            'name': raw.split()[0],
            'description': description,
        })

if not options or not commands:
    raise SystemExit('failed to parse enough data from claude --help')

print(json.dumps({'options': options, 'commands': commands}))
PY
}
```

- [ ] **Step 3: Add enum extraction and completion rendering**

Append these functions inside `tools/zsh-completion/setup_claude_completion.sh`:

```bash
option_choices() {
  python3 - <<'PY' "$1" "$2"
import re
import sys
raw = sys.argv[1]
description = sys.argv[2]
if '<mode>' in raw:
    match = re.search(r'choices:\s+(.+)', description)
    if match:
        values = re.findall(r'"([^"]+)"', match.group(1))
        if values:
            print(' '.join(values))
elif '<level>' in raw:
    match = re.search(r'\(([^)]*)\)', description)
    if match:
        print(' '.join(part.strip() for part in match.group(1).split(',')))
elif '<format>' in raw:
    match = re.search(r'choices:\s+(.+)', description)
    if match:
        values = re.findall(r'"([^"]+)"', match.group(1))
        if values:
            print(' '.join(values))
PY
}

render_option_spec() {
  python3 - <<'PY' "$1"
import json
import re
import sys
option = json.loads(sys.argv[1])
raw = option['raw']
description = option['description'].replace("'", "'\\''")
names = option['names']
argument = None
match = re.search(r'( <[^>]+>| \[[^]]+\])$', raw)
if match:
    argument = match.group(1).strip()
    raw = raw[: -len(match.group(1))]
    names = [part.strip() for part in raw.split(', ')]

if len(names) == 2 and names[0].startswith('-') and names[1].startswith('--'):
    prefix = f"'({names[0]} {names[1]})'{{{names[0]},{names[1]}}}"
else:
    prefix = ' '.join(f"'{name}'" for name in names)

if argument == '<mode>':
    print(f"{prefix}[{description}]:mode:({option['choices']})")
elif argument == '<level>':
    print(f"{prefix}[{description}]:level:({option['choices']})")
elif argument == '<format>':
    print(f"{prefix}[{description}]:format:({option['choices']})")
elif argument and 'dir' in argument.lower():
    print(f"{prefix}[{description}]:directory:_directories -/")
elif argument:
    label = argument.strip('<>[]').split(':')[0]
    print(f"{prefix}[{description}]:{label}:")
else:
    print(f"{prefix}[{description}]")
PY
}
```

- [ ] **Step 4: Generate `_claude` from parsed help instead of static text**

Replace the heredoc-writing section with this flow:

```bash
generate_completion_file() {
  local help_text parsed_json options_json commands_json
  local -a option_specs command_specs
  local option_json command_json raw description choices spec

  help_text="$(capture_help)"
  parsed_json="$(printf '%s\n' "$help_text" | parse_help_to_json)" || fail 'failed to parse claude --help'
  options_json="$(python3 - <<'PY' "$parsed_json"
import json
import sys
for item in json.loads(sys.argv[1])['options']:
    print(json.dumps(item))
PY
)"
  commands_json="$(python3 - <<'PY' "$parsed_json"
import json
import sys
for item in json.loads(sys.argv[1])['commands']:
    print(json.dumps(item))
PY
)"

  while IFS= read -r option_json; do
    [[ -n "$option_json" ]] || continue
    raw="$(python3 - <<'PY' "$option_json"
import json
import sys
print(json.loads(sys.argv[1])['raw'])
PY
)"
    description="$(python3 - <<'PY' "$option_json"
import json
import sys
print(json.loads(sys.argv[1])['description'])
PY
)"
    choices="$(option_choices "$raw" "$description")"
    if [[ -n "$choices" ]]; then
      option_json="$(python3 - <<'PY' "$option_json" "$choices"
import json
import sys
item = json.loads(sys.argv[1])
item['choices'] = sys.argv[2]
print(json.dumps(item))
PY
)"
    fi
    spec="$(render_option_spec "$option_json")"
    option_specs+=("$spec")
  done <<< "$options_json"

  while IFS= read -r command_json; do
    [[ -n "$command_json" ]] || continue
    command_specs+=("$(python3 - <<'PY' "$command_json"
import json
import sys
item = json.loads(sys.argv[1])
name = item['name'].replace("'", "'\\''")
description = item['description'].replace("'", "'\\''")
print(f"'{name}:{description}'")
PY
)")
  done <<< "$commands_json"

  cat > "$COMPLETION_FILE" <<EOF
#compdef claude

_claude() {
  local curcontext="\$curcontext" state line
  typeset -A opt_args
  local -a global_options commands

  global_options=(
$(printf '    %s\n' "${option_specs[@]}")
  )

  commands=(
$(printf '    %s\n' "${command_specs[@]}")
  )

  _arguments -C \
    \$global_options \
    ': :->command' \
    '*:: :->args'

  case "\$state" in
    command)
      _describe -t commands 'claude commands' commands
      ;;
  esac
}
EOF
}
```

- [ ] **Step 5: Run the installer test file and verify it passes**

Run:

```bash
python3 tools/zsh-completion/test_zsh_completion_installer.py
```

Expected: PASS with `Ran 3 tests` and `OK`.

- [ ] **Step 6: Commit the generator rewrite**

```bash
git add tools/zsh-completion/setup_claude_completion.sh tools/zsh-completion/test_zsh_completion_installer.py
git commit -m "feat: generate claude completions from local cli help"
```

---

### Task 3: Manage a Claude-scoped zshrc block without mutating global completion styles

**Files:**
- Modify: `tools/zsh-completion/setup_claude_completion.sh:1-390`
- Test: `tools/zsh-completion/test_zsh_completion_installer.py`

- [ ] **Step 1: Add managed block rendering helpers**

Add these functions to `tools/zsh-completion/setup_claude_completion.sh`:

```bash
completion_block() {
  cat <<'EOF'
# >>> Claude CLI zsh completion >>>
fpath=("$HOME/.zsh/completions" $fpath)
if ! whence -w compinit >/dev/null 2>&1; then
  autoload -Uz compinit
fi
if [[ -z ${_CLAUDE_COMPLETION_COMPINIT_DONE:-} ]]; then
  compinit
  _CLAUDE_COMPLETION_COMPINIT_DONE=1
fi
zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'
# <<< Claude CLI zsh completion <<<
EOF
}

update_zshrc_block() {
  local block
  block="$(completion_block)"

  if [[ -f "$ZSHRC_PATH" ]]; then
    python3 - <<'PY' "$ZSHRC_PATH" "$BLOCK_START" "$BLOCK_END" "$block"
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
start = sys.argv[2]
end = sys.argv[3]
block = sys.argv[4]
text = path.read_text(encoding='utf-8')
if start in text and end in text:
    before, rest = text.split(start, 1)
    _, after = rest.split(end, 1)
    new_text = before.rstrip() + '\n\n' + block + '\n' + after.lstrip('\n')
else:
    new_text = text.rstrip() + '\n\n' + block + '\n'
path.write_text(new_text, encoding='utf-8')
PY
  else
    printf '%s\n' "$block" > "$ZSHRC_PATH"
  fi
}
```

- [ ] **Step 2: Make the existing zshrc-block test fail against the old behavior**

Run only the block-management test:

```bash
python3 -m unittest tools.zsh-completion.test_zsh_completion_installer.ZshCompletionInstallerTests.test_installer_writes_claude_scoped_zshrc_block
```

Expected: PASS if Task 2 already landed; otherwise update the module path to the actual test runner invocation and verify the test exercises the new managed block. The assertion must confirm that the OpenSpec block remains untouched and that only one Claude-managed block exists.

- [ ] **Step 3: Wire the installer main flow to update only the managed block**

Set the script entrypoint to:

```bash
main() {
  require_claude
  ensure_completions_dir
  generate_completion_file
  update_zshrc_block
  printf '%s\n' 'Claude completion installation complete.'
  printf '%s\n' 'Run: source ~/.zshrc'
}

main "$@"
```

Remove the old `grep`-based `.zshrc` append logic completely.

- [ ] **Step 4: Re-run the installer tests**

Run:

```bash
python3 tools/zsh-completion/test_zsh_completion_installer.py
```

Expected: PASS with the same `Ran 3 tests` and `OK` output.

- [ ] **Step 5: Commit the zshrc block management change**

```bash
git add tools/zsh-completion/setup_claude_completion.sh tools/zsh-completion/test_zsh_completion_installer.py
git commit -m "fix: scope claude completion zstyle to claude only"
```

---

### Task 4: Verify real installer behavior in a temporary HOME and document the new behavior

**Files:**
- Modify: `tools/zsh-completion/README.md:1-82`
- Modify: `tools/zsh-completion/README_CN.md:1-82`
- Test: `tools/zsh-completion/test_zsh_completion_docs.zsh`
- Test: `tools/zsh-completion/test_zsh_completion_installer.py`

- [ ] **Step 1: Update the English README**

Change the feature and script-behavior sections in `tools/zsh-completion/README.md` to:

```md
## Features

- **Help-derived completions**: Regenerates top-level `claude` options and commands from the currently installed Claude CLI
- **Current enum values**: Picks up visible values such as `--permission-mode` and `--effort` directly from `claude --help`
- **Claude-only matcher style**: Adds a conservative `:completion:*:*:claude:*` matcher-list without changing your global completion rules
- **Idempotent install**: Re-runs cleanly by replacing only the installer-managed block in `~/.zshrc`

## What the Script Does

1. Verifies that `claude` is available in `PATH`
2. Captures `claude --help` and generates `~/.zsh/completions/_claude`
3. Updates only its own marked block in `~/.zshrc`
```
```

- [ ] **Step 2: Update the Chinese README**

Change the corresponding sections in `tools/zsh-completion/README_CN.md` to:

```md
## 功能特性

- **基于帮助输出生成补齐**：根据当前已安装的 Claude CLI 的 `claude --help` 重新生成顶层选项和命令补齐
- **枚举值与当前 CLI 同步**：直接从 `claude --help` 提取 `--permission-mode`、`--effort` 等可见取值
- **仅作用于 Claude 的 matcher style**：只添加保守的 `:completion:*:*:claude:*` matcher-list，不改动全局补齐规则
- **可重复安装**：重复运行时只替换安装器自己管理的 `~/.zshrc` 标记块

## 脚本功能说明

1. 检查 `PATH` 中是否存在 `claude`
2. 读取 `claude --help` 并生成 `~/.zsh/completions/_claude`
3. 只更新自己管理的 `~/.zshrc` 标记块
```
```

- [ ] **Step 3: Run the documentation regression test**

Run:

```bash
zsh tools/zsh-completion/test_zsh_completion_docs.zsh
```

Expected: `ok`

- [ ] **Step 4: Run a real installer verification in a temporary HOME**

Run:

```bash
tmp_home=$(mktemp -d) && HOME="$tmp_home" bash < tools/zsh-completion/setup_claude_completion.sh && python3 - <<'PY' "$tmp_home"
import pathlib
import sys
home = pathlib.Path(sys.argv[1])
completion = (home / '.zsh' / 'completions' / '_claude').read_text(encoding='utf-8')
zshrc = (home / '.zshrc').read_text(encoding='utf-8')
assert '--bare' in completion
assert '--name' in completion
assert 'xhigh' in completion
assert "zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'" in zshrc
assert "zstyle ':completion:*' matcher-list" not in zshrc
print('installer verification ok')
PY
```

Expected: `installer verification ok`

- [ ] **Step 5: Run an interactive zsh lookup verification**

Run:

```bash
tmp_home=$(mktemp -d) && HOME="$tmp_home" bash < tools/zsh-completion/setup_claude_completion.sh >/dev/null && HOME="$tmp_home" zsh -ic 'autoload -Uz compinit; compinit; print $_comps[claude]; grep -n -- "--permission-mode" ~/.zsh/completions/_claude; grep -n -- "xhigh" ~/.zsh/completions/_claude'
```

Expected:
- first printed line is `_claude`
- grep output shows a generated `--permission-mode` entry
- grep output shows `xhigh` present in the generated file

- [ ] **Step 6: Commit the docs and verification adjustments**

```bash
git add tools/zsh-completion/README.md tools/zsh-completion/README_CN.md tools/zsh-completion/test_zsh_completion_docs.zsh tools/zsh-completion/test_zsh_completion_installer.py tools/zsh-completion/setup_claude_completion.sh
git commit -m "docs: describe generated claude zsh completions"
```

---

### Task 5: Final regression sweep and branch verification

**Files:**
- Modify: `tools/zsh-completion/setup_claude_completion.sh`
- Modify: `tools/zsh-completion/test_zsh_completion_installer.py`
- Modify: `tools/zsh-completion/README.md`
- Modify: `tools/zsh-completion/README_CN.md`
- Test: `tools/zsh-completion/test_zsh_completion_docs.zsh`

- [ ] **Step 1: Run the full targeted verification suite**

Run:

```bash
python3 tools/zsh-completion/test_zsh_completion_installer.py && zsh tools/zsh-completion/test_zsh_completion_docs.zsh
```

Expected:
- Python suite prints `Ran 3 tests` and `OK`
- zsh doc suite prints `ok`

- [ ] **Step 2: Re-read the approved spec and verify coverage**

Check each requirement in `docs/superpowers/specs/2026-04-27-claude-zsh-completion-refresh-design.md` against the implementation:

- generated from `claude --help`
- no stale static option list as source of truth
- Claude-scoped zstyle only
- managed `~/.zshrc` block only
- clear failure when `claude` missing or parse fails
- temporary-HOME verification path

Record any gaps and fix them before proceeding.

- [ ] **Step 3: Run git diff for final review**

Run:

```bash
git diff -- tools/zsh-completion/setup_claude_completion.sh tools/zsh-completion/test_zsh_completion_installer.py tools/zsh-completion/testdata/claude-help.txt tools/zsh-completion/README.md tools/zsh-completion/README_CN.md tools/zsh-completion/test_zsh_completion_docs.zsh docs/superpowers/plans/2026-04-27-claude-zsh-completion-refresh.md
```

Expected: Diff only shows the intended installer, test, fixture, README, and plan changes.

- [ ] **Step 4: Commit the final regression sweep if fixes were needed**

If Step 2 or Step 3 required code changes:

```bash
git add tools/zsh-completion/setup_claude_completion.sh tools/zsh-completion/test_zsh_completion_installer.py tools/zsh-completion/testdata/claude-help.txt tools/zsh-completion/README.md tools/zsh-completion/README_CN.md tools/zsh-completion/test_zsh_completion_docs.zsh
git commit -m "test: finalize zsh completion refresh verification"
```

If no fixes were needed, skip this commit.

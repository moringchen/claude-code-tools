# Claude Zsh Completion Refresh Design

**Goal:** Make the `zsh-completion` installer generate a `_claude` completion file that matches the currently installed Claude CLI, and isolate Claude-specific completion styling so the user's global matcher settings do not distort `claude` option insertion behavior.

**Scope:** The `tools/zsh-completion/` tool only. This design covers installer behavior, generated completion content, and Claude-scoped zstyle management. It does not change the `cliTitleName` tool or unrelated shell configuration.

## Current Problem

The current installer writes a fixed `_claude` script from a large heredoc embedded in `tools/zsh-completion/setup_claude_completion.sh`. That static file has already drifted from the actual CLI surface.

Observed evidence:

- the installed completion file at `~/.zsh/completions/_claude` is the active completion source
- its `--permission-mode` choices are hard-coded
- it is missing newer flags such as `--bare` and `--name`
- its `--effort` values are stale

As a result, reinstalling with the published curl command reintroduces stale completions.

Separately, the user reports odd cursor placement around hyphenated options. The strongest current suspect is that the user's global completion matcher settings are too aggressive for `claude` option insertion. That should be handled without mutating the user's general completion behavior for unrelated commands.

## User-Approved Direction

1. The installer should generate `_claude` based on the currently installed `claude --help`
2. The installer should not rely on a stale static option list
3. The fix for cursor behavior should not change the user's global completion style
4. Any completion-style adjustment should be limited to Claude-specific contexts only

## Design

### Completion Source of Truth

The source of truth becomes the locally installed Claude CLI at install time.

The installer should:

1. verify that `claude` is available in `PATH`
2. run `claude --help`
3. extract the current top-level options and commands from the help text
4. write a generated `_claude` file to `~/.zsh/completions/_claude`

This means a re-run of the installer refreshes completions to match the user's actual CLI version instead of the repository author's last manually curated snapshot.

### Generation Strategy

The generated completion file should still be a normal zsh completion function, but its data tables should be derived at install time rather than copied from a checked-in heredoc.

The installer may keep a small amount of stable generator logic in the repository, for example:

- parser logic for `claude --help`
- helper functions for shell-escaping descriptions
- generation of `_arguments` blocks for options and commands

But the actual option inventory and command inventory should come from the local CLI output.

### Scope of Generated Coverage

This task guarantees accurate coverage for the top-level `claude` command surface that is directly visible in `claude --help`, including:

- top-level long and short options
- visible commands
- visible enumerated values when they appear explicitly in help text, such as `--permission-mode` and `--effort`

Subcommand detail may still be generated conservatively if the local CLI does not expose a structured machine-readable completion source. In that case, top-level correctness is prioritized over speculative deep subcommand modeling.

### Claude-Scoped Completion Style

The installer should manage a dedicated `~/.zshrc` block for Claude completion setup.

That block should:

- add `~/.zsh/completions` to `fpath` if needed
- initialize completion safely
- set a Claude-specific zstyle under a narrow scope such as `:completion:*:*:claude:*`

The Claude-specific style should be conservative and avoid the looser matcher patterns suspected of causing the `-光标-` insertion issue. It should not rewrite the user's global `zstyle ':completion:*' ...` rules.

### zshrc Management

The installer should manage only its own marked block so repeated installs remain idempotent.

It should not:

- delete unrelated completion settings
- rewrite the OpenSpec block or other existing initialization blocks
- globally relax or tighten the user's completion policy

### Failure Behavior

If `claude` is not installed or `claude --help` cannot be parsed well enough to generate a valid completion file, the installer should fail clearly rather than silently writing stale or partial data.

The failure message should tell the user what prerequisite is missing or what parse step failed.

## Constraints

- No hard-coded canonical option snapshot as the primary completion source
- No global mutation of the user's existing `zstyle ':completion:*'` rules
- No unrelated shell cleanup in `~/.zshrc`
- No dependency on undocumented Claude internals beyond visible CLI help output

## Verification Plan

1. Run the installer in a temporary `HOME` with the local `claude` binary available
2. Confirm it creates `~/.zsh/completions/_claude`
3. Confirm the generated file contains current help-derived entries such as `--bare`, `--name`, current `--permission-mode` values, and current `--effort` values
4. Confirm the managed zshrc block uses a Claude-scoped zstyle rather than altering global matcher rules
5. Load a fresh interactive zsh session and verify `_claude` resolves from `~/.zsh/completions/_claude`
6. Verify `claude --permission-mode <Tab>` offers current values
7. Verify the cursor/insertion behavior no longer exhibits the reported hyphen-placement issue in the Claude completion context

## Risks and Mitigations

### Risk: help-text parsing is brittle

Mitigation: keep the parser deliberately narrow, tied to stable `Usage`, `Options`, and `Commands` sections, and add regression tests around real captured help text.

### Risk: top-level completion becomes accurate but subcommand completion becomes shallower

Mitigation: prioritize correctness over stale overreach. It is better to have accurate top-level completion than incorrect deep completion.

### Risk: Claude-scoped style does not fully fix cursor behavior

Mitigation: keep the style isolated so behavior can be tuned further without affecting unrelated commands or the user's global setup.

## Out of Scope

- changing Claude CLI itself
- adding completion support for shells other than zsh
- rewriting the user's broader completion framework
- introducing a network dependency to fetch completion metadata

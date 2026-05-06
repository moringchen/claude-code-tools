# cliTitleName README Consolidation Design

**Goal:** Consolidate terminal-title documentation and install instructions around `tools/cliTitleName/`, remove the superseded `current-window-title-lock` tool directory, and make quick-install commands consistent across the repo.

**Scope:** Documentation and repository structure only. No new user-facing commands. The title tool remains `titlename`, and the zsh completion tool remains the Claude CLI completion installer.

## Current Problem

The repository currently presents two overlapping terminal-title tools:

1. `tools/current-window-title-lock/` documents `lock-title`, `unlock-title`, and `title-status`
2. `tools/cliTitleName/` documents the newer `titlename` command

This creates conflicting guidance in the top-level `README.md` and `README_CN.md`, and the project structure still advertises the old directory even though the desired product direction is to keep the title functionality under `cliTitleName` only.

The install instructions are also inconsistent:

- some docs still use local repository commands instead of a curl installer
- some quick-install sections omit `source ~/.zshrc`
- the `cliTitleName` docs need to use the GitHub raw install path consistently

## User-Approved Direction

1. Replace the top-level “Current Window Title Lock / 当前窗口标题锁定” entries with `cliTitleName / titlename`
2. Do not keep a migration note for the old tool in the public README files
3. Standardize quick-install examples to `curl -fsSL ... | bash`
4. Include `source ~/.zshrc` in quick-install examples where immediate shell availability matters
5. Remove `tools/current-window-title-lock/` from the repository
6. Correct `cliTitleName` install URLs to use the GitHub raw path

## Design

### Public Tool Surface

The repository should present only one terminal-title tool:

- directory: `tools/cliTitleName/`
- command: `titlename`

The old `current-window-title-lock` tool is superseded and should disappear from:

- the top-level English README
- the top-level Chinese README
- the project structure diagrams
- the repository tree itself

### Top-Level README Updates

Both `README.md` and `README_CN.md` should describe the title tool in terms of `cliTitleName`.

Required changes:

- change the tool name and link from `tools/current-window-title-lock/` to `tools/cliTitleName/`
- replace the old multi-command lock/unlock/status description with a concise `titlename` description
- update the quick-install example to the `curl -fsSL ... | bash` form
- add `source ~/.zshrc` immediately after the install command in the quick-install example
- remove `current-window-title-lock/` from the project structure section
- add `cliTitleName/` into the project structure section

### Tool README Install Consistency

The quick-install sections for the relevant tools should use one consistent pattern.

For `tools/zsh-completion/README.md` and `tools/zsh-completion/README_CN.md`:

- replace the local `zsh setup_claude_completion.sh` quick-install form with a `curl -fsSL ... | bash` example that targets the installer script in the GitHub repo
- include `source ~/.zshrc` after installation

For `tools/cliTitleName/README.md` and `tools/cliTitleName/README_CN.md`:

- ensure the install command uses the GitHub raw URL for `tools/cliTitleName/install.sh`
- include `source ~/.zshrc` in the install snippet itself so the quick-install block matches the documented shell-loading requirement

### Repository Structure Cleanup

Delete `tools/current-window-title-lock/` entirely.

After deletion, all documentation and tests should avoid referring to that directory as a supported tool. If tests currently assert top-level README contents or expected tool listings, update them to match the consolidated structure.

## Constraints

- Keep the terminal-title functionality under `cliTitleName` only
- Do not add compatibility wrappers or keep a duplicate old-tool directory
- Do not change the `titlename` command semantics in this task
- Do not add new commands for migration or aliasing in this task
- Keep install instructions explicit and shell-ready

## Verification Plan

1. Read the updated top-level `README.md` and `README_CN.md` and confirm they reference only `cliTitleName` for terminal-title functionality
2. Read `tools/cliTitleName/README.md` and `tools/cliTitleName/README_CN.md` and confirm they use the GitHub raw install URL and include `source ~/.zshrc`
3. Read `tools/zsh-completion/README.md` and `tools/zsh-completion/README_CN.md` and confirm their quick-install sections use `curl -fsSL ... | bash` plus `source ~/.zshrc`
4. Confirm `tools/current-window-title-lock/` no longer exists
5. Run any relevant repo tests or documentation assertions that cover these files

## Risks and Mitigations

### Risk: docs and tree drift apart again

Mitigation: update both top-level README files, tool README files, and any tests in the same change.

### Risk: install commands point to wrong raw paths

Mitigation: use the repository’s GitHub raw URL form consistently and verify the exact path text in every updated file.

### Risk: users lose immediate shell availability after install

Mitigation: include `source ~/.zshrc` directly in the quick-install snippets, not only in prose.

## Out of Scope

- changing `titlename` runtime behavior
- introducing a non-macOS implementation
- redesigning the zsh completion installer internals
- preserving the old `current-window-title-lock` tool as a deprecated package

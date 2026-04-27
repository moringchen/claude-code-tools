# cliTitleName Session Title Disable Design

**Goal:** Keep a single `titlename` command that sets the current terminal title and then disables later title rewrites for the current interactive shell session.

**Scope:** macOS only. The title-disable behavior is guaranteed only for Ghostty + interactive zsh. Outside that environment, `titlename` continues to behave as a one-shot title setter.

## Current Problem

The existing `titlename` implementation prints an OSC 2 title sequence and exits immediately. That is correct, but Ghostty shell integration later rewrites the title from its `precmd` and `preexec` hooks when `GHOSTTY_SHELL_FEATURES` contains `title`. As a result, the title appears to change briefly and then gets replaced.

## User-Approved Direction

After `titlename "..."` runs:

1. Set the requested title immediately.
2. Disable later title updates for the current shell session.
3. Do not modify Ghostty global configuration.
4. Do not make the change permanent across new shells or new windows.
5. Preserve the single-command user experience.

## Design

### Command Surface

The user-facing command remains:

```sh
titlename "My Window"
```

However, the implementation can no longer be only an external executable script, because a child process cannot remove or override the parent shell's `precmd` / `preexec` behavior. Therefore, the install flow must provide `titlename` as a shell function for interactive zsh sessions.

### Runtime Behavior

When `titlename` is invoked from the installed shell function in Ghostty + interactive zsh:

1. Validate a single non-empty title argument.
2. Validate macOS.
3. Emit the OSC 2 sequence for the requested title.
4. Detect Ghostty title-management hooks in the current shell session.
5. Disable subsequent title rewrites for this shell session only.
6. Return success without spawning a replacement shell.

When `titlename` is invoked outside Ghostty or outside interactive zsh:

1. Keep the existing one-shot behavior.
2. Do not claim that future title rewrites are disabled.

### Integration Shape

The tool directory continues to contain a single command named `titlename`, but installation changes:

- install a shell snippet or rc file fragment that defines the `titlename` function for zsh
- ensure the function is loaded into interactive shells
- keep the existing executable script as the non-function fallback for non-interactive or non-zsh contexts

The shell function is the authoritative implementation for session-level title disabling.

### Ghostty-Specific Strategy

The implementation should disable only the title-related behavior, not unrelated Ghostty features like path or cursor handling.

That means the implementation should target Ghostty's title updates in the current shell session rather than disabling all Ghostty shell integration. The intended outcome is:

- no more prompt-driven title rewrite after `titlename`
- no more command-name title rewrite before subsequent commands
- other Ghostty shell integration behavior remains intact where possible

If Ghostty's title behavior is composed into shell functions dynamically, the implementation may need to override or rewrite only the title-emitting portion in the active session.

### Documentation Changes

README files must be updated to reflect the new behavior:

- default behavior still uses one command
- in Ghostty + interactive zsh, the command also disables later title rewrites for the current shell session
- this is session-scoped, not global
- non-Ghostty and non-zsh contexts keep one-shot semantics

## Constraints

- No global Ghostty config edits.
- No permanent disabling across future shells.
- No extra user command beyond `titlename`.
- No background process or persistent lock loop.
- No promise for shells other than interactive zsh.

## Verification Plan

### Required Failing Tests First

Add tests that capture the new contract:

1. shell-function mode sets the title sequence
2. shell-function mode disables later Ghostty title rewrites in the current shell session
3. non-Ghostty mode still behaves as one-shot output only
4. invalid argument and non-macOS validation still behave exactly as before

### Manual Verification

In Ghostty + interactive zsh:

1. open a fresh shell
2. run `titlename "命名窗口工具"`
3. verify the title changes
4. press Enter / trigger a new prompt
5. verify the title does not get overwritten
6. run another command
7. verify the title still stays unchanged
8. open a new shell
9. verify the normal Ghostty title automation is back

## Risks and Mitigations

### Risk: external script cannot modify parent shell state

Mitigation: move the authoritative implementation into an installed zsh function.

### Risk: disabling too much Ghostty integration

Mitigation: scope the implementation to title rewrite behavior only, leaving unrelated features untouched.

### Risk: environment-specific behavior

Mitigation: document the guaranteed environment explicitly as Ghostty + interactive zsh on macOS.

## Out of Scope

- permanent global Ghostty title disabling
- support for every shell
- support for every terminal emulator
- restoring title automation within the same shell via a second command
- multi-command UX redesign

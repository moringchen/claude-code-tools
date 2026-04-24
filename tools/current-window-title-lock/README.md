# Current Window Title Lock

Keep a terminal window title pinned to a fixed value for the current Zsh session by using the `lock-title` and `unlock-title` shell functions.

## What This Tool Does

This tool provides a small Zsh script that:

- sets the terminal title to a value you choose with `lock-title`
- keeps that title pinned for the current Zsh session by suppressing Ghostty hook-based title updates and Claude title updates while the title is locked
- wraps `claude` so `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1` is set while the title is locked
- restores previously removed Ghostty hooks when you run `unlock-title`, but only if the corresponding hook functions are still defined
- reports the current state with `title-status`

## What Problem It Solves

Some terminal workflows update the window title automatically on every prompt or command. That is convenient most of the time, but it gets in the way when you want one window to stay visually pinned for a specific task, pairing session, or Claude run.

This script gives you a lightweight way to hold a stable title for the current Zsh session so the window stays easy to identify.

## Usage

### Source it in Zsh

```bash
source /path/to/current_window_title_lock.zsh
```

For example, from this repository:

```bash
source ./current_window_title_lock.zsh
```

If you want the functions available in every interactive Zsh session, add the source line to your `~/.zshrc`.

## Command Examples

```bash
# Lock the current window title
lock-title "Claude review"

# Check whether the title is currently locked
title-status
# locked: Claude review

# Update the locked title value
lock-title "Current Window Title Lock"

# Unlock and restore normal behavior
unlock-title

# Confirm the shell is back to normal
title-status
# unlocked
```

Typical workflow:

```bash
source ./current_window_title_lock.zsh
lock-title "Docs session"
claude
unlock-title
```

## Scope Limits and Non-Goals

This tool is intentionally narrow in scope.

- It works for the current Zsh session after you source the script.
- While locked, it suppresses Ghostty hook-based title updates and Claude title updates for that session.
- Other title-changing plugins, hooks, or terminal integrations may still override the title.
- Calling `lock-title` again replaces the current locked title.
- Calling `unlock-title` when already unlocked is safe.
- It is focused on title locking, not terminal theme, tab color, or broader shell customization.
- It does not install itself automatically.
- It does not manage titles for other shells such as Bash or Fish.
- It does not persist lock state across new terminal sessions unless you source it again.
- It is not a general window manager or terminal integration framework.

## Manual Verification

Use these steps to verify the behavior manually:

1. Open a Zsh shell in `tools/current-window-title-lock/`.
2. Run `source ./current_window_title_lock.zsh`.
3. Run `lock-title "Pinned Title"` and confirm the terminal window title changes to `Pinned Title`.
4. Run `title-status` and confirm it prints `locked: Pinned Title`.
5. Start `claude` while the title is locked and confirm the title stays pinned instead of being replaced by Claude.
6. Run `unlock-title`.
7. Run `title-status` again and confirm it prints `unlocked`.
8. Run another command and confirm your terminal returns to its normal title behavior.

## Files

- `current_window_title_lock.zsh`: Zsh implementation for `lock-title`, `unlock-title`, and `title-status`
- `test_current_window_title_lock.zsh`: shell test script for the lock and restore behavior

## License

MIT License - same as the main project.

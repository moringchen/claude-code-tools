# cliTitleName

cliTitleName is a tiny macOS-only utility with one command: `titlename`.

## macOS only

This tool is supported only on macOS.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

After installation, interactive zsh shells load `titlename` as a shell function from `~/.config/cliTitleName/titlename.zsh`. The executable at `~/.local/bin/titlename` remains available as the fallback outside that shell-function context.

## Usage

```sh
titlename "My Window"
```

## What it does

- It sets the current terminal window title immediately.
- In Ghostty, it strips Ghostty's current zsh title-write lines from the active shell hooks, so later prompt and command updates in that shell stop overwriting the title.
- In interactive zsh on macOS, it also installs a `claude()` shell wrapper.
- That wrapper launches Claude Code with `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1`, so Claude Code does not overwrite the title you set with `titlename`.
- The wrapper only affects shells that source this integration; opening a new shell without it restores normal Claude launch behavior.

## What it does not do

- It does not change Ghostty globally.
- It does not modify Ghostty config files.
- It does not modify Claude Code source.
- It does not globally change Claude Code configuration; the disable flag is injected only when the shell wrapper launches `claude`.
- It does not promise shell-wrapper behavior outside interactive zsh on macOS.
- The fallback executable at `~/.local/bin/titlename` remains a one-shot title setter.
- It expects the real `claude` executable to be available on `PATH`.
- It does not support non-macOS systems.

## Claude wrapper behavior

The installed zsh integration defines a `claude()` wrapper that resolves the real executable on `PATH` and runs it with `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1`.

If you already define your own `claude()` function, whichever definition is loaded later wins.

## Files

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`

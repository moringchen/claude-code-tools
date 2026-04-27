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
- In Ghostty + interactive zsh, it also disables later title rewrites for the current shell session.
- The disable behavior is session-scoped: opening a new shell restores Ghostty's normal title automation.

## What it does not do

- It does not change Ghostty globally.
- It does not modify Ghostty config files.
- It does not promise session-level title disabling outside Ghostty + interactive zsh.
- In Ghostty, the session-scoped disable relies on Ghostty's current zsh hook layout; if Ghostty changes that internal integration, `titlename` falls back to one-shot behavior.
- Outside Ghostty + interactive zsh, it behaves like a one-shot title setter only.
- It does not support non-macOS systems.

## Files

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`

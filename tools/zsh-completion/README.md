# Zsh Completion for Claude CLI

Enable intelligent tab completion for the `claude` command in Zsh shells.

## Features

- **Help-derived completions**: Regenerates top-level `claude` options and commands from the currently installed Claude CLI
- **Current enum values**: Picks up visible values such as `--permission-mode` and `--effort` directly from `claude --help`
- **Stable option insertion**: The generated `_claude` applies conservative local matching so aggressive global Zsh matcher rules do not distort Claude option completion
- **Claude-only matcher style**: Adds a conservative `:completion:*:*:claude:*` matcher-list without changing your global completion rules
- **Idempotent install**: Re-runs cleanly by replacing only the installer-managed block in `~/.zshrc`

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```

## What the Script Does

1. Verifies that `claude` is available in `PATH`
2. Captures `claude --help` and generates `~/.zsh/completions/_claude`
3. Updates only its own marked block in `~/.zshrc`
4. Generates a `_claude` function that keeps cursor placement and option matching stable even when your global Zsh completion matchers are more aggressive

## Supported Commands

| Command | Description |
|---------|-------------|
| `agents` | List configured agents |
| `auth` | Manage authentication |
| `auto-mode` | Inspect auto mode classifier configuration |
| `doctor` | Check the health of your Claude Code auto-updater |
| `install` | Install Claude Code native build |
| `mcp` | Configure and manage MCP servers |
| `plugin` / `plugins` | Manage Claude Code plugins |
| `setup-token` | Set up a long-lived authentication token |
| `update` / `upgrade` | Check for updates and install if available |

## Usage Examples

```bash
# Complete global options
claude --<Tab>
# Shows: --model, --permission-mode, --debug, --continue, etc.

# Complete model selection
claude --model <Tab>
# Shows: sonnet, opus, haiku, claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5

# Complete subcommands
claude <Tab>
# Shows: agents, auth, mcp, plugin, update, doctor, etc.

# Complete MCP subcommands
claude mcp <Tab>
# Shows: add, list, remove, serve, add-json, etc.

# Complete plugin subcommands
claude plugin <Tab>
# Shows: install, list, enable, disable, update, etc.
```

## Requirements

- Zsh shell
- [Claude Code](https://claude.ai/code) installed

## Troubleshooting

If completion doesn't work after installation:

```bash
# Remove completion cache and reload
rm -f ~/.zcompdump
compinit
source ~/.zshrc
```

## License

MIT License - same as the main project.

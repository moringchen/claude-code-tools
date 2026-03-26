# Zsh Completion for Claude CLI

Enable intelligent tab completion for the `claude` command in Zsh shells.

## Features

- **Smart Parameter Completion**: Auto-complete all `claude` CLI options with descriptions
- **Command Recognition**: Complete subcommands like `agents`, `mcp`, `plugin`, `update`, etc.
- **Tool Name Suggestions**: Auto-suggest available tools (`Bash`, `Edit`, `Read`, `Write`, `Glob`, `Grep`, `Agent`, `Task`, `WebFetch`, `WebSearch`, `Skill`)
- **Model Selection**: Auto-complete model options (`sonnet`, `opus`, `haiku`, `claude-opus-4-6`, etc.)
- **Permission Mode Completion**: Suggest permission modes (`acceptEdits`, `bypassPermissions`, `default`, `dontAsk`, `plan`, `auto`)

## Installation

```bash
# Run the setup script (Zsh required)
zsh setup_claude_completion.sh

# Reload your shell configuration
source ~/.zshrc
```

## What the Script Does

1. Creates `~/.zsh/completions/` directory
2. Generates `_claude` completion file with all commands and options
3. Updates `~/.zshrc` with proper `fpath` and `compinit` configuration

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

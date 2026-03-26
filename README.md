# Claude Code Tools

A curated collection of tools, scripts, and utilities for enhancing your [Claude Code](https://claude.ai/code) experience.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Tools Overview

### 1. Zsh Completion for Claude CLI

A setup script that enables intelligent tab completion for the `claude` command in Zsh shells.

#### Features
- **Smart Parameter Completion**: Auto-complete all `claude` CLI options with descriptions
- **Command Recognition**: Complete subcommands like `agents`, `mcp`, `plugin`, `update`, etc.
- **Tool Name Suggestions**: Auto-suggest available tools (`Bash`, `Edit`, `Read`, `Write`, `Glob`, `Grep`, `Agent`, `Task`, `WebFetch`, `WebSearch`, `Skill`)
- **Model Selection**: Auto-complete model options (`sonnet`, `opus`, `haiku`, `claude-opus-4-6`, etc.)
- **Permission Mode Completion**: Suggest permission modes (`acceptEdits`, `bypassPermissions`, `default`, `dontAsk`, `plan`, `auto`)

#### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/claude-code-tools.git
cd claude-code-tools

# Run the setup script (Zsh required)
zsh scripts/setup_claude_completion.sh

# Reload your shell configuration
source ~/.zshrc
```

#### Supported Commands

The completion script supports all Claude CLI commands:

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

#### Global Options Supported

- `--model` - Select AI model (sonnet, opus, haiku, etc.)
- `--permission-mode` - Set permission handling mode
- `--allowed-tools` / `--disallowed-tools` - Control available tools
- `--debug` - Enable debug mode
- `--continue` / `--resume` - Resume conversations
- `--worktree` - Create git worktrees
- `--mcp-config` - Load MCP server configurations
- And many more...

## Requirements

- [Claude Code](https://claude.ai/code) installed
- Zsh shell (for completion script)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Anthropic](https://www.anthropic.com/) for creating Claude and Claude Code
- The Claude Code community for inspiration and feedback

---

More tools coming soon! Stay tuned for updates.

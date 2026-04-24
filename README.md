# Claude Code Tools

A curated collection of tools, scripts, and utilities for enhancing your [Claude Code](https://claude.ai/code) experience.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
**[中文](README_CN.md)**

---

## Tools Overview

### 1. [Zsh Completion for Claude CLI](tools/zsh-completion/)

Enable intelligent tab completion for the `claude` command in Zsh shells.

**Key Features:**
- Smart parameter completion for all CLI options
- Auto-complete subcommands (agents, mcp, plugin, etc.)
- Tool name suggestions (Bash, Edit, Read, Write, etc.)
- Model selection (sonnet, opus, haiku, etc.)

**Quick Install:**
```bash
zsh tools/zsh-completion/setup_claude_completion.sh
source ~/.zshrc
```

### 2. [Current Window Title Lock](tools/current-window-title-lock/)

Keep a terminal window title pinned to a fixed value for the current Zsh session.

**Key Features:**
- Lock the current window title with `lock-title`
- Prevent Claude in that shell from changing the locked title
- Restore normal title behavior with `unlock-title`
- Check the current lock state with `title-status`

**Quick Install:**
```bash
source tools/current-window-title-lock/current_window_title_lock.zsh
lock-title "Claude review"
```

---

## Project Structure

```
claudetools/
├── LICENSE
├── README.md                        # English (This file)
├── README_CN.md                     # 中文
└── tools/
    ├── current-window-title-lock/   # Current window title lock tool
    │   ├── current_window_title_lock.zsh
    │   ├── test_current_window_title_lock.zsh
    │   ├── README.md                # Tool documentation (EN)
    │   └── README_CN.md             # Tool documentation (CN)
    └── zsh-completion/              # Zsh completion tool
        ├── setup_claude_completion.sh
        ├── README.md                # Tool documentation (EN)
        └── README_CN.md             # Tool documentation (CN)
```

Each tool has its own directory with:
- The tool script(s)
- English documentation (`README.md`)
- Chinese documentation (`README_CN.md`)

---

## Installation

```bash
# Clone the repository
git clone git@github.com:moringchen/claude-code-tools.git
cd claude-code-tools

# Choose a tool and follow its README
cd tools/zsh-completion
zsh setup_claude_completion.sh
```

---

## Contributing

Contributions are welcome! When adding a new tool:

1. Create a new directory under `tools/`
2. Include the tool script(s)
3. Add both English and Chinese README files
4. Update the main README with a link to your tool

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

More tools coming soon! Stay tuned for updates.

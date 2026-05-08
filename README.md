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
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```

### 2. [cliTitleName](tools/cliTitleName/)

Set the current terminal window title with a single `titlename` command.

**Key Features:**
- Set the current terminal window title immediately
- Automatically install a zsh `claude()` wrapper on macOS interactive zsh
- Launch Claude Code with `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1` so it does not overwrite your chosen title
- Keep the command surface to a single `titlename "..."` invocation
- Leave the standalone `titlename` executable as a one-shot fallback

**Quick Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

### 3. [claudeBoard](tools/claudeBoard/)

A desktop overlay for Claude Code task activity with user-level global hooks, an always-visible top bar, task counts, click-to-focus actions, and notification or voice toggles.

**Quick Start:**
```bash
cd tools/claudeBoard
npm install
npm test
```

### 4. [karpahere](tools/karpahere/)

Install a reusable `/karpahere` slash that appends a single marked Karpathy guidelines block into the current project's `.claude/CLAUDE.md`.

It brings Karpathy-inspired guardrails into project instructions to reduce silent assumptions, overengineering, unrelated edits, and vague execution.
In practice, using the slash is a way to "summon Karpathy into the project" by writing those principles into the repo's instruction file.

**Quick Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/karpahere/install.sh | bash
```

---

## Project Structure

```
claudetools/
├── LICENSE
├── README.md                        # English (This file)
├── README_CN.md                     # 中文
└── tools/
    ├── cliTitleName/                # Terminal title tool
    │   ├── titlename
    │   ├── titlename.zsh
    │   ├── install.sh
    │   ├── test_cliTitleName.zsh
    │   ├── README.md                # Tool documentation (EN)
    │   └── README_CN.md             # Tool documentation (CN)
    ├── zsh-completion/              # Zsh completion tool
    │   ├── setup_claude_completion.sh
    │   ├── README.md                # Tool documentation (EN)
    │   └── README_CN.md             # Tool documentation (CN)
    ├── claudeBoard/                 # claudeBoard desktop overlay
    │   ├── README.md                # Tool documentation (EN)
    │   ├── README_CN.md             # Tool documentation (CN)
    │   ├── package.json
    │   ├── src/
    │   ├── scripts/
    │   └── src-tauri/
    └── karpahere/                   # Local /karpahere slash installer
        ├── install.sh
        ├── SKILL.md
        ├── karpathy-guidelines.md
        ├── README.md                # Tool documentation (EN)
        ├── README_CN.md             # Tool documentation (CN)
        └── upstream/
```

Each tool has its own directory with:
- The tool script(s)
- English documentation (`README.md`)
- Chinese documentation (`README_CN.md`)

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

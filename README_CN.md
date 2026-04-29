# Claude Code 工具集

一个精心整理的 [Claude Code](https://claude.ai/code) 工具、脚本和实用程序集合，用于提升您的使用体验。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
**[English](README.md)**

---

## 工具概览

### 1. [Claude CLI Zsh 参数自动补齐](tools/zsh-completion/)

为 Zsh 终端中的 `claude` 命令提供智能 Tab 键自动补齐功能。

**主要功能：**
- 所有 CLI 选项的智能参数补齐
- 子命令自动补齐（agents、mcp、plugin 等）
- 工具名称提示（Bash、Edit、Read、Write 等）
- 模型选择（sonnet、opus、haiku 等）

**快速安装：**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```

### 2. [cliTitleName](tools/cliTitleName/)

使用单个 `titlename` 命令设置当前终端窗口标题。

**主要功能：**
- 立即设置当前终端窗口标题
- 在 Ghostty + 交互式 zsh 中，禁用当前 shell 会话后续的标题改写
- 保持单一命令入口：`titlename "..."`
- 新开一个 shell 后自动恢复 Ghostty 默认标题行为

**快速安装：**
```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

### 3. [Claude Task Window](tools/claudeTaskWindow/)

用于展示 Claude Code 任务活动的桌面悬浮层，提供用户级全局 hooks、顶部常驻条、任务统计、点击聚焦，以及通知与语音播报开关。

**快速开始：**
```bash
cd tools/claudeTaskWindow
npm install
npm test
```

---

## 项目结构

```
claudetools/
├── LICENSE
├── README.md                        # 英文
├── README_CN.md                     # 中文（本文件）
└── tools/
    ├── cliTitleName/                # 终端标题工具
    │   ├── titlename
    │   ├── titlename.zsh
    │   ├── install.sh
    │   ├── test_cliTitleName.zsh
    │   ├── README.md                # 工具文档（英文）
    │   └── README_CN.md             # 工具文档（中文）
    ├── zsh-completion/              # Zsh 自动补齐工具
    │   ├── setup_claude_completion.sh
    │   ├── README.md                # 工具文档（英文）
    │   └── README_CN.md             # 工具文档（中文）
    └── claudeTaskWindow/            # Claude Task Window 桌面悬浮工具
        ├── README.md                # 工具文档（英文）
        ├── README_CN.md             # 工具文档（中文）
        ├── package.json
        ├── src/
        ├── scripts/
        └── src-tauri/
```

每个工具都有独立的目录，包含：
- 工具脚本
- 英文说明文档（`README.md`）
- 中文说明文档（`README_CN.md`）

---

## 贡献指南

欢迎贡献！添加新工具时请遵循：

1. 在 `tools/` 下创建新目录
2. 包含工具脚本
3. 添加中英文 README 文件
4. 在主 README 中添加工具链接

---

## 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件。

---

更多工具即将推出！敬请关注更新。

---
## 社区支持
学 AI , 上 L 站

[LinuxDO](https://linux.do)

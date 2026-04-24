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
zsh tools/zsh-completion/setup_claude_completion.sh
source ~/.zshrc
```

### 2. [当前窗口标题锁定](tools/current-window-title-lock/)

在当前 Zsh 会话中将终端窗口标题固定为指定值。

**主要功能：**
- 使用 `lock-title` 锁定当前窗口标题
- 防止该 shell 中的 Claude 改写已锁定的标题
- 使用 `unlock-title` 恢复正常标题行为
- 使用 `title-status` 查看当前锁定状态

**快速安装：**
```bash
source tools/current-window-title-lock/current_window_title_lock.zsh
lock-title "Claude review"
```

---

## 项目结构

```
claudetools/
├── LICENSE
├── README.md                        # 英文
├── README_CN.md                     # 中文（本文件）
└── tools/
    ├── current-window-title-lock/   # 当前窗口标题锁定工具
    │   ├── current_window_title_lock.zsh
    │   ├── test_current_window_title_lock.zsh
    │   ├── README.md                # 工具文档（英文）
    │   └── README_CN.md             # 工具文档（中文）
    └── zsh-completion/              # Zsh 自动补齐工具
        ├── setup_claude_completion.sh
        ├── README.md                # 工具文档（英文）
        └── README_CN.md             # 工具文档（中文）
```

每个工具都有独立的目录，包含：
- 工具脚本
- 英文说明文档（`README.md`）
- 中文说明文档（`README_CN.md`）

---

## 安装方法

```bash
# 克隆仓库
git clone git@github.com:moringchen/claude-code-tools.git
cd claude-code-tools

# 选择工具并按 README 说明操作
cd tools/zsh-completion
zsh setup_claude_completion.sh
```

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

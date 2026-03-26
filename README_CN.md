# Claude Code 工具集

一个精心整理的 [Claude Code](https://claude.ai/code) 工具、脚本和实用程序集合，用于提升您的使用体验。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 工具概览

### 1. Claude CLI Zsh 参数自动补齐

一个配置脚本，为 Zsh 终端中的 `claude` 命令提供智能的 Tab 键自动补齐功能。

#### 功能特性
- **智能参数补齐**：自动补齐所有 `claude` CLI 选项并显示描述
- **命令识别**：补齐子命令如 `agents`、`mcp`、`plugin`、`update` 等
- **工具名称建议**：自动提示可用工具（`Bash`、`Edit`、`Read`、`Write`、`Glob`、`Grep`、`Agent`、`Task`、`WebFetch`、`WebSearch`、`Skill`）
- **模型选择**：自动补齐模型选项（`sonnet`、`opus`、`haiku`、`claude-opus-4-6` 等）
- **权限模式补齐**：提示权限模式（`acceptEdits`、`bypassPermissions`、`default`、`dontAsk`、`plan`、`auto`）

#### 安装方法

```bash
# 克隆仓库
git clone https://github.com/yourusername/claude-code-tools.git
cd claude-code-tools

# 运行配置脚本（需要 Zsh）
zsh scripts/setup_claude_completion.sh

# 重新加载 shell 配置
source ~/.zshrc
```

#### 支持的命令

补齐脚本支持所有 Claude CLI 命令：

| 命令 | 描述 |
|---------|-------------|
| `agents` | 列出已配置的代理 |
| `auth` | 管理认证 |
| `auto-mode` | 检查自动模式分类器配置 |
| `doctor` | 检查 Claude Code 自动更新器的健康状态 |
| `install` | 安装 Claude Code 原生构建版本 |
| `mcp` | 配置和管理 MCP 服务器 |
| `plugin` / `plugins` | 管理 Claude Code 插件 |
| `setup-token` | 设置长期有效的认证令牌 |
| `update` / `upgrade` | 检查并安装更新 |

#### 支持的全局选项

- `--model` - 选择 AI 模型（sonnet、opus、haiku 等）
- `--permission-mode` - 设置权限处理模式
- `--allowed-tools` / `--disallowed-tools` - 控制可用工具
- `--debug` - 启用调试模式
- `--continue` / `--resume` - 恢复对话
- `--worktree` - 创建 git 工作树
- `--mcp-config` - 加载 MCP 服务器配置
- 以及更多...

## 系统要求

- 已安装 [Claude Code](https://claude.ai/code)
- Zsh 终端（用于补齐脚本）

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件。

## 致谢

- [Anthropic](https://www.anthropic.com/) 创建 Claude 和 Claude Code
- Claude Code 社区提供的灵感和反馈

---

更多工具即将推出！敬请关注更新。

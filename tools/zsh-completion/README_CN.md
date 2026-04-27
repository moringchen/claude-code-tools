# Claude CLI Zsh 参数自动补齐

为 Zsh 终端中的 `claude` 命令提供智能 Tab 键自动补齐功能。

## 功能特性

- **智能参数补齐**：自动补齐所有 `claude` CLI 选项并显示描述
- **命令识别**：补齐子命令如 `agents`、`mcp`、`plugin`、`update` 等
- **工具名称建议**：自动提示可用工具（`Bash`、`Edit`、`Read`、`Write`、`Glob`、`Grep`、`Agent`、`Task`、`WebFetch`、`WebSearch`、`Skill`）
- **模型选择**：自动补齐模型选项（`sonnet`、`opus`、`haiku`、`claude-opus-4-6` 等）
- **权限模式补齐**：提示权限模式（`acceptEdits`、`bypassPermissions`、`default`、`dontAsk`、`plan`、`auto`）

## 安装方法

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash
source ~/.zshrc
```

## 脚本功能说明

1. 创建 `~/.zsh/completions/` 目录
2. 生成包含所有命令和选项的 `_claude` 补齐文件
3. 更新 `~/.zshrc`，添加正确的 `fpath` 和 `compinit` 配置

## 支持的命令

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

## 使用示例

```bash
# 补齐全局选项
claude --<Tab>
# 显示：--model, --permission-mode, --debug, --continue, 等

# 补齐模型选择
claude --model <Tab>
# 显示：sonnet, opus, haiku, claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5

# 补齐子命令
claude <Tab>
# 显示：agents, auth, mcp, plugin, update, doctor, 等

# 补齐 MCP 子命令
claude mcp <Tab>
# 显示：add, list, remove, serve, add-json, 等

# 补齐插件子命令
claude plugin <Tab>
# 显示：install, list, enable, disable, update, 等
```

## 系统要求

- Zsh 终端
- 已安装 [Claude Code](https://claude.ai/code)

## 故障排除

如果安装后补齐不生效：

```bash
# 清除补齐缓存并重新加载
rm -f ~/.zcompdump
compinit
source ~/.zshrc
```

## 许可证

MIT 许可证 - 与主项目相同。

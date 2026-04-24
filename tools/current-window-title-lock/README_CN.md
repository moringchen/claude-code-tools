# Current Window Title Lock

通过 `lock-title` 和 `unlock-title` 这两个 Zsh 函数，在当前 Zsh 会话中将终端窗口标题固定为你指定的值。

## 工具作用

这个工具提供了一个小型 Zsh 脚本，用来：

- 使用 `lock-title` 将终端标题设置为指定值
- 在当前 Zsh 会话中，在标题锁定期间抑制 Ghostty 基于 hook 的标题更新，以及 Claude 的标题更新，从而尽量保持该标题不变
- 在标题锁定期间包装 `claude`，并设置 `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1`
- 在执行 `unlock-title` 后恢复之前移除的 Ghostty hook，但前提是对应的 hook 函数仍然已定义
- 通过 `title-status` 查看当前锁定状态

## 它解决什么问题

很多终端工作流会在每次执行命令或显示提示符时自动更新窗口标题。平时这很方便，但当你希望某个窗口一直显示固定标题，用来区分某个任务、配对会话或 Claude 工作窗口时，就会造成干扰。

这个脚本提供了一种轻量方式，让当前 Zsh 会话保持稳定标题，便于你快速识别目标窗口。

## 使用方法

### 在 Zsh 中 source 脚本

```bash
source /path/to/current_window_title_lock.zsh
```

在当前仓库中可以这样执行：

```bash
source ./current_window_title_lock.zsh
```

如果你希望每次启动交互式 Zsh 都自动获得这些函数，可以把这条 `source` 语句加入 `~/.zshrc`。

## 命令示例

```bash
# 锁定当前窗口标题
lock-title "Claude review"

# 查看当前是否已锁定
title-status
# locked: Claude review

# 更新锁定后的标题
lock-title "Current Window Title Lock"

# 解锁并恢复正常行为
unlock-title

# 确认 shell 已恢复正常
title-status
# unlocked
```

典型使用流程：

```bash
source ./current_window_title_lock.zsh
lock-title "Docs session"
claude
unlock-title
```

## 作用范围与非目标

这个工具有意保持很小的范围。

- 只在你 source 脚本之后的当前 Zsh 会话中生效。
- 在锁定期间，它会抑制该会话中的 Ghostty 基于 hook 的标题更新，以及 Claude 的标题更新。
- 其他会改写标题的插件、hook 或终端集成仍然可能覆盖当前标题。
- 再次调用 `lock-title` 会直接替换当前锁定标题。
- 在已经解锁的状态下调用 `unlock-title` 也是安全的。
- 只关注标题锁定，不负责终端主题、标签颜色或更广泛的 shell 定制。
- 不会自动安装自己。
- 不负责 Bash、Fish 等其他 shell 的标题管理。
- 不会在新的终端会话之间自动持久化锁定状态，除非你再次 source 脚本。
- 它不是通用窗口管理器，也不是完整的终端集成框架。

## 手动验证步骤

可以按下面的步骤手动验证：

1. 在 `tools/current-window-title-lock/` 目录打开一个 Zsh shell。
2. 执行 `source ./current_window_title_lock.zsh`。
3. 执行 `lock-title "Pinned Title"`，确认终端窗口标题变成 `Pinned Title`。
4. 执行 `title-status`，确认输出为 `locked: Pinned Title`。
5. 在标题锁定期间启动 `claude`，确认标题保持固定，不会被 Claude 改写。
6. 执行 `unlock-title`。
7. 再次执行 `title-status`，确认输出为 `unlocked`。
8. 再运行任意命令，确认终端标题恢复为平常的更新方式。

## 文件说明

- `current_window_title_lock.zsh`：`lock-title`、`unlock-title` 和 `title-status` 的 Zsh 实现
- `test_current_window_title_lock.zsh`：覆盖锁定与恢复行为的 shell 测试脚本

## 许可证

MIT 许可证 - 与主项目保持一致。

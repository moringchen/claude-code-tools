# cliTitleName

cliTitleName 是一个极简的、仅支持 macOS 的工具，只有一个命令 `titlename`。

## 仅支持 macOS

这个工具只支持 macOS。

## 安装

```sh
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash
source ~/.zshrc
```

安装后，交互式 zsh 会从 `~/.config/cliTitleName/titlename.zsh` 加载 `titlename` shell function。`~/.local/bin/titlename` 这个可执行文件仍然会保留，用作非 shell-function 场景下的回退入口。

## 使用方式

```sh
titlename "My Window"
```

## 它会做什么

- 它会立即设置当前终端窗口标题。
- 在 Ghostty 中，它会把当前 shell hook 里负责写标题的那几行去掉，让这个 shell 后续的提示符和命令更新不再覆盖标题。
- 在 macOS 的交互式 zsh 中，它还会安装一个 `claude()` shell wrapper。
- 这个 wrapper 会在启动 Claude Code 时注入 `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1`，避免 Claude Code 覆盖你用 `titlename` 设置的标题。
- 这个 wrapper 只作用于加载了这份集成脚本的 shell；新开一个未加载它的 shell 会恢复普通的 Claude 启动行为。

## 它不会做什么

- 它不会修改 Ghostty 的全局配置。
- 它不会改写 Ghostty 配置文件。
- 它不会修改 Claude Code 源码。
- 它不会全局修改 Claude Code 配置；只有通过这个 shell wrapper 启动 `claude` 时才会注入禁用标题的环境变量。
- 它不承诺在 macOS 的交互式 zsh 之外提供这个 shell-wrapper 行为。
- `~/.local/bin/titlename` 这个回退可执行文件仍然只是一次性设置标题。
- 它要求真正的 `claude` 可执行文件存在于 `PATH` 中。
- 它不支持非 macOS 系统。

## `claude` wrapper 行为

安装后的 zsh 集成会定义一个 `claude()` wrapper：先解析 `PATH` 里的真实可执行文件，再用 `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1` 启动它。

如果你自己也定义了 `claude()` function，后加载的那个定义会生效。

## 文件

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`

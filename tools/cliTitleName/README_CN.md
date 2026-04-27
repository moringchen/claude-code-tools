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
- 在 Ghostty + 交互式 zsh 中，它还会禁用当前 shell 会话后续的标题改写。
- 这个禁用行为只作用于当前会话；新开一个 shell 后，Ghostty 的默认标题自动更新会恢复。

## 它不会做什么

- 它不会修改 Ghostty 的全局配置。
- 它不会改写 Ghostty 配置文件。
- 它不承诺在 Ghostty + 交互式 zsh 之外提供会话级标题禁用能力。
- 在 Ghostty 中，这个会话级禁用依赖 Ghostty 当前的 zsh hook 实现；如果 Ghostty 以后调整这层内部集成，`titlename` 会退回到一次性设置标题的行为。
- 在 Ghostty + 交互式 zsh 之外，它仍然只是一次性设置标题。
- 它不支持非 macOS 系统。

## 文件

- `titlename`
- `titlename.zsh`
- `install.sh`
- `test_cliTitleName.zsh`

#!/bin/zsh

# Claude CLI Zsh 参数补齐一键配置脚本
# 用法: zsh setup_claude_completion.sh

set -e

COMPLETIONS_DIR="$HOME/.zsh/completions"
ZSHRC="$HOME/.zshrc"

echo "=== Claude CLI Zsh 参数补齐配置 ==="
echo ""

# 1. 创建 completions 目录
if [[ ! -d "$COMPLETIONS_DIR" ]]; then
    echo "📁 创建目录: $COMPLETIONS_DIR"
    if ! mkdir -p "$COMPLETIONS_DIR" 2>/dev/null; then
        echo "❌ 错误: 无法创建目录 $COMPLETIONS_DIR"
        echo "   请检查是否有写入权限，或手动创建该目录后重试"
        exit 1
    fi
else
    echo "✅ 目录已存在: $COMPLETIONS_DIR"
fi

# 检查目录是否可写
if [[ ! -w "$COMPLETIONS_DIR" ]]; then
    echo "❌ 错误: 目录 $COMPLETIONS_DIR 没有写入权限"
    echo "   请修改权限后再运行: chmod u+w $COMPLETIONS_DIR"
    exit 1
fi

# 2. 写入 _claude 补齐文件
echo "📝 写入 _claude 补齐文件..."
if ! cat > "$COMPLETIONS_DIR/_claude" << 'CLAUDeOF'
#compdef claude

# Claude CLI Zsh Completion Script
# Generated for Claude Code CLI

_claude() {
  local curcontext="$curcontext" state line
  typeset -A opt_args

  local -a global_options
  global_options=(
    '--add-dir[Additional directories to allow tool access to]:directories:_directories -/'
    '--agent[Agent for the current session]:agent:->agents'
    '--agents[JSON object defining custom agents]:json:'
    '--allow-dangerously-skip-permissions[Enable bypassing all permission checks as an option]'
    '(--allowed-tools --allowedTools)'{--allowed-tools,--allowedTools}'[Comma-separated list of tool names to allow]:tools:->tools'
    '--append-system-prompt[Append a system prompt to the default system prompt]:prompt:'
    '--betas[Beta headers to include in API requests]:betas:'
    '--brief[Enable SendUserMessage tool for agent-to-user communication]'
    '--chrome[Enable Claude in Chrome integration]'
    '(-c --continue)'{-c,--continue}'[Continue the most recent conversation]'
    '--dangerously-skip-permissions[Bypass all permission checks]'
    '(-d --debug)'{-d,--debug}'[Enable debug mode with optional filtering]:filter:(api hooks 1p file mcp permission)'
    '--debug-file[Write debug logs to specific file path]:path:_files'
    '--disable-slash-commands[Disable all skills]'
    '(--disallowed-tools --disallowedTools)'{--disallowed-tools,--disallowedTools}'[Comma-separated list of tool names to deny]:tools:->tools'
    '--effort[Effort level for the session]:level:(low medium high max)'
    '--fallback-model[Enable automatic fallback to specified model]:model:(sonnet opus haiku claude-opus-4-6 claude-sonnet-4-6 claude-haiku-4-5)'
    '--file[File resources to download at startup]:file:_files'
    '--fork-session[Create a new session ID when resuming]'
    '--from-pr[Resume a session linked to a PR]:pr:'
    '(-h --help)'{-h,--help}'[Display help for command]'
    '--ide[Automatically connect to IDE on startup]'
    '--include-partial-messages[Include partial message chunks as they arrive]'
    '--input-format[Input format]:format:(text stream-json)'
    '--json-schema[JSON Schema for structured output validation]:schema:'
    '--max-budget-usd[Maximum dollar amount to spend on API calls]:amount:'
    '--mcp-config[Load MCP servers from JSON files]:config:_files -g "*.json"'
    '--mcp-debug[Enable MCP debug mode (deprecated)]'
    '--model[Model for the current session]:model:(sonnet opus haiku claude-opus-4-6 claude-sonnet-4-6 claude-haiku-4-5)'
    '--no-chrome[Disable Claude in Chrome integration]'
    '--no-session-persistence[Disable session persistence]'
    '--output-format[Output format]:format:(text json stream-json)'
    '--permission-mode[Permission mode for the session]:mode:(acceptEdits bypassPermissions default dontAsk plan auto)'
    '--plugin-dir[Load plugins from directories]:directory:_directories -/'
    '(-p --print)'{-p,--print}'[Print response and exit]'
    '--replay-user-messages[Re-emit user messages from stdin back on stdout]'
    '(-r --resume)'{-r,--resume}'[Resume a conversation by session ID]:session:'
    '--session-id[Use a specific session ID]:uuid:'
    '--setting-sources[Comma-separated list of setting sources]:sources:(user project local)'
    '--settings[Path to settings JSON file]:file:_files -g "*.json"'
    '--strict-mcp-config[Only use MCP servers from --mcp-config]'
    '--system-prompt[System prompt to use for the session]:prompt:'
    '--tmux[Create a tmux session for the worktree]'
    '--tools[Specify the list of available tools]:tools:->tools'
    '--verbose[Override verbose mode setting from config]'
    '(-v --version)'{-v,--version}'[Output the version number]'
    '(-w --worktree)'{-w,--worktree}'[Create a new git worktree]:name:'
  )

  local -a commands
  commands=(
    'agents:List configured agents'
    'auth:Manage authentication'
    'auto-mode:Inspect auto mode classifier configuration'
    'doctor:Check the health of your Claude Code auto-updater'
    'install:Install Claude Code native build'
    'mcp:Configure and manage MCP servers'
    'plugin:Manage Claude Code plugins'
    'plugins:Manage Claude Code plugins'
    'setup-token:Set up a long-lived authentication token'
    'update:Check for updates and install if available'
    'upgrade:Check for updates and install if available'
  )

  _arguments -C \
    $global_options \
    ': :->command' \
    '*:: :->args'

  case "$state" in
    command)
      _describe -t commands 'claude commands' commands
      ;;

    args)
      case "$line[1]" in
        agents)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]' \
            '--setting-sources[Comma-separated list of setting sources]:sources:(user project local)'
          ;;

        auth)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        auto-mode)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        doctor)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        install)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]' \
            '--force[Force installation even if already installed]' \
            ':target:(stable latest)'
          ;;

        mcp)
          local -a mcp_commands
          mcp_commands=(
            'add:Add an MCP server to Claude Code'
            'add-from-claude-desktop:Import MCP servers from Claude Desktop'
            'add-json:Add an MCP server with a JSON string'
            'get:Get details about an MCP server'
            'help:Display help for command'
            'list:List configured MCP servers'
            'remove:Remove an MCP server'
            'reset-project-choices:Reset all approved and rejected project-scoped servers'
            'serve:Start the Claude Code MCP server'
          )

          if (( CURRENT == 2 )); then
            _describe -t commands 'mcp commands' mcp_commands
          else
            case "$line[2]" in
              add)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  '--transport[Transport type]:transport:(http stdio)' \
                  '--header[HTTP headers to include]:header:' \
                  '(-e --env)'{-e,--env}'[Environment variables]:env:' \
                  ':name:' \
                  ':command or URL:' \
                  '*:args:'
                ;;
              add-from-claude-desktop)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              add-json)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:' \
                  ':json:'
                ;;
              get)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:'
                ;;
              remove)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:'
                ;;
              list)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              reset-project-choices)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              serve)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              *)
                _files
                ;;
            esac
          fi
          ;;

        plugin|plugins)
          local -a plugin_commands
          plugin_commands=(
            'disable:Disable an enabled plugin'
            'enable:Enable a disabled plugin'
            'help:Display help for command'
            'install:Install a plugin from available marketplaces'
            'i:Install a plugin from available marketplaces'
            'list:List installed plugins'
            'marketplace:Manage Claude Code marketplaces'
            'uninstall:Uninstall an installed plugin'
            'remove:Uninstall an installed plugin'
            'update:Update a plugin to the latest version'
            'validate:Validate a plugin or marketplace manifest'
          )

          if (( CURRENT == 2 )); then
            _describe -t commands 'plugin commands' plugin_commands
          else
            case "$line[2]" in
              disable)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  '::plugin:'
                ;;
              enable)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              install|i)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              list)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              marketplace)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              uninstall|remove)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              update)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              validate)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':path:_directories'
                ;;
              *)
                _files
                ;;
            esac
          fi
          ;;

        setup-token)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        update|upgrade)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        *)
          _files
          ;;
      esac
      ;;

    tools)
      local -a tools_list
      tools_list=(
        'Bash'
        'Edit'
        'Read'
        'Write'
        'Glob'
        'Grep'
        'Agent'
        'Task'
        'WebFetch'
        'WebSearch'
        'Skill'
      )
      _describe -t tools 'available tools' tools_list
      ;;

    agents)
      local -a agents_list
      agents_list=(
        'default'
        'explore'
        'plan'
        'test'
        'review'
      )
      _describe -t agents 'available agents' agents_list
      ;;
  esac
}
CLAUDeOF
then
    echo "❌ 错误: 无法写入文件 $COMPLETIONS_DIR/_claude"
    echo "   请检查磁盘空间或权限设置"
    exit 1
fi
#compdef claude

# Claude CLI Zsh Completion Script
# Generated for Claude Code CLI

_claude() {
  local curcontext="$curcontext" state line
  typeset -A opt_args

  local -a global_options
  global_options=(
    '--add-dir[Additional directories to allow tool access to]:directories:_directories -/'
    '--agent[Agent for the current session]:agent:->agents'
    '--agents[JSON object defining custom agents]:json:'
    '--allow-dangerously-skip-permissions[Enable bypassing all permission checks as an option]'
    '(--allowed-tools --allowedTools)'{--allowed-tools,--allowedTools}'[Comma-separated list of tool names to allow]:tools:->tools'
    '--append-system-prompt[Append a system prompt to the default system prompt]:prompt:'
    '--betas[Beta headers to include in API requests]:betas:'
    '--brief[Enable SendUserMessage tool for agent-to-user communication]'
    '--chrome[Enable Claude in Chrome integration]'
    '(-c --continue)'{-c,--continue}'[Continue the most recent conversation]'
    '--dangerously-skip-permissions[Bypass all permission checks]'
    '(-d --debug)'{-d,--debug}'[Enable debug mode with optional filtering]:filter:(api hooks 1p file mcp permission)'
    '--debug-file[Write debug logs to specific file path]:path:_files'
    '--disable-slash-commands[Disable all skills]'
    '(--disallowed-tools --disallowedTools)'{--disallowed-tools,--disallowedTools}'[Comma-separated list of tool names to deny]:tools:->tools'
    '--effort[Effort level for the session]:level:(low medium high max)'
    '--fallback-model[Enable automatic fallback to specified model]:model:(sonnet opus haiku claude-opus-4-6 claude-sonnet-4-6 claude-haiku-4-5)'
    '--file[File resources to download at startup]:file:_files'
    '--fork-session[Create a new session ID when resuming]'
    '--from-pr[Resume a session linked to a PR]:pr:'
    '(-h --help)'{-h,--help}'[Display help for command]'
    '--ide[Automatically connect to IDE on startup]'
    '--include-partial-messages[Include partial message chunks as they arrive]'
    '--input-format[Input format]:format:(text stream-json)'
    '--json-schema[JSON Schema for structured output validation]:schema:'
    '--max-budget-usd[Maximum dollar amount to spend on API calls]:amount:'
    '--mcp-config[Load MCP servers from JSON files]:config:_files -g "*.json"'
    '--mcp-debug[Enable MCP debug mode (deprecated)]'
    '--model[Model for the current session]:model:(sonnet opus haiku claude-opus-4-6 claude-sonnet-4-6 claude-haiku-4-5)'
    '--no-chrome[Disable Claude in Chrome integration]'
    '--no-session-persistence[Disable session persistence]'
    '--output-format[Output format]:format:(text json stream-json)'
    '--permission-mode[Permission mode for the session]:mode:(acceptEdits bypassPermissions default dontAsk plan auto)'
    '--plugin-dir[Load plugins from directories]:directory:_directories -/'
    '(-p --print)'{-p,--print}'[Print response and exit]'
    '--replay-user-messages[Re-emit user messages from stdin back on stdout]'
    '(-r --resume)'{-r,--resume}'[Resume a conversation by session ID]:session:'
    '--session-id[Use a specific session ID]:uuid:'
    '--setting-sources[Comma-separated list of setting sources]:sources:(user project local)'
    '--settings[Path to settings JSON file]:file:_files -g "*.json"'
    '--strict-mcp-config[Only use MCP servers from --mcp-config]'
    '--system-prompt[System prompt to use for the session]:prompt:'
    '--tmux[Create a tmux session for the worktree]'
    '--tools[Specify the list of available tools]:tools:->tools'
    '--verbose[Override verbose mode setting from config]'
    '(-v --version)'{-v,--version}'[Output the version number]'
    '(-w --worktree)'{-w,--worktree}'[Create a new git worktree]:name:'
  )

  local -a commands
  commands=(
    'agents:List configured agents'
    'auth:Manage authentication'
    'auto-mode:Inspect auto mode classifier configuration'
    'doctor:Check the health of your Claude Code auto-updater'
    'install:Install Claude Code native build'
    'mcp:Configure and manage MCP servers'
    'plugin:Manage Claude Code plugins'
    'plugins:Manage Claude Code plugins'
    'setup-token:Set up a long-lived authentication token'
    'update:Check for updates and install if available'
    'upgrade:Check for updates and install if available'
  )

  _arguments -C \
    $global_options \
    ': :->command' \
    '*:: :->args'

  case "$state" in
    command)
      _describe -t commands 'claude commands' commands
      ;;

    args)
      case "$line[1]" in
        agents)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]' \
            '--setting-sources[Comma-separated list of setting sources]:sources:(user project local)'
          ;;

        auth)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        auto-mode)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        doctor)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        install)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]' \
            '--force[Force installation even if already installed]' \
            ':target:(stable latest)'
          ;;

        mcp)
          local -a mcp_commands
          mcp_commands=(
            'add:Add an MCP server to Claude Code'
            'add-from-claude-desktop:Import MCP servers from Claude Desktop'
            'add-json:Add an MCP server with a JSON string'
            'get:Get details about an MCP server'
            'help:Display help for command'
            'list:List configured MCP servers'
            'remove:Remove an MCP server'
            'reset-project-choices:Reset all approved and rejected project-scoped servers'
            'serve:Start the Claude Code MCP server'
          )

          if (( CURRENT == 2 )); then
            _describe -t commands 'mcp commands' mcp_commands
          else
            case "$line[2]" in
              add)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  '--transport[Transport type]:transport:(http stdio)' \
                  '--header[HTTP headers to include]:header:' \
                  '(-e --env)'{-e,--env}'[Environment variables]:env:' \
                  ':name:' \
                  ':command or URL:' \
                  '*:args:'
                ;;
              add-from-claude-desktop)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              add-json)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:' \
                  ':json:'
                ;;
              get)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:'
                ;;
              remove)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':name:'
                ;;
              list)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              reset-project-choices)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              serve)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              *)
                _files
                ;;
            esac
          fi
          ;;

        plugin|plugins)
          local -a plugin_commands
          plugin_commands=(
            'disable:Disable an enabled plugin'
            'enable:Enable a disabled plugin'
            'help:Display help for command'
            'install:Install a plugin from available marketplaces'
            'i:Install a plugin from available marketplaces'
            'list:List installed plugins'
            'marketplace:Manage Claude Code marketplaces'
            'uninstall:Uninstall an installed plugin'
            'remove:Uninstall an installed plugin'
            'update:Update a plugin to the latest version'
            'validate:Validate a plugin or marketplace manifest'
          )

          if (( CURRENT == 2 )); then
            _describe -t commands 'plugin commands' plugin_commands
          else
            case "$line[2]" in
              disable)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  '::plugin:'
                ;;
              enable)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              install|i)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              list)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              marketplace)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]'
                ;;
              uninstall|remove)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              update)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':plugin:'
                ;;
              validate)
                _arguments \
                  '(-h --help)'{-h,--help}'[Display help for command]' \
                  ':path:_directories'
                ;;
              *)
                _files
                ;;
            esac
          fi
          ;;

        setup-token)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        update|upgrade)
          _arguments \
            '(-h --help)'{-h,--help}'[Display help for command]'
          ;;

        *)
          _files
          ;;
      esac
      ;;

    tools)
      local -a tools_list
      tools_list=(
        'Bash'
        'Edit'
        'Read'
        'Write'
        'Glob'
        'Grep'
        'Agent'
        'Task'
        'WebFetch'
        'WebSearch'
        'Skill'
      )
      _describe -t tools 'available tools' tools_list
      ;;

    agents)
      local -a agents_list
      agents_list=(
        'default'
        'explore'
        'plan'
        'test'
        'review'
      )
      _describe -t agents 'available agents' agents_list
      ;;
  esac
}
CLAUDeOF

echo "✅ _claude 补齐文件已写入"

# 3. 检查并更新 .zshrc
update_zshrc() {
    if [[ -f "$ZSHRC" ]]; then
        # 检查是否已有 fpath 配置
        if grep -q "fpath=(~/.zsh/completions" "$ZSHRC" 2>/dev/null; then
            echo "✅ .zshrc 中已包含 fpath 配置"
            return 0
        else
            echo "📝 更新 $ZSHRC..."
            {
                echo ""
                echo "# Claude CLI 参数补齐"
                echo "fpath=(~/.zsh/completions \$fpath)"
                echo "autoload -Uz compinit"
                echo "compinit"
            } >> "$ZSHRC" 2>/dev/null || {
                echo "❌ 错误: 无法写入 $ZSHRC"
                echo "   请检查文件权限或磁盘空间"
                return 1
            }
            echo "✅ .zshrc 已更新"
        fi
    else
        echo "📝 创建 $ZSHRC..."
        {
            echo "# Claude CLI 参数补齐"
            echo "fpath=(~/.zsh/completions \$fpath)"
            echo "autoload -Uz compinit"
            echo "compinit"
        } > "$ZSHRC" 2>/dev/null || {
            echo "❌ 错误: 无法创建 $ZSHRC"
            echo "   请检查 $HOME 目录权限"
            return 1
        }
        echo "✅ $ZSHRC 已创建"
    fi
}

if ! update_zshrc; then
    exit 1
fi

echo ""
echo "=== 配置完成 ==="
echo ""
echo "🎉 完成! 现在可以使用 'claude --<Tab>' 进行参数补齐了"
echo ""
echo "请运行以下命令使配置生效："
echo "  source ~/.zshrc"
echo ""
echo "提示: 如果补齐不生效，请尝试运行: rm -f ~/.zcompdump && compinit"


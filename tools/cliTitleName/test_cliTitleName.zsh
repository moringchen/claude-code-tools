#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
COMMAND="$SCRIPT_DIR/titlename"
FUNCTION_IMPL="$SCRIPT_DIR/titlename.zsh"
INSTALLER="$SCRIPT_DIR/install.sh"

assert_eq() {
  local expected="$1"
  local actual="$2"
  local message="$3"

  if [[ "$actual" != "$expected" ]]; then
    print -u2 -- "$message"
    print -u2 -- "expected: $expected"
    print -u2 -- "actual:   $actual"
    exit 1
  fi
}

assert_true() {
  local condition="$1"
  local message="$2"

  if ! eval "$condition"; then
    print -u2 -- "$message"
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" != *"$needle"* ]]; then
    print -u2 -- "$message"
    print -u2 -- "missing: $needle"
    exit 1
  fi
}

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" == *"$needle"* ]]; then
    print -u2 -- "$message"
    print -u2 -- "unexpected: $needle"
    exit 1
  fi
}

run_capture() {
  local stdout_file stderr_file
  stdout_file=$(mktemp)
  stderr_file=$(mktemp)

  set +e
  "$@" >"$stdout_file" 2>"$stderr_file"
  RUN_STATUS=$?
  set -e

  RUN_STDOUT=$(<"$stdout_file")
  RUN_STDERR=$(<"$stderr_file")
  rm -f "$stdout_file" "$stderr_file"
}

typeset -g RUN_STATUS=0
TYPESSET_GUARD=1
unset TYPESSET_GUARD

typeset -g RUN_STDOUT=''
typeset -g RUN_STDERR=''
README_EN_CONTENTS=$(<"$SCRIPT_DIR/README.md")
README_CN_CONTENTS=$(<"$SCRIPT_DIR/README_CN.md")

assert_true '[[ -f "$COMMAND" ]]' 'titlename should exist'
assert_true '[[ -x "$COMMAND" ]]' 'titlename should be executable'
assert_true '[[ -f "$FUNCTION_IMPL" ]]' 'titlename.zsh should exist'

run_capture "$COMMAND" 'Claude Window'
assert_eq '0' "$RUN_STATUS" 'titlename should succeed with one non-empty argument'
assert_eq $'\e]2;Claude Window\a' "$RUN_STDOUT" 'titlename should emit the expected OSC title sequence'
assert_eq '' "$RUN_STDERR" 'titlename should not write stderr on success'

_ghostty_precmd() {
  builtin print -rnu 9 $'\e]133;A;cl=line\a'
  builtin print -rnu 9 $'\e]2;prompt title\a'
  builtin print -rnu 9 $'\e]133;C\a'
}

_ghostty_preexec() {
  builtin print -rnu 9 $'\e]2;command title\a'
  builtin print -rnu 9 $'\e]133;C\a'
}

TERM_PROGRAM=ghostty
source "$FUNCTION_IMPL"

run_capture titlename 'Claude Window'
assert_eq '0' "$RUN_STATUS" 'titlename function should succeed with one non-empty argument'
assert_eq $'\e]2;Claude Window\a' "$RUN_STDOUT" 'titlename function should emit the expected OSC title sequence'
assert_eq '' "$RUN_STDERR" 'titlename function should not write stderr on success'
assert_not_contains "${functions[_ghostty_precmd]}" '\e]2;' 'titlename should remove Ghostty title writes from _ghostty_precmd'
assert_not_contains "${functions[_ghostty_preexec]}" '\e]2;' 'titlename should remove Ghostty title writes from _ghostty_preexec'
assert_contains "${functions[_ghostty_precmd]}" '\e]133;A;cl=line\a' 'titlename should preserve Ghostty prompt-start marker in _ghostty_precmd'
assert_contains "${functions[_ghostty_precmd]}" '\e]133;C\a' 'titlename should preserve Ghostty prompt-end marker in _ghostty_precmd'
assert_contains "${functions[_ghostty_preexec]}" '\e]133;C\a' 'titlename should preserve Ghostty semantic marker in _ghostty_preexec'

run_capture "$COMMAND"
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail without arguments'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on usage failure'
assert_eq 'usage: titlename "Window Title"' "$RUN_STDERR" 'titlename should print the exact usage message without arguments'

run_capture "$COMMAND" ''
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail with an empty string argument'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on empty-string failure'
assert_eq 'usage: titlename "Window Title"' "$RUN_STDERR" 'titlename should print the exact usage message for an empty string argument'

fake_bin_dir=$(mktemp -d)
cat >"$fake_bin_dir/uname" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' 'Linux'
EOF
chmod +x "$fake_bin_dir/uname"

run_capture env PATH="$fake_bin_dir:$PATH" "$COMMAND" 'Claude Window'
assert_true '[[ "$RUN_STATUS" -ne 0 ]]' 'titlename should fail on non-macOS platforms'
assert_eq '' "$RUN_STDOUT" 'titlename should not write stdout on non-macOS failure'
assert_eq 'titlename supports macOS only' "$RUN_STDERR" 'titlename should print the exact macOS-only error'

installer_home=$(mktemp -d)
installed_path="$installer_home/.local/bin/titlename"
installed_function_path="$installer_home/.config/cliTitleName/titlename.zsh"
zshrc_path="$installer_home/.zshrc"
managed_block=$'# >>> cliTitleName >>>\n[[ -r "$HOME/.config/cliTitleName/titlename.zsh" ]] && source "$HOME/.config/cliTitleName/titlename.zsh"\n# <<< cliTitleName <<<'
expected_install_stdout=$'Installed fallback executable to '"$installed_path"$'\nInstalled zsh integration to '"$installed_function_path"$'\nManaged zshrc loader in '"$zshrc_path"$'\nAdd '"$installer_home/.local/bin"$' to your PATH if needed.'
print -r -- '# existing zshrc content' > "$zshrc_path"
run_capture env HOME="$installer_home" PATH="/usr/bin:/bin:/usr/sbin:/sbin" bash "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should succeed on macOS'
assert_true '[[ -f "$installed_path" ]]' 'install.sh should install titlename into ~/.local/bin'
assert_true '[[ -x "$installed_path" ]]' 'install.sh should make the installed titlename executable'
assert_true '[[ -f "$installed_function_path" ]]' 'install.sh should install titlename.zsh into ~/.config/cliTitleName'
assert_eq "$expected_install_stdout" "$RUN_STDOUT" 'install.sh should print installed paths and PATH hint'
assert_eq '' "$RUN_STDERR" 'install.sh should not write stderr on success'
expected_command_contents=$(<"$COMMAND")
installed_command_contents=$(<"$installed_path")
expected_function_contents=$(<"$FUNCTION_IMPL")
installed_function_contents=$(<"$installed_function_path")
zshrc_contents=$(<"$zshrc_path")
managed_block_count=$(grep -c '^# >>> cliTitleName >>>$' "$zshrc_path")
assert_eq "$expected_command_contents" "$installed_command_contents" 'install.sh should install the expected titlename script contents'
assert_eq "$expected_function_contents" "$installed_function_contents" 'install.sh should install the expected titlename.zsh contents'
assert_contains "$zshrc_contents" '# existing zshrc content' 'install.sh should preserve existing ~/.zshrc content'
assert_contains "$zshrc_contents" "$managed_block" 'install.sh should add the managed cliTitleName block to ~/.zshrc'
assert_eq '1' "$managed_block_count" 'install.sh should add exactly one managed block on first run'

run_capture env HOME="$installer_home" PATH="/usr/bin:/bin:/usr/sbin:/sbin" bash "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should be idempotent on a second run'
assert_eq "$expected_install_stdout" "$RUN_STDOUT" 'install.sh should report the same installed paths on a second run'
assert_eq '' "$RUN_STDERR" 'install.sh should not write stderr on the second run'
zshrc_contents=$(<"$zshrc_path")
managed_block_count=$(grep -c '^# >>> cliTitleName >>>$' "$zshrc_path")
assert_eq '1' "$managed_block_count" 'install.sh should not duplicate the managed block on a second run'
assert_contains "$zshrc_contents" "$managed_block" 'install.sh should keep the managed block intact on a second run'
assert_contains "$zshrc_contents" '# existing zshrc content' 'install.sh should preserve existing ~/.zshrc content on a second run'
assert_contains "$README_EN_CONTENTS" $'```sh\ncurl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash\nsource ~/.zshrc\n```' 'README.md should include source ~/.zshrc in the install snippet'
assert_contains "$README_EN_CONTENTS" 'Ghostty + interactive zsh' 'README.md should document the Ghostty + interactive zsh scope'
assert_contains "$README_EN_CONTENTS" 'disables later title rewrites for the current shell session' 'README.md should document session-scoped title disabling'
assert_contains "$README_EN_CONTENTS" 'does not change Ghostty globally' 'README.md should document that the change is not global'
assert_contains "$README_EN_CONTENTS" 'shell function from `~/.config/cliTitleName/titlename.zsh`' 'README.md should document the shell-function integration path'
assert_contains "$README_EN_CONTENTS" 'if Ghostty changes that internal integration, `titlename` falls back to one-shot behavior.' 'README.md should document the Ghostty hook-layout compatibility limitation'
assert_contains "$README_EN_CONTENTS" 'Outside Ghostty + interactive zsh, it behaves like a one-shot title setter only.' 'README.md should document one-shot fallback behavior outside Ghostty + interactive zsh'
assert_contains "$README_CN_CONTENTS" $'```sh\ncurl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/cliTitleName/install.sh | bash\nsource ~/.zshrc\n```' 'README_CN.md should include source ~/.zshrc in the install snippet'
assert_contains "$README_CN_CONTENTS" 'Ghostty + 交互式 zsh' 'README_CN.md should document the Ghostty + 交互式 zsh scope'
assert_contains "$README_CN_CONTENTS" '禁用当前 shell 会话后续的标题改写' 'README_CN.md should document session-scoped title disabling'
assert_contains "$README_CN_CONTENTS" '不会修改 Ghostty 的全局配置' 'README_CN.md should document that the change is not global'
assert_contains "$README_CN_CONTENTS" '从 `~/.config/cliTitleName/titlename.zsh` 加载 `titlename` shell function' 'README_CN.md should document the shell-function integration path'
assert_contains "$README_CN_CONTENTS" '如果 Ghostty 以后调整这层内部集成，`titlename` 会退回到一次性设置标题的行为。' 'README_CN.md should document the Ghostty hook-layout compatibility limitation'
assert_contains "$README_CN_CONTENTS" '在 Ghostty + 交互式 zsh 之外，它仍然只是一次性设置标题。' 'README_CN.md should document one-shot fallback behavior outside Ghostty + interactive zsh'

rm -rf "$fake_bin_dir" "$installer_home"

print 'ok'

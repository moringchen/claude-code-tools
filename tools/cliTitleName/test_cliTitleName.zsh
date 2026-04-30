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

functions[_ghostty_precmd]=$(cat <<'EOF'
	builtin local -i cmd_status=$?
	builtin emulate -L zsh -o no_warn_create_global -o no_aliases
	if ! builtin zle
	then
		if (( _ghostty_state == 1 ))
		then
			builtin print -nu $_ghostty_fd '\e]133;D;'$cmd_status'\a'
			(( _ghostty_state = 2 ))
		elif (( _ghostty_state == 2 ))
		then
			builtin print -nu $_ghostty_fd '\e]133;D\a'
		fi
	fi
	builtin local mark1=$'%{\e]133;A;cl=line\a%}'
	if [[ -o prompt_percent ]]
	then
		builtin typeset -g precmd_functions
		if [[ ${precmd_functions[-1]} == _ghostty_precmd ]]
		then
			builtin local ps1_changed=0
			if [[ -n ${_ghostty_saved_ps1+x} ]]
			then
				if [[ $PS1 == $_ghostty_marked_ps1 ]]
				then
					PS1=$_ghostty_saved_ps1
					PS2=$_ghostty_saved_ps2
				elif [[ $PS1 != $_ghostty_saved_ps1 ]]
				then
					ps1_changed=1
				fi
			fi
			_ghostty_saved_ps1=$PS1
			_ghostty_saved_ps2=$PS2
			builtin local mark2=$'%{\e]133;P;k=s\a%}'
			builtin local markB=$'%{\e]133;B\a%}'
			[[ $PS1 == *[^%]% || $PS1 == % ]] && PS1=$PS1%
			PS1=${mark1}${PS1}${markB}
			if (( ! ps1_changed )) && [[ $PS1 == *$'\n'* ]]
			then
				PS1=${PS1//$'\n'/$'\n'${mark2}}
			fi
			[[ $PS2 == *[^%]% || $PS2 == % ]] && PS2=$PS2%
			PS2=${mark2}${PS2}${markB}
			_ghostty_marked_ps1=$PS1
			(( _ghostty_state = 2 ))
		else
			precmd_functions=(${precmd_functions:#_ghostty_precmd} _ghostty_precmd)
			if ! builtin zle
			then
				builtin print -rnu $_ghostty_fd -- $mark1[3,-3]
				(( _ghostty_state = 2 ))
			fi
		fi
	elif ! builtin zle
	then
		builtin print -rnu $_ghostty_fd -- $mark1[3,-3]
		(( _ghostty_state = 2 ))
	fi
	_ghostty_report_pwd
	builtin print -rnu 1 '\e]2;prompt title\a'
EOF
)
functions[_ghostty_preexec]=$(cat <<'EOF'
	builtin emulate -L zsh -o no_warn_create_global -o no_aliases
	if [[ -n ${_ghostty_saved_ps1+x} && $PS1 == $_ghostty_marked_ps1 ]]
	then
		PS1=$_ghostty_saved_ps1
		PS2=$_ghostty_saved_ps2
	fi
	builtin print -nu $_ghostty_fd '\e]133;C\a'
	(( _ghostty_state = 1 ))
	builtin print -rnu 1 '\e]2;command title\a'
EOF
)
_ghostty_report_pwd() {
  builtin true
}
_ghostty_fd=1
_ghostty_state=0
precmd_functions=(_ghostty_precmd)
PS1='%# '
PS2='> '
_ghostty_saved_ps1=$PS1
_ghostty_saved_ps2=$PS2
_ghostty_marked_ps1=$PS1

run_capture titlename 'Claude Window'
assert_eq '0' "$RUN_STATUS" 'titlename should succeed when patching Ghostty-style multiline hook bodies'
assert_eq $'\e]2;Claude Window\a' "$RUN_STDOUT" 'titlename should still emit the expected OSC title sequence for Ghostty-style hooks'
assert_eq '' "$RUN_STDERR" 'titlename should not emit parse errors when patching Ghostty-style multiline hook bodies'
assert_not_contains "${functions[_ghostty_precmd]}" '\e]2;' 'titlename should remove title writes from Ghostty-style _ghostty_precmd bodies'
assert_not_contains "${functions[_ghostty_preexec]}" '\e]2;' 'titlename should remove title writes from Ghostty-style _ghostty_preexec bodies'
assert_contains "${functions[_ghostty_preexec]}" '_ghostty_marked_ps1' 'titlename should preserve Ghostty-style multiline preexec conditions'
assert_contains "${functions[_ghostty_precmd]}" 'ps1_changed' 'titlename should preserve Ghostty-style multiline precmd conditions'
assert_contains "${functions[_ghostty_precmd]}" '_ghostty_report_pwd' 'titlename should preserve non-title Ghostty precmd behavior'

wrapper_bin_dir=$(mktemp -d)
cat >"$wrapper_bin_dir/claude" <<'EOF'
#!/usr/bin/env bash
printf 'env=%s\n' "${CLAUDE_CODE_DISABLE_TERMINAL_TITLE-unset}"
printf 'argc=%s\n' "$#"
i=1
for arg in "$@"; do
  printf 'arg%s_start\n%s\narg%s_end\n' "$i" "$arg" "$i"
  i=$((i + 1))
done
EOF
chmod +x "$wrapper_bin_dir/claude"

original_path="$PATH"
PATH="$wrapper_bin_dir:$PATH"
unset CLAUDE_CODE_DISABLE_TERMINAL_TITLE || true

run_capture claude '--model' 'alpha beta' '' $'line1
line2'
assert_eq '0' "$RUN_STATUS" 'claude wrapper should delegate successfully'
assert_contains "$RUN_STDOUT" 'env=1' 'claude wrapper should inject CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1'
assert_contains "$RUN_STDOUT" 'argc=4' 'claude wrapper should preserve argument count'
assert_contains "$RUN_STDOUT" $'arg1_start\n--model\narg1_end' 'claude wrapper should preserve the first argument exactly'
assert_contains "$RUN_STDOUT" $'arg2_start\nalpha beta\narg2_end' 'claude wrapper should preserve spaces inside arguments'
assert_contains "$RUN_STDOUT" $'arg3_start\n\narg3_end' 'claude wrapper should preserve empty string arguments'
assert_contains "$RUN_STDOUT" $'arg4_start\nline1\nline2\narg4_end' 'claude wrapper should preserve embedded newlines inside arguments'
assert_eq '' "$RUN_STDERR" 'claude wrapper should not write stderr when executable exists'
assert_true '[[ -z "${CLAUDE_CODE_DISABLE_TERMINAL_TITLE-}" ]]' 'claude wrapper should not leak the disable flag into the current shell'

PATH="/usr/bin:/bin:/usr/sbin:/sbin"
run_capture claude --version
assert_eq '127' "$RUN_STATUS" 'claude wrapper should return 127 when the executable is missing'
assert_eq '' "$RUN_STDOUT" 'claude wrapper should not write stdout when executable is missing'
assert_eq 'cliTitleName: claude executable not found in PATH' "$RUN_STDERR" 'claude wrapper should explain when claude is missing'
PATH="$original_path"

rm -rf "$wrapper_bin_dir"

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

stdin_home=$(mktemp -d)
stdin_installed_path="$stdin_home/.local/bin/titlename"
stdin_installed_function_path="$stdin_home/.config/cliTitleName/titlename.zsh"
stdin_zshrc_path="$stdin_home/.zshrc"
stdin_expected_install_stdout=$'Installed fallback executable to '"$stdin_installed_path"$'\nInstalled zsh integration to '"$stdin_installed_function_path"$'\nManaged zshrc loader in '"$stdin_zshrc_path"$'\nAdd '"$stdin_home/.local/bin"$' to your PATH if needed.'
print -r -- '# stdin zshrc content' > "$stdin_zshrc_path"
run_capture env HOME="$stdin_home" PATH="/usr/bin:/bin:/usr/sbin:/sbin" bash -c 'bash < "$1"' _ "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should succeed when piped to bash via stdin'
assert_true '[[ -f "$stdin_installed_path" ]]' 'stdin install should install titlename into ~/.local/bin'
assert_true '[[ -x "$stdin_installed_path" ]]' 'stdin install should make the installed titlename executable'
assert_true '[[ -f "$stdin_installed_function_path" ]]' 'stdin install should install titlename.zsh into ~/.config/cliTitleName'
assert_eq "$stdin_expected_install_stdout" "$RUN_STDOUT" 'stdin install should print installed paths and PATH hint'
assert_eq '' "$RUN_STDERR" 'stdin install should not write stderr on success'
stdin_installed_command_contents=$(<"$stdin_installed_path")
stdin_installed_function_contents=$(<"$stdin_installed_function_path")
stdin_zshrc_contents=$(<"$stdin_zshrc_path")
stdin_managed_block_count=$(grep -c '^# >>> cliTitleName >>>$' "$stdin_zshrc_path")
assert_eq "$expected_command_contents" "$stdin_installed_command_contents" 'stdin install should install the expected titlename script contents'
assert_eq "$expected_function_contents" "$stdin_installed_function_contents" 'stdin install should install the expected titlename.zsh contents'
assert_contains "$stdin_zshrc_contents" '# stdin zshrc content' 'stdin install should preserve existing ~/.zshrc content'
assert_contains "$stdin_zshrc_contents" "$managed_block" 'stdin install should add the managed cliTitleName block to ~/.zshrc'
assert_eq '1' "$stdin_managed_block_count" 'stdin install should add exactly one managed block'

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
assert_contains "$README_EN_CONTENTS" 'interactive zsh on macOS' 'README.md should document the interactive zsh on macOS scope'
assert_contains "$README_EN_CONTENTS" 'strips Ghostty'"'"'s current zsh title-write lines' 'README.md should document Ghostty hook patching behavior'
assert_contains "$README_EN_CONTENTS" 'installs a `claude()` shell wrapper' 'README.md should document the automatic claude wrapper'
assert_contains "$README_EN_CONTENTS" 'CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1' 'README.md should document the Claude Code title-disable env var'
assert_contains "$README_EN_CONTENTS" 'It does not modify Claude Code source.' 'README.md should document that Claude Code source is not modified'
assert_contains "$README_EN_CONTENTS" 'expects the real `claude` executable to be available on `PATH`' 'README.md should document the PATH requirement'
assert_contains "$README_EN_CONTENTS" 'whichever definition is loaded later wins' 'README.md should document wrapper conflict resolution'
assert_contains "$README_EN_CONTENTS" 'shell function from `~/.config/cliTitleName/titlename.zsh`' 'README.md should document the shell-function integration path'
assert_contains "$README_CN_CONTENTS" 'macOS 的交互式 zsh' 'README_CN.md should document the macOS interactive zsh scope'
assert_contains "$README_CN_CONTENTS" '把当前 shell hook 里负责写标题的那几行去掉' 'README_CN.md should document Ghostty hook patching behavior'
assert_contains "$README_CN_CONTENTS" '安装一个 `claude()` shell wrapper' 'README_CN.md should document the automatic claude wrapper'
assert_contains "$README_CN_CONTENTS" 'CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1' 'README_CN.md should document the Claude Code title-disable env var'
assert_contains "$README_CN_CONTENTS" '它不会修改 Claude Code 源码。' 'README_CN.md should document that Claude Code source is not modified'
assert_contains "$README_CN_CONTENTS" '真正的 `claude` 可执行文件存在于 `PATH` 中' 'README_CN.md should document the PATH requirement'
assert_contains "$README_CN_CONTENTS" '后加载的那个定义会生效' 'README_CN.md should document wrapper conflict resolution'
assert_contains "$README_CN_CONTENTS" '从 `~/.config/cliTitleName/titlename.zsh` 加载 `titlename` shell function' 'README_CN.md should document the shell-function integration path'

rm -rf "$fake_bin_dir" "$installer_home"

print 'ok'

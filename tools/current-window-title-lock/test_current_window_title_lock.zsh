#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}

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

_ghostty_precmd() { : }
_ghostty_preexec() { : }
precmd_functions=(_ghostty_precmd)
preexec_functions=(_ghostty_preexec)

claude() {
  print "original claude"
}

source "$SCRIPT_DIR/current_window_title_lock.zsh"

functions lock-title >/dev/null
functions unlock-title >/dev/null
functions title-status >/dev/null

local original_claude_function=${functions[claude]}
local first_lock_output
local second_lock_output
local first_lock_output_file
local second_lock_output_file

first_lock_output_file=$(mktemp)
lock-title "Pinned A" >"$first_lock_output_file"
first_lock_output=$(<"$first_lock_output_file")
rm -f "$first_lock_output_file"

assert_eq $'\e]2;Pinned A\a' "$first_lock_output" "lock-title should emit title sequence for Pinned A"
assert_eq "Pinned A" "$TITLE_LOCK_VALUE" "lock-title should set TITLE_LOCK_VALUE"
assert_true '[[ ${#${(M)precmd_functions:#_ghostty_precmd}} -eq 0 ]]' "lock-title should remove _ghostty_precmd"
assert_true '[[ ${#${(M)preexec_functions:#_ghostty_preexec}} -eq 0 ]]' "lock-title should remove _ghostty_preexec"
assert_true '[[ ${+functions[claude]} -eq 1 ]]' "lock-title should install claude wrapper"
assert_eq 'CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1 command claude "$@"' "${${functions[claude]-}#$'\t'}" "lock-title should install expected claude wrapper"
assert_eq "locked: Pinned A" "$(title-status)" "title-status should report first locked state"

second_lock_output_file=$(mktemp)
lock-title "Pinned B" >"$second_lock_output_file"
second_lock_output=$(<"$second_lock_output_file")
rm -f "$second_lock_output_file"

assert_eq $'\e]2;Pinned B\a' "$second_lock_output" "second lock-title should emit title sequence for Pinned B"
assert_eq "Pinned B" "$TITLE_LOCK_VALUE" "second lock-title should update TITLE_LOCK_VALUE"
assert_eq "locked: Pinned B" "$(title-status)" "title-status should report second locked state"
assert_true '[[ ${#${(M)precmd_functions:#_ghostty_precmd}} -eq 0 ]]' "second lock-title should keep _ghostty_precmd removed"
assert_true '[[ ${#${(M)preexec_functions:#_ghostty_preexec}} -eq 0 ]]' "second lock-title should keep _ghostty_preexec removed"

unlock-title
unlock-title

assert_eq "" "$TITLE_LOCK_VALUE" "double unlock should clear TITLE_LOCK_VALUE"
assert_true '[[ ${#${(M)precmd_functions:#_ghostty_precmd}} -eq 1 ]]' "double unlock should restore _ghostty_precmd"
assert_true '[[ ${#${(M)preexec_functions:#_ghostty_preexec}} -eq 1 ]]' "double unlock should restore _ghostty_preexec"
assert_eq "unlocked" "$(title-status)" "title-status should report unlocked state after double unlock"
assert_true '[[ ${+functions[claude]} -eq 1 ]]' "double unlock should restore original claude function"
assert_eq "$original_claude_function" "${functions[claude]}" "double unlock should restore original claude function body"

print "ok"

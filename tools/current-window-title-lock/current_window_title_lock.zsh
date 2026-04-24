typeset -g TITLE_LOCK_VALUE="${TITLE_LOCK_VALUE-}"
typeset -g TITLE_LOCK_REMOVED_GHOSTTY_PRECMD="${TITLE_LOCK_REMOVED_GHOSTTY_PRECMD-0}"
typeset -g TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC="${TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC-0}"
typeset -g TITLE_LOCK_HAD_CLAUDE_FUNCTION="${TITLE_LOCK_HAD_CLAUDE_FUNCTION-0}"
typeset -g TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION="${TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION-}"

_title_lock_write_title() {
  printf '\e]2;%s\a' "$1"
}

_title_lock_remove_hook() {
  local array_name="$1"
  local hook_name="$2"

  typeset -ga "$array_name"
  eval "$array_name=(\${${array_name}:#$hook_name})"
}

_title_lock_restore_hook() {
  local array_name="$1"
  local hook_name="$2"

  (( ${+functions[$hook_name]} )) || return 0
  typeset -ga "$array_name"
  eval '[[ " ${'"$array_name"'[*]} " == *" '"$hook_name"' "* ]] || '"$array_name"'+=('"$hook_name"')'
}

lock-title() {
  local was_locked=0
  [[ -n "$TITLE_LOCK_VALUE" ]] && was_locked=1
  TITLE_LOCK_VALUE="$1"

  if [[ " ${precmd_functions[*]-} " == *" _ghostty_precmd "* ]]; then
    _title_lock_remove_hook precmd_functions _ghostty_precmd
    TITLE_LOCK_REMOVED_GHOSTTY_PRECMD=1
  fi

  if [[ " ${preexec_functions[*]-} " == *" _ghostty_preexec "* ]]; then
    _title_lock_remove_hook preexec_functions _ghostty_preexec
    TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC=1
  fi

  if (( ! was_locked )); then
    if (( ${+functions[claude]} )); then
      TITLE_LOCK_HAD_CLAUDE_FUNCTION=1
      TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION=${functions[claude]}
    else
      TITLE_LOCK_HAD_CLAUDE_FUNCTION=0
      TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION=''
    fi
  fi

  functions[claude]='CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1 command claude "$@"'
  _title_lock_write_title "$TITLE_LOCK_VALUE"
}

unlock-title() {
  if [[ -z "$TITLE_LOCK_VALUE" ]] && (( ! TITLE_LOCK_REMOVED_GHOSTTY_PRECMD )) && (( ! TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC )) && (( ! TITLE_LOCK_HAD_CLAUDE_FUNCTION )) && [[ -z "$TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION" ]]; then
    return 0
  fi

  TITLE_LOCK_VALUE=""

  if (( TITLE_LOCK_REMOVED_GHOSTTY_PRECMD )); then
    _title_lock_restore_hook precmd_functions _ghostty_precmd
  fi

  if (( TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC )); then
    _title_lock_restore_hook preexec_functions _ghostty_preexec
  fi

  TITLE_LOCK_REMOVED_GHOSTTY_PRECMD=0
  TITLE_LOCK_REMOVED_GHOSTTY_PREEXEC=0

  if (( TITLE_LOCK_HAD_CLAUDE_FUNCTION )); then
    functions[claude]="$TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION"
  else
    unfunction claude 2>/dev/null || true
  fi

  TITLE_LOCK_HAD_CLAUDE_FUNCTION=0
  TITLE_LOCK_ORIGINAL_CLAUDE_FUNCTION=''
}

title-status() {
  if [[ -n "$TITLE_LOCK_VALUE" ]]; then
    print "locked: $TITLE_LOCK_VALUE"
  else
    print "unlocked"
  fi
}

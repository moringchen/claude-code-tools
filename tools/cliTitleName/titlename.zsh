# Ghostty currently injects OSC 2 title writes directly into these zsh hook
# functions, so we patch the active session hooks instead of changing config.
_titlename_strip_ghostty_title_lines() {
  local function_name="$1"
  local function_body filtered_lines line

  (( ${+functions[$function_name]} )) || return 0

  function_body=${functions[$function_name]}
  filtered_lines=()

  for line in ${(f)function_body}; do
    [[ "$line" == *$'\e]2;'* ]] && continue
    [[ "$line" == *'\e]2;'* ]] && continue
    filtered_lines+=("$line")
  done

  functions[$function_name]="${(F)filtered_lines}"
}

_titlename_disable_ghostty_title_rewrites() {
  _titlename_strip_ghostty_title_lines _ghostty_precmd
  _titlename_strip_ghostty_title_lines _ghostty_preexec
}

claude() {
  local claude_path
  claude_path=$(whence -p claude 2>/dev/null || true)

  if [[ -z "$claude_path" ]]; then
    printf '%s\n' 'cliTitleName: claude executable not found in PATH' >&2
    return 127
  fi

  CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1 command "$claude_path" "$@"
}

titlename() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    printf '%s\n' 'titlename supports macOS only' >&2
    return 1
  fi

  if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
    printf '%s\n' 'usage: titlename "Window Title"' >&2
    return 1
  fi

  if [[ "${TERM_PROGRAM-}" == "ghostty" ]]; then
    _titlename_disable_ghostty_title_rewrites
  fi

  printf '\033]2;%s\a' "$1"
}

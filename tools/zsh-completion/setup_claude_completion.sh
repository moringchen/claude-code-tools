#!/usr/bin/env bash

set -euo pipefail

COMPLETIONS_DIR="$HOME/.zsh/completions"
COMPLETION_FILE="$COMPLETIONS_DIR/_claude"
ZSHRC_PATH="$HOME/.zshrc"
BLOCK_START='# >>> Claude CLI zsh completion >>>'
BLOCK_END='# <<< Claude CLI zsh completion <<<'

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

require_claude() {
  command -v claude >/dev/null 2>&1 || fail 'claude was not found in PATH'
}

capture_help() {
  claude --help || fail 'failed to run claude --help'
}

ensure_completions_dir() {
  mkdir -p "$COMPLETIONS_DIR"
  [[ -w "$COMPLETIONS_DIR" ]] || fail "directory is not writable: $COMPLETIONS_DIR"
}

generate_completion_file() {
  local help_text
  help_text="$(capture_help)"
  HELPTEXT="$help_text" python3 - <<'PY' > "$COMPLETION_FILE"
import os
import re

help_text = os.environ['HELPTEXT']
lines = help_text.splitlines()
section = None
options = []
commands = []

for line in lines:
    if line == 'Options:':
        section = 'options'
        continue
    if line == 'Commands:':
        section = 'commands'
        continue
    if not line.startswith('  '):
        continue
    stripped = line.strip()
    if not stripped:
        continue
    parts = re.split(r'\s{2,}', stripped, maxsplit=1)
    if len(parts) != 2:
        continue
    raw, description = parts
    if section == 'options' and raw.startswith('-'):
        options.append({'raw': raw, 'description': description})
    elif section == 'commands':
        commands.append({'raw': raw, 'name': raw.split()[0], 'description': description})

if not options or not commands:
    raise SystemExit('failed to parse enough data from claude --help')


def split_names(raw: str):
    m = re.match(r'(.+?)(\s+(<[^>]+>|\[[^]]+\]))?$', raw)
    flags = m.group(1)
    argument = m.group(3)
    return [part.strip() for part in flags.split(', ')], argument


def extract_choices(argument: str | None, description: str):
    if argument in {'<mode>', '<format>'}:
        return re.findall(r'"([^"]+)"', description)
    if argument == '<level>':
        match = re.search(r'\(([^)]*)\)', description)
        if match:
            return [part.strip() for part in match.group(1).split(',')]
    return []


def escape(text: str):
    return text.replace('\\', '\\\\').replace("'", "'\\''")


def render_option(option):
    names, argument = split_names(option['raw'])
    description = escape(option['description'])
    choices = extract_choices(argument, option['description'])

    if len(names) == 2 and names[0].startswith('-') and names[1].startswith('--'):
        prefix = f"'({names[0]} {names[1]})'{{{names[0]},{names[1]}}}"
    else:
        prefix = ' '.join(f"'{name}'" for name in names)

    if argument == '<mode>' and choices:
        return f"{prefix}[{description}]:mode:({' '.join(choices)})"
    if argument == '<level>' and choices:
        return f"{prefix}[{description}]:level:({' '.join(choices)})"
    if argument == '<format>' and choices:
        return f"{prefix}[{description}]:format:({' '.join(choices)})"
    if argument:
        label = argument.strip('<>[]').split(':', 1)[0]
        return f"{prefix}[{description}]:{label}:"
    return f"{prefix}[{description}]"


print('#compdef claude')
print()
print('_claude() {')
print('  local curcontext="$curcontext" state line')
print('  typeset -A opt_args')
print('  local -a global_options commands')
print()
print('  global_options=(')
for option in options:
    print(f'    {render_option(option)}')
print('  )')
print()
print('  commands=(')
for command in commands:
    print(f"    '{escape(command['name'])}:{escape(command['description'])}'")
print('  )')
print()
print('  _arguments -C \\\n    $global_options \\\n    ": :->command" \\\n    "*:: :->args"')
print()
print('  case "$state" in')
print('    command)')
print("      _describe -t commands 'claude commands' commands")
print('      ;;')
print('  esac')
print('}')
PY
}

completion_block() {
  cat <<'EOF'
# >>> Claude CLI zsh completion >>>
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
if [[ -z ${_CLAUDE_COMPLETION_COMPINIT_DONE:-} ]]; then
  compinit
  _CLAUDE_COMPLETION_COMPINIT_DONE=1
fi
zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'
# <<< Claude CLI zsh completion <<<
EOF
}

update_zshrc_block() {
  local block
  block="$(completion_block)"

  if [[ -f "$ZSHRC_PATH" ]]; then
    python3 - <<'PY' "$ZSHRC_PATH" "$BLOCK_START" "$BLOCK_END" "$block"
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
start = sys.argv[2]
end = sys.argv[3]
block = sys.argv[4]
text = path.read_text(encoding='utf-8')

if start in text and end in text:
    before, rest = text.split(start, 1)
    _, after = rest.split(end, 1)
    new_text = before.rstrip() + '\n\n' + block + '\n' + after.lstrip('\n')
else:
    new_text = text.rstrip() + '\n\n' + block + '\n' if text.strip() else block + '\n'

path.write_text(new_text, encoding='utf-8')
PY
  else
    printf '%s\n' "$block" > "$ZSHRC_PATH"
  fi
}

main() {
  require_claude
  ensure_completions_dir
  generate_completion_file
  update_zshrc_block
  printf '%s\n' 'Claude completion installation complete.'
  printf '%s\n' 'Run: source ~/.zshrc'
}

main "$@"

#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${HOME}/.local/bin"
INSTALL_PATH="$INSTALL_DIR/titlename"
ZSH_INTEGRATION_DIR="${HOME}/.config/cliTitleName"
ZSH_INTEGRATION_PATH="$ZSH_INTEGRATION_DIR/titlename.zsh"
ZSHRC_PATH="${HOME}/.zshrc"
BLOCK_START='# >>> cliTitleName >>>'
BLOCK_LINE='[[ -r "$HOME/.config/cliTitleName/titlename.zsh" ]] && source "$HOME/.config/cliTitleName/titlename.zsh"'
BLOCK_END='# <<< cliTitleName <<<'

mkdir -p "$INSTALL_DIR" "$ZSH_INTEGRATION_DIR"
cat >"$INSTALL_PATH" <<'EOF'
#!/usr/bin/env bash

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '%s\n' 'titlename supports macOS only' >&2
  exit 1
fi

if [[ $# -ne 1 ]] || [[ -z "$1" ]]; then
  printf '%s\n' 'usage: titlename "Window Title"' >&2
  exit 1
fi

printf '\033]2;%s\a' "$1"
EOF
chmod +x "$INSTALL_PATH"
cp "$SCRIPT_DIR/titlename.zsh" "$ZSH_INTEGRATION_PATH"
touch "$ZSHRC_PATH"

tmp_zshrc=$(mktemp)
skip_block=0
while IFS= read -r line || [[ -n "$line" ]]; do
  if [[ $skip_block -eq 0 && "$line" == "$BLOCK_START" ]]; then
    skip_block=1
    continue
  fi

  if [[ $skip_block -eq 1 ]]; then
    if [[ "$line" == "$BLOCK_END" ]]; then
      skip_block=0
    fi
    continue
  fi

  printf '%s\n' "$line" >>"$tmp_zshrc"
done <"$ZSHRC_PATH"

if [[ -s "$tmp_zshrc" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '%s\n' "$line"
  done <"$tmp_zshrc" >"$ZSHRC_PATH"
  printf '\n%s\n%s\n%s\n' "$BLOCK_START" "$BLOCK_LINE" "$BLOCK_END" >>"$ZSHRC_PATH"
else
  printf '%s\n%s\n%s\n' "$BLOCK_START" "$BLOCK_LINE" "$BLOCK_END" >"$ZSHRC_PATH"
fi

rm -f "$tmp_zshrc"

if command -v zsh >/dev/null 2>&1; then
  HOME="$HOME" zsh -i -c "source \"$ZSHRC_PATH\"" >/dev/null 2>&1 || true
fi

printf 'Installed fallback executable to %s\n' "$INSTALL_PATH"
printf 'Installed zsh integration to %s\n' "$ZSH_INTEGRATION_PATH"
printf 'Managed zshrc loader in %s\n' "$ZSHRC_PATH"

case ":${PATH:-}:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'Add %s to your PATH if needed.\n' "$INSTALL_DIR" ;;
esac

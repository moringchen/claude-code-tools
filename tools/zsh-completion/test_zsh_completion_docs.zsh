#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
README_EN_CONTENTS=$(<"$SCRIPT_DIR/README.md")
README_CN_CONTENTS=$(<"$SCRIPT_DIR/README_CN.md")

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

assert_contains "$README_EN_CONTENTS" $'```bash\ncurl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash\nsource ~/.zshrc\n```' 'README.md should include the curl-based install snippet'
assert_contains "$README_CN_CONTENTS" $'```bash\ncurl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/zsh-completion/setup_claude_completion.sh | bash\nsource ~/.zshrc\n```' 'README_CN.md should include the curl-based install snippet'

print 'ok'

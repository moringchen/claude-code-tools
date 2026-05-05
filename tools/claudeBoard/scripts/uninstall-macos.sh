#!/bin/sh
set -eu

TARGET_DIR="$HOME/.claude-board"
TARGET_SCRIPT="$TARGET_DIR/hook-dispatch.sh"

if [ -f "$TARGET_SCRIPT" ]; then
  rm -f "$TARGET_SCRIPT"
fi

printf 'Removed claudeBoard hook wrapper at %s\n' "$TARGET_SCRIPT"
printf 'Buffered events in %s were left intact. Remove that directory manually if you want a full purge.\n' "$TARGET_DIR"
printf 'Remove the Claude Code hook command that referenced hook-dispatch.sh if it is still configured.\n'

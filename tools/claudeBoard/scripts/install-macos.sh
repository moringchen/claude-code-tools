#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
TARGET_DIR="$HOME/.claude-board"
TARGET_SCRIPT="$TARGET_DIR/hook-dispatch.sh"

mkdir -p "$TARGET_DIR"
cp "$SCRIPT_DIR/hook-dispatch.sh" "$TARGET_SCRIPT"
chmod +x "$TARGET_SCRIPT"

printf 'Installed claudeBoard hook wrapper at %s\n' "$TARGET_SCRIPT"
printf 'Use this command in Claude Code hook settings: %s\n' "$TARGET_SCRIPT"

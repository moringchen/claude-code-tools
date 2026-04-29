#!/bin/sh
set -eu

payload=$(cat)
url="http://127.0.0.1:46123/events"
buffer_path="${CLAUDE_TASK_WINDOW_BUFFER_PATH:-$HOME/.claude-task-window/events.jsonl}"

if command -v curl >/dev/null 2>&1; then
  if printf '%s' "$payload" | curl -fsS -H "content-type: application/json" --data-binary @- "$url" >/dev/null 2>&1; then
    exit 0
  fi
fi

mkdir -p "$(dirname "$buffer_path")"
printf '%s\n' "$payload" >> "$buffer_path"

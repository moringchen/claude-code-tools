#!/bin/sh
set -eu

payload=$(cat)
url="http://127.0.0.1:46123/events"
buffer_path="${CLAUDE_BOARD_BUFFER_PATH:-$HOME/.claude-board/events.jsonl}"
occurred_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
claude_pid="${PPID:-0}"
claude_title=$(ps -p "$claude_pid" -o command= 2>/dev/null | head -n 1 | tr -d '\n')

if command -v python3 >/dev/null 2>&1; then
  payload=$(CLAUDE_BOARD_PAYLOAD="$payload" CLAUDE_BOARD_PID="$claude_pid" CLAUDE_BOARD_TITLE="$claude_title" CLAUDE_BOARD_OCCURRED_AT="$occurred_at" python3 <<'PY'
import json
import os

payload = json.loads(os.environ["CLAUDE_BOARD_PAYLOAD"])
payload["claude_board_pid"] = int(os.environ.get("CLAUDE_BOARD_PID", "0") or 0)
payload["claude_board_title"] = os.environ.get("CLAUDE_BOARD_TITLE", "")
payload["claude_board_occurred_at"] = os.environ["CLAUDE_BOARD_OCCURRED_AT"]
print(json.dumps(payload, ensure_ascii=False))
PY
)
fi

if command -v curl >/dev/null 2>&1; then
  if printf '%s' "$payload" | curl -fsS -H "content-type: application/json" --data-binary @- "$url" >/dev/null 2>&1; then
    exit 0
  fi
fi

mkdir -p "$(dirname "$buffer_path")"
printf '%s\n' "$payload" >> "$buffer_path"

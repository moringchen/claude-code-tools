# claudeBoard

claudeBoard is a desktop overlay for Claude Code task activity. It keeps an always-visible top overlay on screen so you can see task state at a glance and jump back to the right terminal when action is needed.

## Supported Platforms

- macOS Apple Silicon
- macOS Intel
- Windows 10
- Windows 11

## Features

- User-level global hooks integration so task events are captured from your Claude Code setup without per-project wiring.
- Always-visible top overlay that shows 当前无任务 when nothing is running.
- Live counts for total, needs-user, completed, and running tasks.
- Click-to-focus behavior that brings the matching terminal window, tab, or pane to the front.
- Desktop notification toggles for completed and needs-user tasks.
- Voice announcement toggles for completed and needs-user tasks.

## Installation

```bash
cd tools/claudeBoard
npm install
cargo fetch --manifest-path src-tauri/Cargo.toml
```

## Development

```bash
cd tools/claudeBoard
npm run dev
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

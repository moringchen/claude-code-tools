# claudeBoard

claudeBoard 是一个面向 Claude Code 任务活动的桌面悬浮层工具。它会在屏幕顶部保持常驻显示，方便你随时查看任务状态，并在需要时快速回到对应终端。

## 支持平台

- macOS Apple Silicon
- macOS Intel
- Windows 10
- Windows 11

## 功能特性

- 支持用户级全局 hooks 集成，无需在每个项目里重复配置即可接收 Claude Code 任务事件。
- 始终置顶的顶部悬浮条，在没有任务时显示 当前无任务。
- 实时展示总数、需确认、已完成、运行中等任务统计。
- 点击即可聚焦到对应的终端窗口、标签页或窗格。
- 可分别开关桌面通知，用于已完成和需要确认的任务。
- 可分别开关语音播报，用于已完成和需要确认的任务。

## 安装

```bash
cd tools/claudeBoard
npm install
cargo fetch --manifest-path src-tauri/Cargo.toml
```

## 开发

```bash
cd tools/claudeBoard
npm run dev
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

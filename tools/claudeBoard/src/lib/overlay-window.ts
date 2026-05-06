import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/window";

export const COLLAPSED_OVERLAY_SIZE = {
  width: 260,
  height: 64,
} as const;

export const EXPANDED_OVERLAY_SIZE = {
  width: 260,
  height: 320,
} as const;

// Task row height (44px) + gap (6px) = ~50px per task, plus header (44px) and padding (20px)
const HEADER_HEIGHT = 44;
const TASK_ROW_HEIGHT = 50;
const PADDING = 20;

export function getOverlayWindowSize(expanded: boolean, taskCount = 0) {
  if (!expanded) {
    return COLLAPSED_OVERLAY_SIZE;
  }
  // Calculate dynamic height based on task count
  const contentHeight = HEADER_HEIGHT + (taskCount * TASK_ROW_HEIGHT) + PADDING;
  const height = Math.min(Math.max(contentHeight, 100), EXPANDED_OVERLAY_SIZE.height);
  return { width: EXPANDED_OVERLAY_SIZE.width, height };
}

export function createOverlayLogicalSize(expanded: boolean, taskCount = 0) {
  const { width, height } = getOverlayWindowSize(expanded, taskCount);
  return new LogicalSize(width, height);
}

export function requestOverlayRaise() {
  return invoke("request_overlay_raise");
}

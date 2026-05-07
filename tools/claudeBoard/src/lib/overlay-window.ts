import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/window";

export const COLLAPSED_OVERLAY_SIZE = {
  width: 260,
  height: 64,
} as const;

export const COMPACT_OVERLAY_SIZE = {
  width: 120,
  height: 48,
} as const;

export const EXPANDED_OVERLAY_SIZE = {
  width: 260,
  height: 320,
} as const;

export type OverlayMode = "collapsed" | "expanded" | "compact";

const HEADER_HEIGHT = 44;
const TASK_ROW_HEIGHT = 50;
const PADDING = 20;

export function getOverlayWindowSize(mode: OverlayMode, taskCount = 0) {
  if (mode === "collapsed") {
    return COLLAPSED_OVERLAY_SIZE;
  }

  if (mode === "compact") {
    return COMPACT_OVERLAY_SIZE;
  }

  const contentHeight = HEADER_HEIGHT + (taskCount * TASK_ROW_HEIGHT) + PADDING;
  const height = Math.min(Math.max(contentHeight, 100), EXPANDED_OVERLAY_SIZE.height);
  return { width: EXPANDED_OVERLAY_SIZE.width, height };
}

export function createOverlayLogicalSize(mode: OverlayMode, taskCount = 0) {
  const { width, height } = getOverlayWindowSize(mode, taskCount);
  return new LogicalSize(width, height);
}

export function requestOverlayRaise() {
  return invoke("request_overlay_raise");
}

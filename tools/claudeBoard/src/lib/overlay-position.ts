export type OverlayPosition = { x: number; y: number };

export function getDefaultOverlayPosition(screenWidth: number, barWidth: number): OverlayPosition {
  return {
    x: Math.round((screenWidth - barWidth) / 2),
    y: 12,
  };
}

export function overlayLabel(totalTasks: number): string {
  return totalTasks === 0 ? "当前无任务" : "Claude Tasks";
}

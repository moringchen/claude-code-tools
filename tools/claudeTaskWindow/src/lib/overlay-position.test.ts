import { describe, expect, it } from "vitest";
import { getDefaultOverlayPosition, overlayLabel } from "./overlay-position";

describe("overlay-position", () => {
  it("centers the bar horizontally near the top edge", () => {
    expect(getDefaultOverlayPosition(1440, 360)).toEqual({ x: 540, y: 12 });
  });

  it("shows the idle copy when there are no tasks", () => {
    expect(overlayLabel(0)).toBe("当前无任务");
  });
});

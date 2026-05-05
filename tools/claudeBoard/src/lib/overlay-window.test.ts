import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  COLLAPSED_OVERLAY_SIZE,
  EXPANDED_OVERLAY_SIZE,
  getOverlayWindowSize,
  requestOverlayRaise,
} from "./overlay-window";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("overlay-window", () => {
  it("keeps the last working native startup footprint", () => {
    expect(COLLAPSED_OVERLAY_SIZE).toEqual({ width: 420, height: 320 });
    expect(getOverlayWindowSize(false)).toEqual(COLLAPSED_OVERLAY_SIZE);
  });

  it("uses the same native size after expansion so click coverage stays stable", () => {
    expect(EXPANDED_OVERLAY_SIZE).toEqual({ width: 420, height: 320 });
    expect(getOverlayWindowSize(true)).toEqual(EXPANDED_OVERLAY_SIZE);
  });

  it("requests native overlay raise through the Tauri command bridge", () => {
    requestOverlayRaise();

    expect(invoke).toHaveBeenCalledWith("request_overlay_raise");
  });
});

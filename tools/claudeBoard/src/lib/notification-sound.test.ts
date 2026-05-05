import { describe, expect, it } from "vitest";
import { getNotificationToneSpecs } from "./notification-sound";

describe("notification-sound", () => {
  it("uses classic tones by default for waiting and completed", () => {
    expect(getNotificationToneSpecs("waiting", "classic")).toEqual([{ frequency: 660, durationMs: 180 }]);
    expect(getNotificationToneSpecs("completed", "classic")).toEqual([{ frequency: 880, durationMs: 120 }]);
  });

  it("uses softer lower tones for the soft style", () => {
    expect(getNotificationToneSpecs("waiting", "soft")).toEqual([{ frequency: 523.25, durationMs: 160 }]);
    expect(getNotificationToneSpecs("completed", "soft")).toEqual([{ frequency: 659.25, durationMs: 120 }]);
  });

  it("uses a brighter two-tone waiting cue", () => {
    expect(getNotificationToneSpecs("waiting", "bright")).toEqual([
      { frequency: 784, durationMs: 90 },
      { frequency: 988, durationMs: 90 },
    ]);
  });

  it("returns no tones when there is no alert", () => {
    expect(getNotificationToneSpecs(null, "bright")).toEqual([]);
  });
});

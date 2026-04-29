import { describe, expect, it } from "vitest";
import { defaultPreferences, loadPreferences, savePreferences } from "./settings";

describe("settings", () => {
  it("returns defaults when storage is empty", () => {
    const storage = new Map<string, string>();
    expect(loadPreferences(storage)).toEqual(defaultPreferences);
  });

  it("round-trips preferences through storage", () => {
    const storage = new Map<string, string>();
    const next = {
      notifyCompleted: false,
      notifyNeedsUser: true,
      speakCompleted: true,
      speakNeedsUser: false,
    };

    savePreferences(storage, next);
    expect(loadPreferences(storage)).toEqual(next);
  });
});

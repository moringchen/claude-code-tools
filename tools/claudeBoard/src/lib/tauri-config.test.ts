import { describe, expect, it } from "vitest";
import tauriConfig from "../../src-tauri/tauri.conf.json";

describe("tauri native window config", () => {
  it("uses the taller collapsed overlay height for the main window", () => {
    expect(tauriConfig.app.windows[0]).toEqual(expect.objectContaining({
      title: "claudeBoard",
      width: 260,
      height: 64,
    }));
  });

  it("labels the hidden overlay window as main so native code can show it", () => {
    expect(tauriConfig.app.windows[0]).toEqual(expect.objectContaining({
      label: "main",
      visible: false,
    }));
  });

  it("waits for the daemon snapshot endpoint before Tauri dev continues", () => {
    expect(tauriConfig.build.beforeDevCommand).toContain("127.0.0.1:46123/snapshot");
    expect(tauriConfig.build.beforeDevCommand).toContain("until curl");
  });
});

import { invoke } from "@tauri-apps/api/core";

export type SoundType = "waiting" | "completed";

const debugLogs: string[] = [];

export function getDebugLogs(): string[] {
  return [...debugLogs];
}

async function rustLog(level: "log" | "warn" | "error", message: string) {
  const timestamp = new Date().toISOString();
  const logLine = `[${timestamp}] [${level}] ${message}`;
  debugLogs.push(logLine);
  if (debugLogs.length > 100) {
    debugLogs.shift();
  }

  console[level === "error" ? "error" : level === "warn" ? "warn" : "log"](message);
  try {
    await invoke("log_from_frontend", { level, message });
  } catch {
  }
}

export function markUserInteraction(): void {
  void rustLog("log", "[sound] User interaction observed");
}

export function getUserInteractionStatus(): boolean {
  return true;
}

export async function playSound(type: SoundType): Promise<void> {
  await rustLog("log", `[sound] playSound called for type: ${type}`);

  try {
    const result = await invoke<null>("play_sound_file", { soundType: type });
    await rustLog("log", `[sound] Backend playback requested: ${type} result=${String(result)}`);
  } catch (error) {
    await rustLog("error", `[sound] Failed to request backend playback: ${type}, ${String(error)}`);
    throw error;
  }
}

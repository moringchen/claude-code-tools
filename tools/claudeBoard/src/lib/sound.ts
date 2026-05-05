import { invoke } from "@tauri-apps/api/core";

export type SoundType = "waiting" | "completed";

// Custom MP3 file paths
const CUSTOM_SOUND_PATHS: Record<SoundType, string> = {
  waiting: "/Users/moringchen/Downloads/待回复.mp3",
  completed: "/Users/moringchen/Downloads/任务完成.mp3",
};

// Track if we've had user interaction (required for audio playback)
let hasUserInteraction = false;

// Debug logs array for diagnostics
const debugLogs: string[] = [];

export function getDebugLogs(): string[] {
  return [...debugLogs];
}

// Helper to log to both console and Rust backend
async function rustLog(level: "log" | "warn" | "error", message: string) {
  const timestamp = new Date().toISOString();
  const logLine = `[${timestamp}] [${level}] ${message}`;
  debugLogs.push(logLine);
  // Keep only last 100 logs
  if (debugLogs.length > 100) {
    debugLogs.shift();
  }

  console[level === "error" ? "error" : level === "warn" ? "warn" : "log"](message);
  try {
    await invoke("log_from_frontend", { level, message });
  } catch {
    // Ignore logging errors
  }
}

export function markUserInteraction(): void {
  hasUserInteraction = true;
  void rustLog("log", "[sound] User interaction marked, audio can now play");
}

export function getUserInteractionStatus(): boolean {
  return hasUserInteraction;
}

export async function playSound(type: SoundType): Promise<void> {
  await rustLog("log", `[sound] playSound called for type: ${type}, hasUserInteraction: ${hasUserInteraction}`);

  try {
    // Check if we have user interaction (browser autoplay policy)
    if (!hasUserInteraction) {
      await rustLog("log", "[sound] Skipping playback - no user interaction yet");
      return;
    }

    const path = CUSTOM_SOUND_PATHS[type];
    await rustLog("log", `[sound] Loading sound: ${type} from: ${path}`);

    // Read file via Tauri command
    let data: number[];
    try {
      data = await invoke("read_sound_file", { path });
      await rustLog("log", `[sound] File loaded, size: ${data.length} bytes`);
    } catch (invokeError) {
      await rustLog("error", `[sound] Failed to invoke read_sound_file: ${String(invokeError)}`);
      return;
    }

    const uint8Array = new Uint8Array(data);
    await rustLog("log", `[sound] Created Uint8Array, length: ${uint8Array.length}`);

    // Create blob and object URL
    const blob = new Blob([uint8Array], { type: "audio/mpeg" });
    await rustLog("log", `[sound] Created blob, size: ${blob.size}`);

    const url = URL.createObjectURL(blob);
    await rustLog("log", `[sound] Created object URL: ${url}`);

    // Play audio
    const audio = new Audio(url);
    audio.volume = 0.7;
    await rustLog("log", "[sound] Created Audio element");

    // Wait for audio to be ready
    await rustLog("log", "[sound] Waiting for audio to be ready...");
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        void rustLog("log", "[sound] Audio load timeout, trying to play anyway");
        resolve();
      }, 2000);

      audio.addEventListener("canplaythrough", () => {
        clearTimeout(timeout);
        void rustLog("log", "[sound] Audio ready (canplaythrough)");
        resolve();
      }, { once: true });

      audio.addEventListener("error", (e) => {
        clearTimeout(timeout);
        void rustLog("error", `[sound] Audio error: ${e}`);
        reject(new Error(`Audio error: ${e}`));
      }, { once: true });

      audio.load();
    });

    await rustLog("log", "[sound] Attempting to play audio...");
    try {
      await audio.play();
      await rustLog("log", `[sound] Played successfully: ${type}`);
    } catch (playError) {
      await rustLog("error", `[sound] audio.play() failed: ${String(playError)}`);
    }

    // Cleanup URL after playing
    audio.addEventListener("ended", () => {
      URL.revokeObjectURL(url);
      void rustLog("log", `[sound] Cleaned up object URL for: ${type}`);
    });
  } catch (error) {
    await rustLog("error", `[sound] Failed to play sound: ${type}, ${String(error)}`);
  }
}

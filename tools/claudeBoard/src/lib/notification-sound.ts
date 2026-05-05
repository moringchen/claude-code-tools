import { convertFileSrc } from "@tauri-apps/api/core";
import type { SnapshotAlert } from "./snapshot-alert";
import type { NotificationSoundStyle } from "./settings";

export type ToneSpec = {
  frequency: number;
  durationMs: number;
};

type PlayableAlert = Exclude<SnapshotAlert, null>;

const NOTIFICATION_TONES: Record<NotificationSoundStyle, Record<PlayableAlert, ToneSpec[]>> = {
  classic: {
    waiting: [{ frequency: 660, durationMs: 180 }],
    completed: [{ frequency: 880, durationMs: 120 }],
  },
  soft: {
    waiting: [{ frequency: 523.25, durationMs: 160 }],
    completed: [{ frequency: 659.25, durationMs: 120 }],
  },
  bright: {
    waiting: [
      { frequency: 784, durationMs: 90 },
      { frequency: 988, durationMs: 90 },
    ],
    completed: [{ frequency: 1046.5, durationMs: 100 }],
  },
};

// Get the correct sound URL based on environment
function getSoundUrl(alert: PlayableAlert): string {
  // In Tauri app, use resource protocol
  if (typeof window !== "undefined" && (window as { __TAURI__?: boolean }).__TAURI__) {
    return convertFileSrc(`sounds/${alert}.mp3`);
  }
  // Fallback for web/dev
  return `/sounds/${alert}.mp3`;
}

let audioContext: AudioContext | null = null;
const audioCache: Map<string, HTMLAudioElement> = new Map();

function getAudioContext(): AudioContext | null {
  if (audioContext) {
    return audioContext;
  }

  const AudioContextConstructor = window.AudioContext ?? null;
  if (!AudioContextConstructor) {
    return null;
  }

  audioContext = new AudioContextConstructor();
  return audioContext;
}

async function playTone(frequency: number, durationMs: number) {
  const context = getAudioContext();
  if (!context) {
    return;
  }

  if (context.resume) {
    await context.resume().catch(() => undefined);
  }

  const oscillator = context.createOscillator();
  const gain = context.createGain();
  const startAt = context.currentTime;
  const endAt = startAt + durationMs / 1000;

  oscillator.type = "sine";
  oscillator.frequency.value = frequency;
  gain.gain.setValueAtTime(0.0001, startAt);
  gain.gain.linearRampToValueAtTime(0.14, startAt + 0.01);
  gain.gain.linearRampToValueAtTime(0.0001, endAt);

  oscillator.connect(gain);
  gain.connect(context.destination);
  oscillator.start(startAt);
  oscillator.stop(endAt);
}

async function playCustomSound(alert: PlayableAlert): Promise<void> {
  const soundUrl = getSoundUrl(alert);

  // Check if already cached
  let audio = audioCache.get(soundUrl);
  if (!audio) {
    audio = new Audio(soundUrl);
    audioCache.set(soundUrl, audio);
  }

  // Reset and play
  audio.currentTime = 0;
  try {
    await audio.play();
  } catch (e) {
    // Auto-play policy may block, ignore
  }
}

export function getNotificationToneSpecs(alert: SnapshotAlert, style: NotificationSoundStyle = "classic") {
  if (!alert) {
    return [];
  }

  return NOTIFICATION_TONES[style][alert];
}

export async function playRefreshNotification(
  alert: SnapshotAlert,
  style: NotificationSoundStyle = "classic",
) {
  if (!alert) {
    return;
  }

  // Always use custom MP3 sounds if available
  await playCustomSound(alert);
}

export type Preferences = {
  notifyCompleted: boolean;
  notifyNeedsUser: boolean;
  speakCompleted: boolean;
  speakNeedsUser: boolean;
};

export const defaultPreferences: Preferences = {
  notifyCompleted: true,
  notifyNeedsUser: true,
  speakCompleted: true,
  speakNeedsUser: true,
};

const STORAGE_KEY = "claude-task-window.preferences";

type StorageLike = Pick<Map<string, string>, "get" | "set">;

export function loadPreferences(storage: StorageLike): Preferences {
  const raw = storage.get(STORAGE_KEY);
  if (!raw) {
    return defaultPreferences;
  }

  return {
    ...defaultPreferences,
    ...(JSON.parse(raw) as Partial<Preferences>),
  };
}

export function savePreferences(storage: StorageLike, preferences: Preferences): void {
  storage.set(STORAGE_KEY, JSON.stringify(preferences));
}

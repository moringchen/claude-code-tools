import type { Preferences } from "../lib/settings";

type ContextMenuProps = {
  preferences: Preferences;
  onChange: (preferences: Preferences) => void;
};

export function ContextMenu({ preferences, onChange }: ContextMenuProps) {
  const toggle = (key: keyof Preferences) => {
    onChange({
      ...preferences,
      [key]: !preferences[key],
    });
  };

  return (
    <div>
      <label>
        <input
          type="checkbox"
          aria-label="notify-completed"
          checked={preferences.notifyCompleted}
          onChange={() => toggle("notifyCompleted")}
        />
        完成时通知
      </label>
      <label>
        <input
          type="checkbox"
          aria-label="notify-needs-user"
          checked={preferences.notifyNeedsUser}
          onChange={() => toggle("notifyNeedsUser")}
        />
        需要确认时通知
      </label>
      <label>
        <input
          type="checkbox"
          aria-label="speak-completed"
          checked={preferences.speakCompleted}
          onChange={() => toggle("speakCompleted")}
        />
        完成时播报
      </label>
      <label>
        <input
          type="checkbox"
          aria-label="speak-needs-user"
          checked={preferences.speakNeedsUser}
          onChange={() => toggle("speakNeedsUser")}
        />
        需要确认时播报
      </label>
    </div>
  );
}

import type { TaskCard } from "../lib/task-model";
import {
  resolveVisualStatus,
  visualStatusLabel,
  type VisualStatus,
} from "../lib/visual-status";

type StatusVisualProps = {
  task: TaskCard;
  variant: "header" | "list";
  showLabel: boolean;
};

function glyphForStatus(status: VisualStatus) {
  switch (status) {
    case "waiting":
      return (
        <svg
          viewBox="0 0 16 16"
          aria-hidden="true"
          focusable="false"
          className="status-glyph status-glyph-waiting"
        >
          <path
            className="status-glyph-waiting-bubble"
            d="M3.25 4.85c0-1.55 1.25-2.8 2.8-2.8h3.9c1.55 0 2.8 1.25 2.8 2.8v2.5c0 1.55-1.25 2.8-2.8 2.8H8.4l-2.25 2.15c-.28.27-.75.07-.75-.32v-1.83h-.35c-1 0-1.8-.8-1.8-1.8V4.85Z"
            fill="currentColor"
          />
          <path
            className="status-glyph-waiting-reply"
            d="M6.3 5.65h4.2M6.3 7.8h2.6"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1.2"
          />
          <circle className="status-glyph-waiting-dot" cx="11.95" cy="4.45" r="1.15" fill="currentColor" />
        </svg>
      );
    case "completed":
      return (
        <svg
          viewBox="0 0 16 16"
          aria-hidden="true"
          focusable="false"
          className="status-glyph status-glyph-completed"
        >
          <circle className="status-glyph-completed-core" cx="8" cy="8" r="4.65" fill="currentColor" />
          <path
            className="status-glyph-completed-check"
            d="M5.55 8.1 7.3 9.9 10.8 6.4"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="1.7"
          />
          <path
            className="status-glyph-completed-spark"
            d="M12.2 2.6v1.2M12.2 5.15v1.05M10.55 4.1h1.1M12.75 4.1h1.1"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1"
          />
        </svg>
      );
    case "running":
    default:
      return (
        <svg
          viewBox="0 0 16 16"
          aria-hidden="true"
          focusable="false"
          className="status-glyph status-glyph-running"
        >
          <circle className="status-glyph-running-ring" cx="8" cy="8" r="5.15" fill="none" stroke="currentColor" strokeWidth="1.2" />
          <path
            className="status-glyph-running-orbit"
            d="M11.8 4.55a4.8 4.8 0 0 1 1 5.35"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1.55"
          />
          <path
            className="status-glyph-running-tool"
            d="M6.1 5.95 9.9 9.75M9.15 5.2l1.65 1.65M5.35 9l1.65 1.65"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1.2"
          />
          <circle className="status-glyph-running-core" cx="8" cy="8" r="1.35" fill="currentColor" />
        </svg>
      );
  }
}

export function StatusVisual({ task, variant, showLabel }: StatusVisualProps) {
  const status = resolveVisualStatus(task);

  if (!status) {
    return null;
  }

  const label = visualStatusLabel(status);

  return (
    <span
      className={`status-visual status-visual-${variant} status-visual-${status}`}
      data-status={status}
      aria-label={showLabel ? label : `${label} 图标`}
      role="img"
    >
      <span className="status-visual-icon">{glyphForStatus(status)}</span>
      {showLabel ? <span className="status-visual-label">{label}</span> : null}
    </span>
  );
}

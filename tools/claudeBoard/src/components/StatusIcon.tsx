import type { VisualStatus } from "../lib/visual-status";

type StatusIconProps = {
  status: VisualStatus;
};

export function StatusIcon({ status }: StatusIconProps) {
  switch (status) {
    case "running":
      return (
        <span className="status-icon running" title="进行中">
          <span className="spinner"></span>
        </span>
      );
    case "waiting":
      return (
        <span className="status-icon waiting" title="待回复">
          <span className="pulse"></span>
        </span>
      );
    case "completed":
      return (
        <span className="status-icon completed" title="已完成">
          <span className="check">✓</span>
        </span>
      );
    default:
      return null;
  }
}

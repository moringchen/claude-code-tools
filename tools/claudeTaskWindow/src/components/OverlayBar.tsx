import type { TaskCounts } from "../lib/task-model";

type OverlayBarProps = {
  counts: TaskCounts;
  isExpanded: boolean;
  onToggle: () => void;
};

export function OverlayBar({ counts, isExpanded, onToggle }: OverlayBarProps) {
  return (
    <button
      type="button"
      className="overlay-bar"
      aria-expanded={isExpanded}
      onClick={onToggle}
    >
      {counts.total === 0 ? (
        <span>当前无任务</span>
      ) : (
        <>
          <span>总 {counts.total}</span>
          <span>需确认 {counts.needsUser}</span>
          <span>已完成 {counts.completed}</span>
          <span>运行中 {counts.running}</span>
        </>
      )}
    </button>
  );
}

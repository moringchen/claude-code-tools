import { useRef } from "react";
import type { MouseEvent as ReactMouseEvent } from "react";
import { markUserInteraction } from "../lib/sound";

type OverlayBarProps = {
  summary: string;
  isExpanded: boolean;
  isCompact: boolean;
  onToggle: () => void;
  onEnterCompactMode: () => void;
  onDragStart: () => void;
};

type DragState = {
  startX: number;
  startY: number;
  didDrag: boolean;
  suppressClick: boolean;
};

const DRAG_THRESHOLD = 4;

export function OverlayBar({ summary, isExpanded, isCompact, onToggle, onEnterCompactMode, onDragStart }: OverlayBarProps) {
  const dragStateRef = useRef<DragState | null>(null);

  const handleMouseDown = (event: ReactMouseEvent<HTMLButtonElement>) => {
    markUserInteraction();

    dragStateRef.current = {
      startX: event.clientX,
      startY: event.clientY,
      didDrag: false,
      suppressClick: false,
    };
  };

  const handleMouseMove = (event: ReactMouseEvent<HTMLButtonElement>) => {
    const dragState = dragStateRef.current;
    if (!dragState || dragState.didDrag) {
      return;
    }

    const movedX = event.clientX - dragState.startX;
    const movedY = event.clientY - dragState.startY;

    if (Math.hypot(movedX, movedY) >= DRAG_THRESHOLD) {
      dragState.didDrag = true;
      dragState.suppressClick = true;
      onDragStart();
    }
  };

  const handleClick = () => {
    if (dragStateRef.current?.suppressClick) {
      dragStateRef.current = null;
      return;
    }

    dragStateRef.current = null;
    onToggle();
  };

  const handleDoubleClick = () => {
    dragStateRef.current = null;
    if (!isCompact) {
      onEnterCompactMode();
    }
  };

  const resetDragState = () => {
    if (!dragStateRef.current) {
      return;
    }

    if (dragStateRef.current.didDrag) {
      dragStateRef.current.didDrag = false;
      return;
    }

    dragStateRef.current = null;
  };

  return (
    <button
      type="button"
      className="island-bar"
      aria-expanded={isExpanded}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseLeave={resetDragState}
    >
      <span className="island-summary">{summary}</span>
    </button>
  );
}

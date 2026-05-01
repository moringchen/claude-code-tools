import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OverlayBar } from "./OverlayBar";

afterEach(() => {
  cleanup();
});

describe("OverlayBar", () => {
  it("renders the collapsed island summary", () => {
    render(
      <OverlayBar
        summary="需确认：approve command"
        isExpanded={false}
        onToggle={() => {}}
        onDragStart={() => {}}
      />,
    );

    expect(screen.getByText("需确认：approve command")).toBeTruthy();
  });

  it("toggles expansion for a stationary click", () => {
    const onToggle = vi.fn();
    const onDragStart = vi.fn();

    render(
      <OverlayBar
        summary="当前无任务"
        isExpanded={false}
        onToggle={onToggle}
        onDragStart={onDragStart}
      />,
    );

    const button = screen.getByRole("button", { name: "当前无任务" });
    fireEvent.mouseDown(button, { clientX: 100, clientY: 100 });
    fireEvent.click(button);

    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(onDragStart).not.toHaveBeenCalled();
  });

  it("starts dragging after pointer movement crosses threshold", () => {
    const onToggle = vi.fn();
    const onDragStart = vi.fn();

    render(
      <OverlayBar
        summary="当前无任务"
        isExpanded={false}
        onToggle={onToggle}
        onDragStart={onDragStart}
      />,
    );

    const button = screen.getByRole("button", { name: "当前无任务" });
    fireEvent.mouseDown(button, { clientX: 10, clientY: 10 });
    fireEvent.mouseMove(button, { clientX: 24, clientY: 10 });

    expect(onDragStart).toHaveBeenCalledTimes(1);
    expect(onToggle).not.toHaveBeenCalled();
  });
});


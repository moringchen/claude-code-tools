import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OverlayBar } from "./OverlayBar";

describe("OverlayBar", () => {
  it("renders 当前无任务 when counts are zero", () => {
    render(
      <OverlayBar
        counts={{ total: 0, needsUser: 0, completed: 0, running: 0 }}
        isExpanded={false}
        onToggle={() => {}}
      />,
    );

    expect(screen.getByText("当前无任务")).toBeTruthy();
  });

  it("renders the summary counters when tasks exist", () => {
    render(
      <OverlayBar
        counts={{ total: 2, needsUser: 1, completed: 1, running: 0 }}
        isExpanded={true}
        onToggle={() => {}}
      />,
    );

    expect(screen.getByText("总 2")).toBeTruthy();
    expect(screen.getByText("需确认 1")).toBeTruthy();
    expect(screen.getByText("已完成 1")).toBeTruthy();
    expect(screen.getByText("运行中 0")).toBeTruthy();
  });
});

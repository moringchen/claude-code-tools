import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ContextMenu } from "./ContextMenu";

const basePreferences = {
  notifyCompleted: true,
  notifyNeedsUser: false,
  speakCompleted: true,
  speakNeedsUser: false,
};

afterEach(() => {
  cleanup();
});

describe("ContextMenu", () => {
  it("renders all preferences", () => {
    render(<ContextMenu preferences={basePreferences} onChange={vi.fn()} />);

    expect(screen.getByLabelText("notify-completed")).toBeTruthy();
    expect(screen.getByLabelText("notify-needs-user")).toBeTruthy();
    expect(screen.getByLabelText("speak-completed")).toBeTruthy();
    expect(screen.getByLabelText("speak-needs-user")).toBeTruthy();
  });

  it("toggles notify completed only", () => {
    const onChange = vi.fn();

    render(<ContextMenu preferences={basePreferences} onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("notify-completed"));

    expect(onChange).toHaveBeenCalledWith({
      notifyCompleted: false,
      notifyNeedsUser: false,
      speakCompleted: true,
      speakNeedsUser: false,
    });
  });

  it("toggles notify needs user only", () => {
    const onChange = vi.fn();

    render(<ContextMenu preferences={basePreferences} onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("notify-needs-user"));

    expect(onChange).toHaveBeenCalledWith({
      notifyCompleted: true,
      notifyNeedsUser: true,
      speakCompleted: true,
      speakNeedsUser: false,
    });
  });

  it("toggles speak completed only", () => {
    const onChange = vi.fn();

    render(<ContextMenu preferences={basePreferences} onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("speak-completed"));

    expect(onChange).toHaveBeenCalledWith({
      notifyCompleted: true,
      notifyNeedsUser: false,
      speakCompleted: false,
      speakNeedsUser: false,
    });
  });

  it("toggles speak needs user only", () => {
    const onChange = vi.fn();

    render(<ContextMenu preferences={basePreferences} onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("speak-needs-user"));

    expect(onChange).toHaveBeenCalledWith({
      notifyCompleted: true,
      notifyNeedsUser: false,
      speakCompleted: true,
      speakNeedsUser: true,
    });
  });
});

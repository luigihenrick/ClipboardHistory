import { describe, expect, it, vi } from "vitest";

import { focusSearchField } from "./focus";

function createField() {
  return {
    focus: vi.fn<(options?: FocusOptions) => void>(),
    select: vi.fn<() => void>(),
  };
}

describe("focusSearchField", () => {
  it("focuses and selects immediately, then focuses again on the next frame", () => {
    const field = createField();
    let scheduled: (() => void) | undefined;

    focusSearchField(field, (callback) => {
      scheduled = callback;
    });

    expect(field.focus).toHaveBeenCalledTimes(1);
    expect(field.focus).toHaveBeenCalledWith({ preventScroll: true });
    expect(field.select).toHaveBeenCalledTimes(1);

    scheduled?.();

    expect(field.focus).toHaveBeenCalledTimes(2);
    expect(field.focus).toHaveBeenLastCalledWith({ preventScroll: true });
    expect(field.select).toHaveBeenCalledTimes(1);
  });

  it("does nothing when the field is not mounted", () => {
    const schedule = vi.fn<(callback: () => void) => void>();

    expect(() => focusSearchField(null, schedule)).not.toThrow();
    expect(schedule).not.toHaveBeenCalled();
  });
});

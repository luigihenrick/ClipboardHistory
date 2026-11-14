import { describe, expect, it, vi } from "vitest";

import { focusSearchField, retrySearchFocus } from "./focus";

function createField() {
  return {
    focus: vi.fn<(options?: FocusOptions) => void>(),
    select: vi.fn<() => void>(),
  };
}

function createRetryField(focusOnAttempt: number) {
  let attempts = 0;
  const field = {
    focus: vi.fn<(options?: FocusOptions) => void>(() => {
      attempts += 1;
    }),
    select: vi.fn<() => void>(),
    isFocused: () => attempts >= focusOnAttempt,
  };

  return field;
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

describe("retrySearchFocus", () => {
  it("keeps focusing until the field reports focus", () => {
    const field = createRetryField(3);
    const scheduled: Array<() => void> = [];
    const schedule = vi.fn<(callback: () => void, delay: number) => () => void>(
      (callback) => {
        scheduled.push(callback);
        return vi.fn();
      },
    );

    retrySearchFocus(field, schedule, 5, 10);

    expect(field.focus).toHaveBeenCalledTimes(1);
    scheduled.shift()?.();
    expect(field.focus).toHaveBeenCalledTimes(2);
    scheduled.shift()?.();

    expect(field.focus).toHaveBeenCalledTimes(3);
    expect(field.select).toHaveBeenCalledTimes(3);
    expect(schedule).toHaveBeenCalledTimes(2);
    expect(scheduled).toHaveLength(0);
  });

  it("stops after the maximum number of attempts", () => {
    const field = createRetryField(Number.POSITIVE_INFINITY);
    const scheduled: Array<() => void> = [];
    const schedule = vi.fn<(callback: () => void, delay: number) => () => void>(
      (callback) => {
        scheduled.push(callback);
        return vi.fn();
      },
    );

    retrySearchFocus(field, schedule, 3, 10);
    scheduled.shift()?.();
    scheduled.shift()?.();

    expect(field.focus).toHaveBeenCalledTimes(3);
    expect(schedule).toHaveBeenCalledTimes(2);
    expect(scheduled).toHaveLength(0);
  });

  it("cancels pending attempts", () => {
    const field = createRetryField(Number.POSITIVE_INFINITY);
    const scheduled: Array<() => void> = [];
    const cancel = vi.fn();
    const schedule = vi.fn<(callback: () => void, delay: number) => () => void>(
      (callback) => {
        scheduled.push(callback);
        return cancel;
      },
    );

    const stop = retrySearchFocus(field, schedule, 5, 10);
    stop();
    scheduled.shift()?.();

    expect(field.focus).toHaveBeenCalledTimes(1);
    expect(cancel).toHaveBeenCalledOnce();
  });

  it("does nothing without a field", () => {
    const schedule = vi.fn<(callback: () => void, delay: number) => () => void>();

    expect(() => retrySearchFocus(null, schedule)).not.toThrow();
    expect(schedule).not.toHaveBeenCalled();
  });
});

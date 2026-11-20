import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FOCUS_RETRY_ATTEMPTS,
  FOCUS_RETRY_INTERVAL_MS,
  focusSearchField,
  inputFocusTarget,
  retrySearchFocus,
} from "./focus";

function createField() {
  return {
    focus: vi.fn<(options?: FocusOptions) => void>(),
    select: vi.fn<() => void>(),
  };
}

function createRetryField(focusOnAttempt: number) {
  let attempts = 0;
  return {
    focus: vi.fn<(options?: FocusOptions) => void>(() => {
      attempts += 1;
    }),
    select: vi.fn<() => void>(),
    isFocused: () => attempts >= focusOnAttempt,
  };
}

describe("focusSearchField", () => {
  it("focuses and selects immediately, then focuses again on the next frame", () => {
    const field = createField();
    let scheduled: (() => void) | undefined;

    focusSearchField(field, (callback) => {
      scheduled = callback;
    });

    expect(field.focus).toHaveBeenCalledWith({ preventScroll: true });
    expect(field.select).toHaveBeenCalledTimes(1);

    scheduled?.();

    expect(field.focus).toHaveBeenCalledTimes(2);
    expect(field.select).toHaveBeenCalledTimes(1);
  });

  it("does nothing when the field is not mounted", () => {
    const schedule = vi.fn<(callback: () => void) => void>();
    expect(() => focusSearchField(null, schedule)).not.toThrow();
    expect(schedule).not.toHaveBeenCalled();
  });
});

describe("inputFocusTarget", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("reports whether the input is the active element", () => {
    const input = document.createElement("input");
    document.body.append(input);
    const target = inputFocusTarget(input);

    expect(target).not.toBeNull();
    expect(target?.isFocused()).toBe(false);

    input.focus();
    expect(target?.isFocused()).toBe(true);

    target?.select();
    expect(input.selectionStart).toBe(0);
    target?.focus();
    expect(document.activeElement).toBe(input);
  });

  it("returns null without an input", () => {
    expect(inputFocusTarget(null)).toBeNull();
  });
});

describe("retrySearchFocus", () => {
  it("polls fast enough to focus well within a second", () => {
    expect(FOCUS_RETRY_INTERVAL_MS).toBeLessThanOrEqual(50);
    expect(FOCUS_RETRY_ATTEMPTS * FOCUS_RETRY_INTERVAL_MS).toBeGreaterThanOrEqual(
      10_000,
    );
  });

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
    scheduled.shift()?.();
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

type FocusableField = {
  focus: (options?: FocusOptions) => void;
  select: () => void;
};

export type FocusableSearchField = FocusableField & {
  isFocused: () => boolean;
};

type FrameScheduler = (callback: () => void) => void;
type TimeoutScheduler = (callback: () => void, delay: number) => () => void;

/**
 * Focuses and selects the search field immediately, then again on the next frame. The second
 * attempt covers the moment when the popup window finishes mapping and gains focus.
 */
export function focusSearchField(
  field: FocusableField | null,
  schedule: FrameScheduler = (callback) => requestAnimationFrame(callback),
): void {
  if (!field) return;

  const focus = () => field.focus({ preventScroll: true });
  focus();
  field.select();

  // Focus only on the retry: selecting again could wipe text the user already started typing.
  schedule(focus);
}

/** Builds the focus target used by the popup's search input. */
export function inputFocusTarget(input: HTMLInputElement | null): FocusableSearchField | null {
  if (!input) return null;

  return {
    focus: (options) => input.focus(options),
    select: () => input.select(),
    isFocused: () => document.activeElement === input && document.hasFocus(),
  };
}

/**
 * Keeps trying to focus the search input for a few seconds. This covers slow AppImage cold
 * starts, where the window can be mapped and focused before the webview is ready to handle it.
 * Returns a function that cancels the pending retries.
 */
export function retrySearchFocus(
  field: FocusableSearchField | null,
  schedule: TimeoutScheduler = (callback, delay) => {
    const id = window.setTimeout(callback, delay);
    return () => window.clearTimeout(id);
  },
  attempts = 120,
  interval = 100,
): () => void {
  if (!field) return () => undefined;

  let cancelled = false;
  let cancelTimer: (() => void) | undefined;
  let remaining = attempts;

  const attempt = () => {
    if (cancelled) return;
    if (field.isFocused()) {
      cancelTimer?.();
      return;
    }

    field.focus({ preventScroll: true });
    field.select();
    remaining -= 1;
    if (field.isFocused() || remaining <= 0) return;

    cancelTimer = schedule(attempt, interval);
  };

  attempt();

  return () => {
    cancelled = true;
    cancelTimer?.();
  };
}

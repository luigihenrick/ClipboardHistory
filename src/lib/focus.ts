type FocusableField = {
  focus: (options?: FocusOptions) => void;
  select: () => void;
};

type FrameScheduler = (callback: () => void) => void;

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

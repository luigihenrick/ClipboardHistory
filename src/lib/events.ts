import { listen } from "@tauri-apps/api/event";

type EventHandler<T> = (payload: T) => void;
type Unlisten = () => void;

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Subscribes to an app event.
 *
 * In a regular browser (the dev-only UI preview) events are dispatched on `window` instead,
 * which lets the React screens run without the Rust backend.
 */
export function onEvent<T>(name: string, handler: EventHandler<T>): Promise<Unlisten> {
  if (!isTauri) {
    const listener = (event: Event) => handler((event as CustomEvent<T>).detail);
    window.addEventListener(`cliphistory:${name}`, listener);
    return Promise.resolve(() =>
      window.removeEventListener(`cliphistory:${name}`, listener),
    );
  }

  return listen<T>(name, (event) => handler(event.payload));
}

export function emitPreviewEvent<T>(name: string, payload: T): void {
  window.dispatchEvent(new CustomEvent(`cliphistory:${name}`, { detail: payload }));
}

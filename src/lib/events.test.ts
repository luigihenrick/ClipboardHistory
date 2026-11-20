import { afterEach, describe, expect, it, vi } from "vitest";

import { emitPreviewEvent, onEvent } from "./events";

describe("events", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("subscribes to browser preview events and unsubscribes", async () => {
    const handler = vi.fn();
    const unlisten = await onEvent<{ value: number }>("demo", handler);

    emitPreviewEvent("demo", { value: 42 });
    expect(handler).toHaveBeenCalledWith({ value: 42 });

    unlisten();
    emitPreviewEvent("demo", { value: 7 });
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("uses the Tauri event bridge when available", async () => {
    vi.resetModules();
    const listen = vi.fn((_name: string, handler: (event: { payload: unknown }) => void) => {
      handler({ payload: { value: 9 } });
      return Promise.resolve(() => undefined);
    });
    vi.doMock("@tauri-apps/api/event", () => ({ listen }));
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });

    try {
      const events = await import("./events");
      const handler = vi.fn();
      const unlisten = await events.onEvent<{ value: number }>("tauri-demo", handler);

      expect(listen).toHaveBeenCalledWith("tauri-demo", expect.any(Function));
      expect(handler).toHaveBeenCalledWith({ value: 9 });
      unlisten();
    } finally {
      delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
      vi.doUnmock("@tauri-apps/api/event");
      vi.resetModules();
    }
  });
});

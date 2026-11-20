import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "./ipc";

describe("preview IPC wrappers", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    window.history.replaceState({}, "", "/");
  });

  afterEach(() => {
    window.history.replaceState({}, "", "/");
  });

  it("loads entries, settings, stats and app info", async () => {
    expect((await api.getEntries("acacia")).length).toBe(1);
    expect((await api.getSettings()).language).toBe("en");
    expect(await api.getStats()).toMatchObject({ total: 9, pinned: 2 });
    expect((await api.getAppInfo()).wayland).toBe(true);
    expect(await api.shortcutStatus()).toBe(true);
    expect((await api.getDiagnostics()).tauriVersion).toBe("preview");
  });

  it("updates settings and language", async () => {
    const settings = await api.getSettings();
    const saved = await api.saveSettings({ ...settings, maxItems: 40 });
    expect(saved.maxItems).toBe(40);
    expect((await api.setLanguage("pt-BR")).language).toBe("pt-BR");
  });

  it("delegates entry actions to the preview backend", async () => {
    const info = vi.spyOn(console, "info").mockImplementation(() => undefined);
    const entries = await api.getEntries("");
    const id = entries.find((entry) => !entry.pinned)!.id;

    await api.copyEntry(id);
    expect(await api.togglePin(id)).toBe(true);
    await api.deleteEntry(id);
    expect((await api.getEntries("")).some((entry) => entry.id === id)).toBe(false);
    expect(await api.clearHistory()).toBeGreaterThan(0);
    expect(info).toHaveBeenCalled();
  });

  it("opens and closes preview windows", async () => {
    const open = vi.spyOn(window, "open").mockImplementation(() => null);
    const close = vi.spyOn(window, "close").mockImplementation(() => undefined);

    await api.openSettings();
    await api.openExternal("https://github.com/luigihenrick/ClipboardHistory");
    await api.hideSettings();
    await api.hidePopup();

    expect(open).toHaveBeenCalledTimes(2);
    expect(close).toHaveBeenCalledOnce();
  });

  it("returns a preview update when requested", async () => {
    window.history.replaceState({}, "", "/?update=1");

    const update = await api.getUpdate();
    expect(update?.version).toMatch(/^v\d+\.\d+\.\d+$/);
    expect((await api.checkForUpdates())?.version).toBe(update?.version);
  });

  it("copies diagnostics and reports search focus in preview mode", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    await api.copyDiagnostics();
    await api.openLogsFolder();
    await api.reportSearchFocus(42);

    expect(writeText).toHaveBeenCalledWith(
      expect.stringContaining(`ClipHistory ${__APP_VERSION__}`),
    );
  });
});

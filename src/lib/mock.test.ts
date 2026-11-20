import { beforeEach, describe, expect, it, vi } from "vitest";

async function loadMock() {
  vi.resetModules();
  return import("./mock");
}

describe("preview mock backend", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });

  it("returns and filters the sample history", async () => {
    const mock = await loadMock();

    expect(await mock.getEntries("")).toHaveLength(9);
    expect(await mock.getEntries("ACACIA")).toHaveLength(1);
    expect(await mock.getEntries("does-not-exist")).toHaveLength(0);
  });

  it("reads the stored preview language", async () => {
    localStorage.setItem("cliphistory.language", "pt-BR");
    const mock = await loadMock();

    expect((await mock.getSettings()).language).toBe("pt-BR");
  });

  it("loads, saves and changes settings", async () => {
    const mock = await loadMock();
    const initial = await mock.getSettings();

    expect(initial.language).toBe("en");

    const saved = await mock.saveSettings({
      ...initial,
      maxItems: 50,
      autoPaste: false,
    });
    expect(saved.maxItems).toBe(50);
    expect((await mock.getSettings()).autoPaste).toBe(false);

    const pt = await mock.setLanguage("pt-BR");
    expect(pt.language).toBe("pt-BR");
    expect((await mock.getSettings()).language).toBe("pt-BR");
  });

  it("reports stats and shortcut status", async () => {
    const mock = await loadMock();

    expect(await mock.getStats()).toEqual({ total: 9, pinned: 2, images: 2 });
    expect(await mock.shortcutStatus()).toBe(true);
  });

  it("copies entries, toggles pins and deletes entries", async () => {
    const mock = await loadMock();
    const info = vi.spyOn(console, "info").mockImplementation(() => undefined);
    const before = await mock.getEntries("");
    const target = before.find((entry) => entry.id === 3)!;
    const oldTimestamp = target.createdAt;

    await mock.copyEntry(target.id);
    const copied = (await mock.getEntries(""))[0];
    expect(copied.id).toBe(target.id);
    expect(copied.createdAt).toBeGreaterThanOrEqual(oldTimestamp);
    expect(info).toHaveBeenCalled();

    await mock.copyEntry(999);
    expect(await mock.togglePin(target.id)).toBe(true);
    expect(await mock.togglePin(target.id)).toBe(false);
    expect(await mock.togglePin(999)).toBe(false);

    await mock.deleteEntry(target.id);
    expect((await mock.getEntries("")).some((entry) => entry.id === target.id)).toBe(false);
  });

  it("clears only unpinned entries", async () => {
    const mock = await loadMock();

    expect(await mock.clearHistory()).toBe(7);
    const remaining = await mock.getEntries("");
    expect(remaining).toHaveLength(2);
    expect(remaining.every((entry) => entry.pinned)).toBe(true);
  });

  it("handles preview window actions", async () => {
    const mock = await loadMock();
    const open = vi.spyOn(window, "open").mockImplementation(() => null);
    const close = vi.spyOn(window, "close").mockImplementation(() => undefined);
    const info = vi.spyOn(console, "info").mockImplementation(() => undefined);

    await mock.hidePopup();
    await mock.openSettings();
    await mock.hideSettings();

    expect(info).toHaveBeenCalled();
    expect(open).toHaveBeenCalled();
    expect(close).toHaveBeenCalled();
  });
});

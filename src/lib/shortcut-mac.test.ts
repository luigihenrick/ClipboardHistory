import { afterEach, describe, expect, it, vi } from "vitest";

describe("shortcut formatting on macOS", () => {
  afterEach(() => {
    vi.resetModules();
  });

  it("uses macOS modifiers and joins labels with spaces", async () => {
    const original = navigator.userAgent;
    Object.defineProperty(navigator, "userAgent", {
      configurable: true,
      value: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    });

    try {
      vi.resetModules();
      const shortcut = await import("./shortcut");

      const formatted = shortcut.formatAccelerator(
        "CommandOrControl+Alt+Shift+ArrowUp+KeyV",
        "en",
      );
      expect(formatted).toContain("⌘");
      expect(formatted).toContain("⌥");
      expect(formatted).toContain("⇧");
      expect(formatted).toContain("↑");
      expect(formatted).not.toContain("+");
    } finally {
      Object.defineProperty(navigator, "userAgent", {
        configurable: true,
        value: original,
      });
    }
  });
});

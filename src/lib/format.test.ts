import { describe, expect, it } from "vitest";

import { previewText, relativeTime } from "./format";

const now = Date.UTC(2026, 0, 1, 12, 0, 0);
const minute = 60_000;
const hour = 60 * minute;
const day = 24 * hour;

describe("previewText", () => {
  it("collapses whitespace and trims the preview", () => {
    expect(previewText("  one\n\n\ttwo   three  ")).toBe("one two three");
  });

  it("keeps ordinary text unchanged", () => {
    expect(previewText("hello")).toBe("hello");
  });
});

describe("relativeTime", () => {
  it("formats the current moment", () => {
    expect(relativeTime(now, "en", now)).toBe("now");
    expect(relativeTime(now, "pt-BR", now)).toBe("agora");
  });

  it("formats minutes", () => {
    expect(relativeTime(now - 5 * minute, "en", now)).toBe("5 min ago");
    expect(relativeTime(now - 5 * minute, "pt-BR", now)).toBe("há 5 min");
  });

  it("formats hours", () => {
    expect(relativeTime(now - 3 * hour, "en", now)).toBe("3 h ago");
    expect(relativeTime(now - 3 * hour, "pt-BR", now)).toBe("há 3 h");
  });

  it("formats yesterday and multiple days", () => {
    expect(relativeTime(now - day, "en", now)).toBe("yesterday");
    expect(relativeTime(now - 3 * day, "en", now)).toBe("3 days ago");
    expect(relativeTime(now - 3 * day, "pt-BR", now)).toBe("há 3 dias");
  });

  it("uses a localized date for older entries", () => {
    const old = Date.UTC(2025, 0, 15, 12, 0, 0);
    expect(relativeTime(old, "en", now)).toMatch(/Jan/i);
    expect(relativeTime(old, "pt-BR", now)).toMatch(/jan/i);
  });

  it("treats future timestamps as now", () => {
    expect(relativeTime(now + minute, "en", now)).toBe("now");
  });
});

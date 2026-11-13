import { describe, expect, it } from "vitest";

import { relativeTime } from "./format";

const now = Date.UTC(2026, 0, 1, 12, 0, 0);

describe("relativeTime", () => {
  it("formats minutes in English", () => {
    expect(relativeTime(now - 5 * 60_000, "en", now)).toBe("5 min ago");
  });

  it("formats minutes in Portuguese", () => {
    expect(relativeTime(now - 5 * 60_000, "pt-BR", now)).toBe("há 5 min");
  });

  it("formats the current moment", () => {
    expect(relativeTime(now, "en", now)).toBe("now");
    expect(relativeTime(now, "pt-BR", now)).toBe("agora");
  });
});

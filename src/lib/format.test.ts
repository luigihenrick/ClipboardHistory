import { describe, expect, it } from "vitest";

import { charLabel, relativeTime } from "./format";

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

describe("charLabel", () => {
  it("uses singular and plural in English", () => {
    expect(charLabel(1, "en")).toBe("1 character");
    expect(charLabel(2, "en")).toBe("2 characters");
  });

  it("uses singular and plural in Portuguese", () => {
    expect(charLabel(1, "pt-BR")).toBe("1 caractere");
    expect(charLabel(2, "pt-BR")).toBe("2 caracteres");
  });
});

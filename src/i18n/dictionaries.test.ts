import { describe, expect, it } from "vitest";

import { en, ptBR } from "./dictionaries";

function placeholders(value: string): string[] {
  return [...value.matchAll(/\{(\w+)\}/g)]
    .map((match) => match[1])
    .sort();
}

describe("translations", () => {
  it("have matching keys", () => {
    expect(Object.keys(ptBR).sort()).toEqual(Object.keys(en).sort());
  });

  it("have no empty values", () => {
    for (const [key, value] of [
      ...Object.entries(en),
      ...Object.entries(ptBR),
    ]) {
      expect(value.trim(), key).not.toBe("");
    }
  });

  it("use the same interpolation placeholders in both languages", () => {
    for (const key of Object.keys(en) as Array<keyof typeof en>) {
      expect(placeholders(ptBR[key]), key).toEqual(placeholders(en[key]));
    }
  });
});

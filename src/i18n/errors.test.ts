import { describe, expect, it } from "vitest";

import { localizeError } from "./errors";

describe("localizeError", () => {
  it("returns the original message in English", () => {
    expect(localizeError("Item not found.", "en")).toBe("Item not found.");
  });

  it("translates known backend errors to Portuguese", () => {
    expect(localizeError("Item not found.", "pt-BR")).toBe(
      "Item não encontrado.",
    );
    expect(localizeError('Could not register "Ctrl+`": busy', "pt-BR")).toBe(
      'Não foi possível registrar "Ctrl+`": busy',
    );
  });

  it("keeps unknown technical messages unchanged", () => {
    expect(localizeError("Connection refused", "pt-BR")).toBe(
      "Connection refused",
    );
  });
});

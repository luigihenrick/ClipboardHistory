import { describe, expect, it } from "vitest";

import { translate } from "./translate";

describe("translate", () => {
  it("returns English and Portuguese templates", () => {
    expect(translate("en", "common.close")).toBe("Close");
    expect(translate("pt-BR", "common.close")).toBe("Fechar");
  });

  it("interpolates parameters", () => {
    expect(translate("en", "time.minutes", { count: 5 })).toBe("5 min ago");
    expect(translate("pt-BR", "time.minutes", { count: 5 })).toBe("há 5 min");
  });

  it("leaves unknown placeholders and missing parameters intact", () => {
    expect(translate("en", "time.minutes")).toBe("{count} min ago");
    expect(translate("en", "time.minutes", { count: undefined as never })).toBe(
      "{count} min ago",
    );
  });

});

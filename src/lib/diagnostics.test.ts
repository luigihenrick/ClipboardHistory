import { describe, expect, it } from "vitest";

import { getDiagnostics } from "./ipc";

describe("diagnostics", () => {
  it("returns a preview snapshot outside Tauri", async () => {
    const diagnostics = await getDiagnostics();

    expect(diagnostics.appVersion).toBe(__APP_VERSION__);
    expect(diagnostics.tauriVersion).toBe("preview");
    expect(diagnostics.summary).toContain("ClipHistory");
  });
});

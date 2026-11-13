import { describe, expect, it } from "vitest";

import packageJson from "../../package.json";
import packageLock from "../../package-lock.json";

describe("version source", () => {
  it("keeps package-lock.json in sync with package.json", () => {
    expect(packageLock.version).toBe(packageJson.version);
    expect(packageLock.packages[""].version).toBe(packageJson.version);
  });

  it("uses a valid semver version", () => {
    expect(packageJson.version).toMatch(/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
  });
});

import { describe, expect, it, vi } from "vitest";

import { checkForUpdate, isNewerVersion } from "./update";

function response(payload: unknown, ok = true, status = 200): Response {
  return {
    ok,
    status,
    json: async () => payload,
  } as Response;
}

describe("isNewerVersion", () => {
  it("compares major, minor and patch versions", () => {
    expect(isNewerVersion("v1.3.0", "1.2.2")).toBe(true);
    expect(isNewerVersion("1.2.3", "1.2.2")).toBe(true);
    expect(isNewerVersion("1.2.2", "1.2.2")).toBe(false);
    expect(isNewerVersion("v1.1.9", "1.2.0")).toBe(false);
  });

  it("treats prereleases as older than the final release", () => {
    expect(isNewerVersion("v1.2.3-beta.1", "1.2.2")).toBe(true);
    expect(isNewerVersion("v1.2.3", "1.2.3-beta.1")).toBe(true);
    expect(isNewerVersion("v1.2.3-beta.1", "1.2.3")).toBe(false);
  });

  it("compares prerelease identifiers numerically", () => {
    expect(isNewerVersion("v1.2.3-beta.10", "1.2.3-beta.2")).toBe(true);
    expect(isNewerVersion("v1.2.3-beta.2", "1.2.3-beta.10")).toBe(false);
    expect(isNewerVersion("v1.2.3-rc.1", "1.2.3-beta.10")).toBe(true);
  });

  it("ignores invalid versions", () => {
    expect(isNewerVersion("nightly", "1.2.2")).toBe(false);
    expect(isNewerVersion("v1.2.2", "dev")).toBe(false);
  });
});

describe("checkForUpdate", () => {
  it("returns update information for a newer release", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      response({
        tag_name: "v1.3.0",
        name: "ClipHistory 1.3.0",
        body: "Release notes",
        html_url: "https://github.com/luigihenrick/ClipboardHistory/releases/tag/v1.3.0",
      }),
    );

    await expect(checkForUpdate("1.2.2", fetcher)).resolves.toEqual({
      version: "v1.3.0",
      name: "ClipHistory 1.3.0",
      notes: "Release notes",
      url: "https://github.com/luigihenrick/ClipboardHistory/releases/tag/v1.3.0",
    });
  });

  it("returns null when the current version is already the latest", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      response({
        tag_name: "v1.2.2",
        name: "ClipHistory 1.2.2",
        body: "",
        html_url: "https://github.com/luigihenrick/ClipboardHistory/releases/tag/v1.2.2",
      }),
    );

    await expect(checkForUpdate("1.2.2", fetcher)).resolves.toBeNull();
  });

  it("rejects an unsuccessful response", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(response({}, false, 403));

    await expect(checkForUpdate("1.2.2", fetcher)).rejects.toThrow(
      "Update check failed with status 403",
    );
  });
});

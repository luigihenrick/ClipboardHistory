import { describe, expect, it, vi } from "vitest";

import { checkForUpdate } from "./update";

function response(payload: unknown): Response {
  return {
    ok: true,
    status: 200,
    json: async () => payload,
  } as Response;
}

describe("update payload validation", () => {
  it("rejects a non-object payload", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(response(null));
    await expect(checkForUpdate("1.0.0", fetcher)).rejects.toThrow(
      "Invalid update response.",
    );
  });

  it("rejects missing required fields", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      response({ tag_name: "v2.0.0" }),
    );
    await expect(checkForUpdate("1.0.0", fetcher)).rejects.toThrow(
      "Invalid update response.",
    );
  });

  it("uses fallbacks for optional fields and rejects non-HTTPS URLs", async () => {
    const withoutOptional = vi.fn<typeof fetch>().mockResolvedValue(
      response({
        tag_name: "v2.0.0",
        html_url: "https://github.com/luigihenrick/ClipboardHistory",
      }),
    );
    await expect(checkForUpdate("1.0.0", withoutOptional)).resolves.toMatchObject({
      name: "v2.0.0",
      notes: "",
    });

    const insecure = vi.fn<typeof fetch>().mockResolvedValue(
      response({
        tag_name: "v2.0.0",
        html_url: "http://example.com/release",
      }),
    );
    await expect(checkForUpdate("1.0.0", insecure)).resolves.toBeNull();
  });
});

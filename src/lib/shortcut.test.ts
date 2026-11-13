import { describe, expect, it } from "vitest";

import { buildAccelerator, formatAccelerator } from "./shortcut";

function keyEvent(
  event: Partial<KeyboardEvent> & { code: string; key: string },
): KeyboardEvent {
  return {
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ...event,
  } as KeyboardEvent;
}

describe("formatAccelerator", () => {
  it("renders the Backquote key", () => {
    expect(formatAccelerator("CommandOrControl+Backquote", "en")).toContain("`");
  });

  it("localizes the Space key", () => {
    expect(formatAccelerator("CommandOrControl+Space", "en")).toContain("Space");
    expect(formatAccelerator("CommandOrControl+Space", "pt-BR")).toContain(
      "Espaço",
    );
  });
});

describe("buildAccelerator", () => {
  it("falls back to event.key when event.code is empty", () => {
    const event = keyEvent({ code: "", key: "`", ctrlKey: true });

    expect(buildAccelerator(event)).toBe("CommandOrControl+Backquote");
  });

  it("uses the physical code for letters", () => {
    const event = keyEvent({
      code: "KeyV",
      key: "v",
      ctrlKey: true,
      shiftKey: true,
    });

    expect(buildAccelerator(event)).toBe("CommandOrControl+Shift+V");
  });
});

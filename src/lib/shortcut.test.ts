import { describe, expect, it } from "vitest";

import {
  buildAccelerator,
  DEFAULT_SHORTCUT,
  formatAccelerator,
  hasStrongModifier,
} from "./shortcut";

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

  it("formats modifiers, symbols and unknown keys", () => {
    const formatted = formatAccelerator("Ctrl+Alt+Shift+ArrowUp+KeyQ", "en");
    expect(formatted).toContain("↑");
    expect(formatted).toContain("KeyQ");
    expect(formatAccelerator("Super+1", "en")).toContain("1");
  });

  it("handles empty accelerators", () => {
    expect(formatAccelerator("", "en")).toBe("");
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

  it("supports digits, function keys and named keys", () => {
    expect(buildAccelerator(keyEvent({ code: "Digit4", key: "4", metaKey: true }))).toBe(
      "CommandOrControl+4",
    );
    expect(buildAccelerator(keyEvent({ code: "F12", key: "F12", altKey: true }))).toBe(
      "Alt+F12",
    );
    expect(buildAccelerator(keyEvent({ code: "Enter", key: "Enter" }))).toBe("Enter");
  });

  it("supports the event.key fallback for characters and named keys", () => {
    expect(buildAccelerator(keyEvent({ code: "", key: "q", ctrlKey: true }))).toBe(
      "CommandOrControl+Q",
    );
    expect(buildAccelerator(keyEvent({ code: "", key: "Escape" }))).toBe(
      "Escape",
    );
    expect(buildAccelerator(keyEvent({ code: "", key: "?" }))).toBeNull();
  });

  it("returns null for unsupported or modifier-only events", () => {
    expect(buildAccelerator(keyEvent({ code: "", key: "Unknown" }))).toBeNull();
    expect(buildAccelerator(keyEvent({ code: "ControlLeft", key: "Control" }))).toBeNull();
  });

  it("detects strong modifiers", () => {
    expect(hasStrongModifier(keyEvent({ code: "KeyV", key: "v", ctrlKey: true }))).toBe(
      true,
    );
    expect(hasStrongModifier(keyEvent({ code: "KeyV", key: "v", altKey: true }))).toBe(
      true,
    );
    expect(hasStrongModifier(keyEvent({ code: "KeyV", key: "v", shiftKey: true }))).toBe(
      false,
    );
    expect(DEFAULT_SHORTCUT).toBe("CommandOrControl+Shift+V");
  });
});

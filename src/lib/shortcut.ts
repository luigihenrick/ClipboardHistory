import type { Language } from "../i18n/types";

export const DEFAULT_SHORTCUT = "CommandOrControl+Shift+V";

export const isMac =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent);

const MODIFIER_LABELS: Record<string, string> = {
  COMMANDORCONTROL: isMac ? "⌘" : "Ctrl",
  COMMANDORCTRL: isMac ? "⌘" : "Ctrl",
  CMDORCTRL: isMac ? "⌘" : "Ctrl",
  CMDORCONTROL: isMac ? "⌘" : "Ctrl",
  CONTROL: isMac ? "⌃" : "Ctrl",
  CTRL: isMac ? "⌃" : "Ctrl",
  ALT: isMac ? "⌥" : "Alt",
  OPTION: isMac ? "⌥" : "Alt",
  SHIFT: isMac ? "⇧" : "Shift",
  COMMAND: isMac ? "⌘" : "Win",
  CMD: isMac ? "⌘" : "Win",
  META: isMac ? "⌘" : "Win",
  SUPER: isMac ? "⌘" : "Win",
};

const KEY_LABELS: Record<string, string> = {
  ARROWUP: "↑",
  UP: "↑",
  ARROWDOWN: "↓",
  DOWN: "↓",
  ARROWLEFT: "←",
  LEFT: "←",
  ARROWRIGHT: "→",
  RIGHT: "→",
  ENTER: "↵",
  SPACE: "Space",
  ESCAPE: "Esc",
  ESC: "Esc",
  BACKSPACE: "Backspace",
  TAB: "Tab",
  DELETE: "Del",
  PAGEUP: "PgUp",
  PAGEDOWN: "PgDn",
  BACKQUOTE: "`",
  MINUS: "-",
  EQUAL: "=",
  BRACKETLEFT: "[",
  BRACKETRIGHT: "]",
  BACKSLASH: "\\",
  SEMICOLON: ";",
  QUOTE: "'",
  COMMA: ",",
  PERIOD: ".",
  SLASH: "/",
};

/** Turns a Tauri accelerator such as `CommandOrControl+Shift+V` into something readable. */
export function formatAccelerator(
  accelerator: string,
  language: Language = "en",
): string {
  if (!accelerator) return "";

  const labels = accelerator
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => {
      const upper = part.toUpperCase();
      if (MODIFIER_LABELS[upper]) return MODIFIER_LABELS[upper];
      if (upper === "SPACE") return language === "pt-BR" ? "Espaço" : "Space";
      if (KEY_LABELS[upper]) return KEY_LABELS[upper];
      if (part.length === 1) return part.toUpperCase();
      return part;
    });

  return isMac ? labels.join(" ") : labels.join(" + ");
}

const CODE_KEYS: Record<string, string> = {
  Space: "Space",
  Enter: "Enter",
  NumpadEnter: "NumpadEnter",
  Tab: "Tab",
  Escape: "Escape",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  ArrowUp: "ArrowUp",
  ArrowDown: "ArrowDown",
  ArrowLeft: "ArrowLeft",
  ArrowRight: "ArrowRight",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backquote: "`",
  Numpad0: "Numpad0",
  Numpad1: "Numpad1",
  Numpad2: "Numpad2",
  Numpad3: "Numpad3",
  Numpad4: "Numpad4",
  Numpad5: "Numpad5",
  Numpad6: "Numpad6",
  Numpad7: "Numpad7",
  Numpad8: "Numpad8",
  Numpad9: "Numpad9",
  NumpadAdd: "NumpadAdd",
  NumpadSubtract: "NumpadSubtract",
  NumpadMultiply: "NumpadMultiply",
  NumpadDivide: "NumpadDivide",
  NumpadDecimal: "NumpadDecimal",
};

function keyFromCode(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  return CODE_KEYS[code] ?? null;
}

const CHAR_KEYS: Record<string, string> = {
  "`": "Backquote",
  "-": "Minus",
  "=": "Equal",
  "[": "BracketLeft",
  "]": "BracketRight",
  "\\": "Backslash",
  ";": "Semicolon",
  "'": "Quote",
  ",": "Comma",
  ".": "Period",
  "/": "Slash",
  " ": "Space",
};

const NAMED_KEYS: Record<string, string> = {
  Escape: "Escape",
  Esc: "Escape",
  Enter: "Enter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  ArrowUp: "ArrowUp",
  ArrowDown: "ArrowDown",
  ArrowLeft: "ArrowLeft",
  ArrowRight: "ArrowRight",
};

/**
 * Fallback for platforms/layouts where `event.code` is missing (some Linux dead-key layouts
 * report an empty code). Uses the logical key value instead.
 */
function keyFromKeyValue(key: string): string | null {
  if (key.length === 1) {
    if (/[a-z]/i.test(key)) return key.toUpperCase();
    if (/[0-9]/.test(key)) return key;
    return CHAR_KEYS[key] ?? null;
  }

  return NAMED_KEYS[key] ?? null;
}

/**
 * Builds a Tauri accelerator from a keyboard event, or `null` when the event is only a
 * modifier or a key we cannot express.
 */
export function buildAccelerator(event: KeyboardEvent): string | null {
  const key = keyFromCode(event.code) ?? keyFromKeyValue(event.key);
  if (!key) return null;

  const modifiers: string[] = [];
  if (event.ctrlKey || event.metaKey) modifiers.push("CommandOrControl");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");

  return [...modifiers, key].join("+");
}

export function hasStrongModifier(event: KeyboardEvent): boolean {
  return event.ctrlKey || event.metaKey || event.altKey;
}

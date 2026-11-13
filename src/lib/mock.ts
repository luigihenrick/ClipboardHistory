import type { Entry, Settings, Stats } from "../types";
import { emitPreviewEvent } from "./events";

/**
 * In-memory backend used only when the UI runs in a regular browser (`npm run dev` without
 * Tauri). It makes it possible to iterate on the layout and keyboard behavior without Rust.
 * Tauri builds never reach this module.
 */

const now = Date.now();
const minute = 60_000;
const hour = 60 * minute;
const day = 24 * hour;

function svgThumbnail(label: string): string {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="128" height="96">
    <rect width="128" height="96" fill="#d4d4d4"/>
    <text x="10" y="56" font-family="sans-serif" font-size="16" font-weight="700" fill="#525252">${label}</text>
  </svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

const storedLanguage =
  typeof localStorage !== "undefined" &&
  localStorage.getItem("cliphistory.language") === "pt-BR"
    ? "pt-BR"
    : "en";

let settings: Settings = {
  shortcut: "CommandOrControl+Shift+V",
  maxItems: 100,
  startAtLogin: false,
  hideOnBlur: true,
  autoPaste: true,
  language: storedLanguage,
};

let entries: Entry[] = [
  {
    id: 1,
    kind: "image",
    text: null,
    thumbnail: svgThumbnail("Dashboard"),
    width: 1280,
    height: 800,
    pinned: true,
    createdAt: now - 26 * minute,
  },
  {
    id: 2,
    kind: "text",
    text: "npm run tauri dev",
    thumbnail: null,
    width: null,
    height: null,
    pinned: true,
    createdAt: now - 48 * minute,
  },
  {
    id: 3,
    kind: "text",
    text: "ClipHistory home page: clipboard history, search, pinned items and a global shortcut on Windows, Linux and macOS.",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 3 * minute,
  },
  {
    id: 4,
    kind: "image",
    text: null,
    thumbnail: svgThumbnail("Screenshot"),
    width: 1440,
    height: 900,
    pinned: false,
    createdAt: now - 11 * minute,
  },
  {
    id: 5,
    kind: "text",
    text: "const historical = await getEntries(search);\n\nif (historical.length === 0) {\n  setLoading(false);\n}",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 37 * minute,
  },
  {
    id: 6,
    kind: "text",
    text: "#000000",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 2 * hour,
  },
  {
    id: 7,
    kind: "text",
    text: "128 Acacia Street — Apt 42\nBelo Horizonte, MG\n30140-000",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 5 * hour,
  },
  {
    id: 8,
    kind: "text",
    text: "Thanks for the feedback! I will review the changes and send the updated version by tomorrow morning.",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 1 * day - 2 * hour,
  },
  {
    id: 9,
    kind: "text",
    text: "pnpm create tauri-app --template react-ts",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: now - 3 * day,
  },
];

function sortEntries(): void {
  entries.sort((a, b) => b.createdAt - a.createdAt);
}

function emit(name: string, payload: unknown = null): void {
  emitPreviewEvent(name, payload);
}

export async function getEntries(search: string): Promise<Entry[]> {
  const term = search.trim().toLowerCase();

  if (!term) return [...entries];

  return entries.filter(
    (entry) =>
      entry.kind === "text" && (entry.text ?? "").toLowerCase().includes(term),
  );
}

export async function getSettings(): Promise<Settings> {
  return { ...settings };
}

export async function getStats(): Promise<Stats> {
  return {
    total: entries.length,
    pinned: entries.filter((entry) => entry.pinned).length,
    images: entries.filter((entry) => entry.kind === "image").length,
  };
}

export async function saveSettings(next: Settings): Promise<Settings> {
  settings = { ...next };
  emit("settings://changed", { ...settings });
  return { ...settings };
}

export async function setLanguage(language: Settings["language"]): Promise<Settings> {
  settings = { ...settings, language };
  emit("settings://changed", { ...settings });
  return { ...settings };
}

export async function shortcutStatus(): Promise<boolean> {
  return true;
}

export async function copyEntry(id: number): Promise<void> {
  const entry = entries.find((item) => item.id === id);
  if (entry) entry.createdAt = Date.now();
  sortEntries();
  // In the preview there is no real popup or paste simulation.
  console.info(
    "[preview] copiado:",
    entry?.kind,
    entry?.text ?? entry?.id,
    settings.autoPaste ? "(auto paste ligado)" : "",
  );
}

export async function togglePin(id: number): Promise<boolean> {
  const entry = entries.find((item) => item.id === id);
  if (!entry) return false;

  entry.pinned = !entry.pinned;
  sortEntries();
  emit("history://changed");
  return entry.pinned;
}

export async function deleteEntry(id: number): Promise<void> {
  entries = entries.filter((item) => item.id !== id);
  emit("history://changed");
}

export async function clearHistory(): Promise<number> {
  const before = entries.length;
  entries = entries.filter((item) => item.pinned);
  emit("history://changed");
  return before - entries.length;
}

export async function hidePopup(): Promise<void> {
  console.info("[preview] hide_popup");
}

export async function openSettings(): Promise<void> {
  window.open(
    `${window.location.pathname}?window=settings`,
    "_blank",
    "width=620,height=800",
  );
}

export async function hideSettings(): Promise<void> {
  window.close();
}

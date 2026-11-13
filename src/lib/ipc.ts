import { invoke } from "@tauri-apps/api/core";

import type { AppInfo, Entry, Settings, Stats } from "../types";
import { isTauri } from "./events";
import * as mock from "./mock";

// Dev-only UI preview when running `npm run dev` in a normal browser.
const preview = import.meta.env.DEV && !isTauri;

function call<T>(
  command: string,
  args: Record<string, unknown>,
  fallback: () => Promise<T>,
): Promise<T> {
  return preview ? fallback() : invoke<T>(command, args);
}

export function getEntries(search: string) {
  return call<Entry[]>("get_entries", { search: search.trim() || null }, () =>
    mock.getEntries(search),
  );
}

export function getAppInfo() {
  return call<AppInfo>(
    "get_app_info",
    {},
    // Mocked as Wayland so the fallback callout can be previewed in a browser.
    async () => ({
      platform: "linux",
      sessionType: "wayland",
      wayland: true,
      xwaylandAvailable: true,
      gdkBackend: "x11",
      executable: "/caminho/para/cliphistory",
    }),
  );
}

export function getSettings() {
  return call<Settings>("get_settings", {}, () => mock.getSettings());
}

export function getStats() {
  return call<Stats>("get_stats", {}, () => mock.getStats());
}

export function saveSettings(settings: Settings) {
  return call<Settings>("save_settings", { settings }, () =>
    mock.saveSettings(settings),
  );
}

export function setLanguage(language: Settings["language"]) {
  return call<Settings>("set_language", { language }, () =>
    mock.setLanguage(language),
  );
}

export function shortcutStatus() {
  return call<boolean>("shortcut_status", {}, () => mock.shortcutStatus());
}

export function copyEntry(id: number) {
  return call<void>("copy_entry", { id }, () => mock.copyEntry(id));
}

export function togglePin(id: number) {
  return call<boolean>("toggle_pin", { id }, () => mock.togglePin(id));
}

export function deleteEntry(id: number) {
  return call<void>("delete_entry", { id }, () => mock.deleteEntry(id));
}

export function clearHistory() {
  return call<number>("clear_history", {}, () => mock.clearHistory());
}

export function hidePopup() {
  return call<void>("hide_popup", {}, () => mock.hidePopup());
}

export function openSettings() {
  return call<void>("open_settings", {}, () => mock.openSettings());
}

export function hideSettings() {
  return call<void>("hide_settings", {}, () => mock.hideSettings());
}

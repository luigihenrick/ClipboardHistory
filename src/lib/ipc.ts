import { invoke } from "@tauri-apps/api/core";

import type { AppInfo, Entry, Settings, Stats, UpdateInfo } from "../types";
import { isTauri } from "./events";
import * as mock from "./mock";
import { checkForUpdate } from "./update";

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

function previewUpdate(): UpdateInfo | null {
  // In the browser preview, `?update=1` shows the update card for layout testing.
  if (!new URLSearchParams(window.location.search).has("update")) {
    return null;
  }

  const [major, minor, patch] = __APP_VERSION__.split("-")[0].split(".").map(Number);
  const previewVersion = `${major}.${minor}.${patch + 1}`;

  return {
    version: `v${previewVersion}`,
    name: `ClipHistory ${previewVersion}`,
    notes:
      "### What's new\n\n- Image previews in the clipboard list\n- More reliable auto-paste\n- Faster popup opening\n\n[MIT License](https://github.com/luigihenrick/ClipboardHistory/blob/main/LICENSE)",
    url: "https://github.com/luigihenrick/ClipboardHistory/releases/latest",
  };
}

export function getUpdate() {
  return call<UpdateInfo | null>("get_update", {}, async () => previewUpdate());
}

export function checkForUpdates() {
  return preview ? Promise.resolve(previewUpdate()) : checkForUpdate(__APP_VERSION__);
}

export function openExternal(url: string) {
  return call<void>("open_external_url", { url }, async () => {
    window.open(url, "_blank", "noopener,noreferrer");
  });
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

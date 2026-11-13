import type { Language } from "./i18n/types";

export interface Entry {
  id: number;
  kind: "text" | "image";
  text: string | null;
  thumbnail: string | null;
  width: number | null;
  height: number | null;
  pinned: boolean;
  /** Creation time in milliseconds since the Unix epoch. */
  createdAt: number;
}

export interface Settings {
  shortcut: string;
  maxItems: number;
  startAtLogin: boolean;
  hideOnBlur: boolean;
  autoPaste: boolean;
  language: Language;
}

export interface Stats {
  total: number;
  pinned: number;
  images: number;
}

export interface AppInfo {
  platform: string;
  sessionType: string | null;
  /** True on Wayland, where global key grabs are not available. */
  wayland: boolean;
  /** Whether an X11 display (XWayland) can actually be reached. */
  xwaylandAvailable: boolean;
  /** Explicit GDK backend override, when set. */
  gdkBackend: string | null;
  /** Absolute path to the running executable. */
  executable: string;
}

export interface UpdateInfo {
  version: string;
  name: string;
  notes: string;
  url: string;
}

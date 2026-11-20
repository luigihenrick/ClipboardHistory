import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Diagnostics, Settings as SettingsModel, UpdateInfo } from "../types";

const { eventHandlers } = vi.hoisted(() => ({
  eventHandlers: new Map<string, (payload?: unknown) => void>(),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("1.3.3"),
}));

vi.mock("../lib/events", () => ({
  onEvent: vi.fn(
    async (name: string, handler: (payload?: unknown) => void) => {
      eventHandlers.set(name, handler);
      return () => eventHandlers.delete(name);
    },
  ),
  isTauri: false,
}));

vi.mock("../lib/ipc", () => ({
  getSettings: vi.fn(),
  getStats: vi.fn(),
  shortcutStatus: vi.fn(),
  getAppInfo: vi.fn(),
  checkForUpdates: vi.fn(),
  getDiagnostics: vi.fn(),
  saveSettings: vi.fn(),
  setLanguage: vi.fn(),
  clearHistory: vi.fn(),
  openExternal: vi.fn(),
  copyDiagnostics: vi.fn(),
  openLogsFolder: vi.fn(),
  hideSettings: vi.fn(),
}));

import * as api from "../lib/ipc";
import { I18nProvider } from "../i18n";
import Settings from "./Settings";

const initialSettings: SettingsModel = {
  shortcut: "CommandOrControl+Shift+V",
  maxItems: 100,
  startAtLogin: false,
  hideOnBlur: true,
  autoPaste: true,
  language: "en",
};

const update: UpdateInfo = {
  version: "v1.3.4",
  name: "ClipHistory 1.3.4",
  notes: "Release notes",
  url: "https://github.com/luigihenrick/ClipboardHistory/releases/tag/v1.3.4",
};

const diagnostics: Diagnostics = {
  appVersion: "1.3.3",
  tauriVersion: "2",
  summary: "diagnostics-summary",
  recentLogs: "log-line",
  logDir: "/tmp/logs",
  generatedAtMs: 1,
};

function renderSettings() {
  return render(
    <I18nProvider>
      <Settings />
    </I18nProvider>,
  );
}

describe("Settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    eventHandlers.clear();
    vi.mocked(api.getSettings).mockResolvedValue(initialSettings);
    vi.mocked(api.getStats).mockResolvedValue({ total: 9, pinned: 2, images: 2 });
    vi.mocked(api.shortcutStatus).mockResolvedValue(true);
    vi.mocked(api.getAppInfo).mockResolvedValue({
      platform: "linux",
      sessionType: "wayland",
      wayland: true,
      xwaylandAvailable: true,
      gdkBackend: "x11",
      executable: "/usr/bin/cliphistory",
    });
    vi.mocked(api.checkForUpdates).mockResolvedValue(update);
    vi.mocked(api.getDiagnostics).mockResolvedValue(diagnostics);
    vi.mocked(api.saveSettings).mockImplementation(async (settings) => settings);
    vi.mocked(api.setLanguage).mockImplementation(async (language) => ({
      ...initialSettings,
      language,
    }));
    vi.mocked(api.clearHistory).mockResolvedValue(3);
    vi.mocked(api.openExternal).mockResolvedValue();
    vi.mocked(api.copyDiagnostics).mockResolvedValue();
    vi.mocked(api.openLogsFolder).mockResolvedValue();
    vi.mocked(api.hideSettings).mockResolvedValue();
  });

  it("shows the updates card when no update is available", async () => {
    vi.mocked(api.checkForUpdates).mockResolvedValueOnce(null);
    renderSettings();

    expect(await screen.findByText("Updates")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Open GitHub release" }),
    ).not.toBeInTheDocument();
  });

  it("handles unavailable diagnostics and action errors", async () => {
    vi.mocked(api.getDiagnostics).mockRejectedValueOnce(new Error("no diag"));
    vi.mocked(api.copyDiagnostics).mockRejectedValueOnce(new Error("copy failed"));
    vi.mocked(api.openLogsFolder).mockRejectedValueOnce(new Error("open failed"));
    vi.mocked(api.checkForUpdates).mockRejectedValue(new Error("check failed"));

    renderSettings();

    expect(await screen.findByText("Diagnostics unavailable.")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Copy diagnostics"));
    expect(
      await screen.findByText("Could not copy diagnostics."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByText("Open logs"));
    expect(
      await screen.findByText("Could not open the logs folder."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Check for updates" }));
    expect(
      await screen.findByText("Could not check for updates. Try again later."),
    ).toBeInTheDocument();
  });

  it("loads settings, stats, version and update information", async () => {
    renderSettings();

    expect(await screen.findByText("Update available")).toBeInTheDocument();
    expect(screen.getByText("v1.3.4")).toBeInTheDocument();
    expect(screen.getByText("v1.3.3 · Tauri 2")).toBeInTheDocument();
    expect(screen.getByText("9")).toBeInTheDocument();
    expect(screen.getAllByText("2")).toHaveLength(2);
    expect(screen.getByText("Ctrl + Shift + V")).toBeInTheDocument();
  });

  it("saves a history limit above the old 100-item display cap", async () => {
    renderSettings();
    await screen.findByRole("button", { name: "Save changes" });

    const slider = screen.getByRole("slider", {
      name: "Maximum number of items in history",
    });
    fireEvent.change(slider, { target: { value: "300" } });
    expect(screen.getByText("300")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Save changes" }));
    await waitFor(() =>
      expect(api.saveSettings).toHaveBeenCalledWith(
        expect.objectContaining({ maxItems: 300 }),
      ),
    );
  });

  it("changes language and saves settings", async () => {
    renderSettings();
    await screen.findByRole("button", { name: "Save changes" });

    fireEvent.click(screen.getByRole("button", { name: "Português" }));
    await waitFor(() =>
      expect(api.setLanguage).toHaveBeenCalledWith("pt-BR"),
    );

    fireEvent.click(screen.getByRole("button", { name: "Salvar alterações" }));
    await waitFor(() => expect(api.saveSettings).toHaveBeenCalled());
    expect(await screen.findByText("Configurações salvas.")).toBeInTheDocument();
  });

  it("clears history after confirmation", async () => {
    renderSettings();
    await screen.findByRole("button", { name: "Clear" });

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => expect(api.clearHistory).toHaveBeenCalled());
    expect(await screen.findByText("3 items removed.")).toBeInTheDocument();
  });

  it("opens the release page and checks for updates manually", async () => {
    renderSettings();
    await screen.findByRole("button", { name: "Open GitHub release" });

    fireEvent.click(screen.getByRole("button", { name: "Open GitHub release" }));
    await waitFor(() =>
      expect(api.openExternal).toHaveBeenCalledWith(update.url),
    );

    vi.mocked(api.checkForUpdates).mockResolvedValueOnce(null);
    fireEvent.click(screen.getByRole("button", { name: "Check for updates" }));

    await waitFor(() => expect(api.checkForUpdates).toHaveBeenCalled());
    expect(
      await screen.findByText("You're on the latest version."),
    ).toBeInTheDocument();
  });

  it("copies diagnostics and opens the logs folder", async () => {
    renderSettings();
    await screen.findByText("diagnostics-summary");

    fireEvent.click(screen.getByText("Copy diagnostics"));
    await waitFor(() => expect(api.copyDiagnostics).toHaveBeenCalled());
    expect(await screen.findByText("Diagnostics copied.")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Open logs"));
    await waitFor(() => expect(api.openLogsFolder).toHaveBeenCalled());

    fireEvent.click(screen.getByText("Refresh"));
    await waitFor(() =>
      expect(api.getDiagnostics).toHaveBeenCalledTimes(2),
    );
  });

  it("updates switches and closes with Escape", async () => {
    renderSettings();
    await screen.findByRole("button", { name: "Save changes" });

    const autoPaste = screen.getByRole("checkbox", { name: /Paste automatically/ });
    fireEvent.click(autoPaste);
    expect(autoPaste).not.toBeChecked();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(api.hideSettings).toHaveBeenCalled();
  });

  it("reacts to an update event", async () => {
    renderSettings();
    await screen.findByText("Update available");

    eventHandlers.get("update://available")?.({
      ...update,
      version: "v1.4.0",
    });

    expect(await screen.findByText("v1.4.0")).toBeInTheDocument();
  });
});

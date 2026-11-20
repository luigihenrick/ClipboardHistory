import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Entry, Settings } from "../types";

const { eventHandlers } = vi.hoisted(() => ({
  eventHandlers: new Map<string, (payload?: unknown) => void>(),
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
  getEntries: vi.fn(),
  getSettings: vi.fn(),
  copyEntry: vi.fn(),
  togglePin: vi.fn(),
  deleteEntry: vi.fn(),
  hidePopup: vi.fn(),
  openSettings: vi.fn(),
  reportSearchFocus: vi.fn(),
}));

import * as api from "../lib/ipc";
import { I18nProvider } from "../i18n";
import Popup from "./Popup";

const settings: Settings = {
  shortcut: "CommandOrControl+Shift+V",
  maxItems: 100,
  startAtLogin: false,
  hideOnBlur: true,
  autoPaste: true,
  language: "en",
};

const entries: Entry[] = [
  {
    id: 1,
    kind: "image",
    text: null,
    thumbnail: "data:image/png;base64,abc",
    width: 1280,
    height: 800,
    pinned: true,
    createdAt: Date.now() - 60_000,
  },
  {
    id: 2,
    kind: "text",
    text: "hello world",
    thumbnail: null,
    width: null,
    height: null,
    pinned: false,
    createdAt: Date.now() - 120_000,
  },
];

function renderPopup() {
  return render(
    <I18nProvider>
      <Popup />
    </I18nProvider>,
  );
}

describe("Popup", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    eventHandlers.clear();
    vi.mocked(api.getEntries).mockResolvedValue(entries);
    vi.mocked(api.getSettings).mockResolvedValue(settings);
    vi.mocked(api.copyEntry).mockResolvedValue();
    vi.mocked(api.togglePin).mockResolvedValue(true);
    vi.mocked(api.deleteEntry).mockResolvedValue();
    vi.mocked(api.hidePopup).mockResolvedValue();
    vi.mocked(api.openSettings).mockResolvedValue();
    vi.mocked(api.reportSearchFocus).mockResolvedValue();
  });

  it("loads and renders text and image entries", async () => {
    renderPopup();

    expect(await screen.findByText("hello world")).toBeInTheDocument();
    expect(screen.getByText("Image")).toBeInTheDocument();
    expect(screen.getByText("1280 × 800")).toBeInTheDocument();
    expect(screen.getByText("2 items")).toBeInTheDocument();
    expect(screen.getByText("Ctrl + Shift + V")).toBeInTheDocument();
  });

  it("debounces search and shows an empty result state", async () => {
    vi.mocked(api.getEntries).mockImplementation(async (term) =>
      term ? [] : entries,
    );
    renderPopup();
    await screen.findByText("hello world");

    fireEvent.change(screen.getByPlaceholderText("Search history…"), {
      target: { value: "missing" },
    });

    await waitFor(() => expect(api.getEntries).toHaveBeenCalledWith("missing"), {
      timeout: 1000,
    });
    expect(await screen.findByText("No results")).toBeInTheDocument();
    expect(screen.getByText("Try a different search term.")).toBeInTheDocument();
  });

  it("shows the empty history state", async () => {
    vi.mocked(api.getEntries).mockResolvedValue([]);
    renderPopup();

    expect(await screen.findByText("Nothing here yet")).toBeInTheDocument();
    expect(
      screen.getByText("Copy anything and ClipHistory will keep it for you."),
    ).toBeInTheDocument();
  });

  it("copies entries by click and keyboard, and supports navigation", async () => {
    renderPopup();
    await screen.findByText("hello world");

    fireEvent.click(screen.getByText("hello world"));
    expect(api.copyEntry).toHaveBeenCalledWith(2);

    fireEvent.keyDown(window, { key: "ArrowDown" });
    fireEvent.keyDown(window, { key: "Enter" });
    expect(api.copyEntry).toHaveBeenCalledWith(2);

    fireEvent.keyDown(window, { key: "ArrowUp" });
    fireEvent.keyDown(window, { key: "Enter" });
    expect(api.copyEntry).toHaveBeenCalledWith(1);
  });

  it("pins and deletes entries from the row actions", async () => {
    renderPopup();
    await screen.findByText("hello world");

    const pinButtons = screen.getAllByLabelText("Pin to top");
    fireEvent.click(pinButtons[0]);
    await waitFor(() => expect(api.togglePin).toHaveBeenCalledWith(2));

    const deleteButtons = screen.getAllByLabelText("Remove from history");
    fireEvent.click(deleteButtons[1]);
    await waitFor(() => expect(api.deleteEntry).toHaveBeenCalledWith(2));
  });

  it("opens settings, clears search and closes the popup", async () => {
    renderPopup();
    await screen.findByText("hello world");

    fireEvent.change(screen.getByPlaceholderText("Search history…"), {
      target: { value: "hello" },
    });
    fireEvent.click(screen.getByLabelText("Clear search"));
    expect(screen.getByPlaceholderText("Search history…")).toHaveValue("");

    fireEvent.click(screen.getByLabelText("Open settings"));
    expect(api.openSettings).toHaveBeenCalled();

    fireEvent.click(screen.getByLabelText("Close"));
    expect(api.hidePopup).toHaveBeenCalled();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(api.hidePopup).toHaveBeenCalledTimes(2);
  });

  it("reacts to popup, history and settings events", async () => {
    renderPopup();
    await screen.findByText("hello world");
    await waitFor(() => expect(eventHandlers.has("popup://shown")).toBe(true));

    eventHandlers.get("popup://shown")?.();
    expect(api.getEntries).toHaveBeenCalled();

    eventHandlers.get("history://changed")?.();
    eventHandlers.get("settings://changed")?.({
      ...settings,
      language: "pt-BR",
    });

    expect(
      await screen.findByPlaceholderText("Buscar no histórico…"),
    ).toBeInTheDocument();
  });

  it("reports search focus timing when the input is focused", async () => {
    renderPopup();
    await screen.findByText("hello world");

    eventHandlers.get("popup://shown")?.();
    const input = screen.getByPlaceholderText("Search history…");
    fireEvent.focus(input);

    expect(api.reportSearchFocus).toHaveBeenCalled();
  });
});

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { I18nProvider } from "../i18n";
import { HotkeyRecorder } from "./HotkeyRecorder";

function renderRecorder(value = "CommandOrControl+Shift+V", onChange = vi.fn()) {
  render(
    <I18nProvider>
      <HotkeyRecorder value={value} onChange={onChange} />
    </I18nProvider>,
  );
  return onChange;
}

describe("HotkeyRecorder", () => {
  it("renders the current shortcut and records a keyboard combination", () => {
    const onChange = renderRecorder();

    expect(screen.getByText("Ctrl + Shift + V")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    fireEvent.keyDown(window, {
      code: "KeyK",
      key: "k",
      ctrlKey: true,
      shiftKey: true,
    });

    expect(onChange).toHaveBeenCalledWith("CommandOrControl+Shift+K");
  });

  it("shows a hint when the combination lacks a strong modifier", () => {
    renderRecorder();

    fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    fireEvent.keyDown(window, { code: "KeyK", key: "k" });

    expect(screen.getByText(/Use Ctrl, Alt, or ⌘ together/)).toBeInTheDocument();
  });

  it("cancels recording with Escape", () => {
    renderRecorder();

    fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
    expect(screen.getByText("Press the combination…")).toBeInTheDocument();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByText("Press the combination…")).not.toBeInTheDocument();
  });

  it("validates manual shortcuts", () => {
    const onChange = renderRecorder();
    const input = screen.getByLabelText("Or type the shortcut");
    const apply = screen.getByRole("button", { name: "Use shortcut" });

    fireEvent.change(input, { target: { value: "" } });
    fireEvent.click(apply);
    expect(screen.getByText(/Type a shortcut/)).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "V" } });
    fireEvent.click(apply);
    expect(screen.getByText(/Use the format Modifier\+Key/)).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "KeyV+KeyK" } });
    fireEvent.click(apply);
    expect(screen.getByText(/Include at least one modifier/)).toBeInTheDocument();

    fireEvent.change(input, {
      target: { value: "CommandOrControl+Shift+K" },
    });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith("CommandOrControl+Shift+K");
  });

  it("applies presets and restores the default", () => {
    const onChange = renderRecorder("Ctrl+K");

    fireEvent.click(screen.getByRole("button", { name: /Use Ctrl \+ `/ }));
    expect(onChange).toHaveBeenCalledWith("CommandOrControl+Backquote");

    fireEvent.click(screen.getByRole("button", { name: "Restore default" }));
    expect(onChange).toHaveBeenCalledWith("CommandOrControl+Shift+V");
  });
});

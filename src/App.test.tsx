import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("./windows/Popup", () => ({
  default: () => <div>popup-window</div>,
}));
vi.mock("./windows/Settings", () => ({
  default: () => <div>settings-window</div>,
}));

import App from "./App";

describe("App", () => {
  it("renders the popup for the popup label", () => {
    render(<App windowLabel="popup" />);
    expect(screen.getByText("popup-window")).toBeInTheDocument();
  });

  it("renders settings for the settings label", () => {
    render(<App windowLabel="settings" />);
    expect(screen.getByText("settings-window")).toBeInTheDocument();
  });
});

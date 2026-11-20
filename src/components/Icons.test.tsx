import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
  CheckIcon,
  ClipboardIcon,
  CloseIcon,
  KeyboardIcon,
  Logo,
  PinIcon,
  SearchIcon,
  SettingsIcon,
  TrashIcon,
  WarningIcon,
} from "./Icons";

describe("icons", () => {
  it("renders every icon with default and custom props", () => {
    const { container } = render(
      <div>
        <Logo />
        <Logo size={32} className="custom" />
        <SearchIcon />
        <SettingsIcon />
        <CloseIcon />
        <PinIcon />
        <PinIcon filled />
        <TrashIcon />
        <CheckIcon />
        <KeyboardIcon />
        <ClipboardIcon />
        <WarningIcon />
      </div>,
    );

    expect(container.querySelectorAll("svg")).toHaveLength(12);
    expect(container.querySelector(".custom")).toBeInTheDocument();
  });
});

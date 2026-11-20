import { render, screen, fireEvent } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { I18nProvider, useI18n } from "./index";

function Probe() {
  const { language, setLanguage, t } = useI18n();

  return (
    <div>
      <span data-testid="language">{language}</span>
      <span data-testid="translated">{t("common.close")}</span>
      <button type="button" onClick={() => setLanguage("pt-BR")}>
        Portuguese
      </button>
      <button type="button" onClick={() => setLanguage("fr" as never)}>
        Invalid
      </button>
    </div>
  );
}

describe("I18nProvider", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.lang = "";
  });

  it("provides translations and persists language changes", () => {
    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>,
    );

    expect(screen.getByTestId("language")).toHaveTextContent("en");
    expect(screen.getByTestId("translated")).toHaveTextContent("Close");

    fireEvent.click(screen.getByText("Portuguese"));

    expect(screen.getByTestId("language")).toHaveTextContent("pt-BR");
    expect(screen.getByTestId("translated")).toHaveTextContent("Fechar");
    expect(localStorage.getItem("cliphistory.language")).toBe("pt-BR");
    expect(document.documentElement.lang).toBe("pt-BR");
  });

  it("falls back to English for an invalid language", () => {
    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>,
    );

    fireEvent.click(screen.getByText("Invalid"));
    expect(screen.getByTestId("language")).toHaveTextContent("en");
  });

  it("reads a stored Portuguese language", () => {
    localStorage.setItem("cliphistory.language", "pt-BR");

    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>,
    );

    expect(screen.getByTestId("language")).toHaveTextContent("pt-BR");
  });

  it("requires a provider", () => {
    expect(() => render(<Probe />)).toThrow("useI18n must be used inside I18nProvider");
  });
});

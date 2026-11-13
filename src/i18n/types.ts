export type Language = "en" | "pt-BR";

export const LANGUAGES: Language[] = ["en", "pt-BR"];

export function isLanguage(value: unknown): value is Language {
  return value === "en" || value === "pt-BR";
}

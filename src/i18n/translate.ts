import { en, ptBR, type TranslationKey } from "./dictionaries";
import type { Language } from "./types";

export function translate(
  language: Language,
  key: TranslationKey,
  params?: Record<string, string | number>,
): string {
  const table: Record<TranslationKey, string> = language === "pt-BR" ? ptBR : en;
  const template = table[key] ?? en[key];

  if (!params) return template;

  return template.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}

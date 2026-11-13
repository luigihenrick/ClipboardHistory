import { translate } from "../i18n/translate";
import type { Language } from "../i18n/types";

const locale = (language: Language) => (language === "pt-BR" ? "pt-BR" : "en-US");

/** Collapses newlines, tabs and repeated spaces so previews stay on a single paragraph. */
export function previewText(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

export function charLabel(length: number, language: Language): string {
  const count = new Intl.NumberFormat(locale(language)).format(length);
  return translate(language, length === 1 ? "chars.one" : "chars.many", {
    count,
  });
}

export function relativeTime(
  timestamp: number,
  language: Language,
  now = Date.now(),
): string {
  const diff = Math.max(0, now - timestamp);
  const minute = 60_000;
  const hour = 3_600_000;
  const day = 86_400_000;

  if (diff < minute) return translate(language, "time.now");
  if (diff < hour) {
    return translate(language, "time.minutes", {
      count: Math.floor(diff / minute),
    });
  }
  if (diff < day) {
    return translate(language, "time.hours", {
      count: Math.floor(diff / hour),
    });
  }
  if (diff < 7 * day) {
    const days = Math.floor(diff / day);
    return days === 1
      ? translate(language, "time.yesterday")
      : translate(language, "time.days", { count: days });
  }

  return new Date(timestamp).toLocaleDateString(locale(language), {
    day: "2-digit",
    month: "short",
  });
}

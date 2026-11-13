import type { Language } from "./types";

const ENGLISH_TO_PORTUGUESE: Array<[string, string]> = [
  [
    "Invalid shortcut. Use combinations such as",
    "Atalho inválido. Use combinações como",
  ],
  [
    "The shortcut must include Ctrl, Alt, or ⌘/Super.",
    "O atalho precisa incluir Ctrl, Alt ou ⌘/Super.",
  ],
  ["Could not register", "Não foi possível registrar"],
  [
    "Failed to configure launch at login:",
    "Falha ao configurar início automático:",
  ],
  ["Settings unavailable.", "Configuração indisponível."],
  ["Item not found.", "Item não encontrado."],
  ["Database unavailable.", "Banco de dados indisponível."],
  ["Invalid image.", "Imagem inválida."],
];

/**
 * Best-effort translation for user-facing errors coming from the Rust backend.
 * Unknown messages (OS/SQLite details) are returned unchanged.
 */
export function localizeError(message: string, language: Language): string {
  if (language === "en") return message;

  for (const [from, to] of ENGLISH_TO_PORTUGUESE) {
    if (message.startsWith(from)) {
      return to + message.slice(from.length);
    }
  }

  return message;
}

import { useEffect, useState } from "react";

import { useI18n } from "../i18n";
import {
  buildAccelerator,
  DEFAULT_SHORTCUT,
  formatAccelerator,
  hasStrongModifier,
} from "../lib/shortcut";

/** Useful preset for Linux layouts where the recorder cannot capture Ctrl+` (dead keys). */
const BACKQUOTE_SHORTCUT = "CommandOrControl+Backquote";

const MODIFIERS = new Set([
  "COMMANDORCONTROL",
  "COMMANDORCTRL",
  "CMDORCTRL",
  "CMDORCONTROL",
  "COMMAND",
  "CMD",
  "CONTROL",
  "CTRL",
  "ALT",
  "OPTION",
  "SHIFT",
  "SUPER",
  "META",
]);

interface HotkeyRecorderProps {
  value: string;
  onChange: (accelerator: string) => void;
}

export function HotkeyRecorder({ value, onChange }: HotkeyRecorderProps) {
  const { t, language } = useI18n();
  const [recording, setRecording] = useState(false);
  const [hint, setHint] = useState<string | null>(null);
  const [manual, setManual] = useState(value);

  useEffect(() => {
    setManual(value);
  }, [value]);

  useEffect(() => {
    if (!recording) return;

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        setRecording(false);
        setHint(null);
        return;
      }

      const accelerator = buildAccelerator(event);
      if (!accelerator) return;

      if (!hasStrongModifier(event)) {
        setHint(t("hotkey.modifierRequired"));
        return;
      }

      onChange(accelerator);
      setRecording(false);
      setHint(null);
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [onChange, recording, t]);

  const applyManual = () => {
    const candidate = manual.trim();

    if (!candidate) {
      setHint(t("hotkey.empty"));
      return;
    }

    const tokens = candidate
      .split("+")
      .map((token) => token.trim())
      .filter(Boolean);
    const last = tokens[tokens.length - 1]?.toUpperCase();

    if (tokens.length < 2 || !last || MODIFIERS.has(last)) {
      setHint(t("hotkey.invalidFormat"));
      return;
    }

    if (!tokens.slice(0, -1).some((token) => MODIFIERS.has(token.toUpperCase()))) {
      setHint(t("hotkey.needsModifier"));
      return;
    }

    onChange(candidate);
    setHint(null);
  };

  return (
    <div className="hotkey-field">
      <div className={`hotkey-display${recording ? " recording" : ""}`}>
        {recording ? (
          <span className="hotkey-recording">{t("hotkey.press")}</span>
        ) : (
          <kbd>{formatAccelerator(value, language)}</kbd>
        )}
      </div>

      <div className="hotkey-actions">
        <button
          type="button"
          className="btn btn-ghost"
          onClick={() => {
            setHint(null);
            setRecording((current) => !current);
          }}
        >
          {recording ? t("common.cancel") : t("hotkey.record")}
        </button>

        {value !== BACKQUOTE_SHORTCUT && (
          <button
            type="button"
            className="btn btn-quiet"
            onClick={() => {
              setRecording(false);
              setHint(null);
              onChange(BACKQUOTE_SHORTCUT);
            }}
          >
            {t("hotkey.usePreset", {
              shortcut: formatAccelerator(BACKQUOTE_SHORTCUT, language),
            })}
          </button>
        )}

        {value !== DEFAULT_SHORTCUT && (
          <button
            type="button"
            className="btn btn-quiet"
            onClick={() => {
              setRecording(false);
              setHint(null);
              onChange(DEFAULT_SHORTCUT);
            }}
          >
            {t("hotkey.restoreDefault")}
          </button>
        )}
      </div>

      {hint && <p className="field-hint field-hint-error">{hint}</p>}
      {recording && <p className="field-hint">{t("hotkey.recordingHint")}</p>}

      <div className="manual-shortcut">
        <label className="field-label" htmlFor="manual-shortcut">
          {t("hotkey.manualLabel")}
        </label>
        <div className="manual-row">
          <input
            id="manual-shortcut"
            className="text-input"
            type="text"
            value={manual}
            spellCheck={false}
            autoComplete="off"
            placeholder="CommandOrControl+Backquote"
            onFocus={() => {
              setRecording(false);
              setHint(null);
            }}
            onChange={(event) => setManual(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                applyManual();
              }
            }}
          />
          <button type="button" className="btn btn-ghost" onClick={applyManual}>
            {t("hotkey.apply")}
          </button>
        </div>
        <p className="field-hint">{t("hotkey.manualHint")}</p>
      </div>
    </div>
  );
}

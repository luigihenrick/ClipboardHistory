import { Suspense, lazy, useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

import { HotkeyRecorder } from "../components/HotkeyRecorder";
import { CheckIcon, CloseIcon, KeyboardIcon, Logo, WarningIcon } from "../components/Icons";
import { useI18n, type Language } from "../i18n";
import { localizeError } from "../i18n/errors";
import { isLanguage } from "../i18n/types";
import { onEvent } from "../lib/events";
import * as api from "../lib/ipc";
import { isMac } from "../lib/shortcut";
import type {
  AppInfo,
  Diagnostics,
  Settings as SettingsModel,
  Stats,
  UpdateInfo,
} from "../types";

const Markdown = lazy(() => import("react-markdown"));

const isLinux = typeof navigator !== "undefined" && /Linux/.test(navigator.userAgent);

type Status = { kind: "success" | "error"; message: string } | null;

export default function Settings() {
  const { t, language, setLanguage: applyLanguage } = useI18n();
  const [form, setForm] = useState<SettingsModel | null>(null);
  const [stats, setStats] = useState<Stats | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [shortcutOk, setShortcutOk] = useState(true);
  const [status, setStatus] = useState<Status>(null);
  const [saving, setSaving] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [version, setVersion] = useState("");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [diagnosticsLoading, setDiagnosticsLoading] = useState(false);

  const loadStats = useCallback(async () => {
    try {
      setStats(await api.getStats());
    } catch (error) {
      console.error("cliphistory: failed to load stats", error);
    }
  }, []);

  const loadDiagnostics = useCallback(async () => {
    setDiagnosticsLoading(true);
    try {
      setDiagnostics(await api.getDiagnostics());
    } catch (error) {
      console.error("cliphistory: failed to load diagnostics", error);
    } finally {
      setDiagnosticsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadDiagnostics();
  }, [loadDiagnostics]);

  useEffect(() => {
    let disposed = false;

    const load = async () => {
      try {
        const [loadedSettings, loadedStats, registered] = await Promise.all([
          api.getSettings(),
          api.getStats(),
          api.shortcutStatus(),
        ]);
        if (disposed) return;

        const normalized: SettingsModel = {
          ...loadedSettings,
          language: isLanguage(loadedSettings.language)
            ? loadedSettings.language
            : "en",
        };

        setForm(normalized);
        setStats(loadedStats);
        setShortcutOk(registered);
        applyLanguage(normalized.language);

        // Optional: older running backends may not expose this command yet.
        try {
          const info = await api.getAppInfo();
          if (!disposed) setAppInfo(info);
        } catch (error) {
          console.warn("cliphistory: environment info unavailable", error);
        }
      } catch (error) {
        console.error("cliphistory: failed to load settings", error);
        if (!disposed) {
          setStatus({ kind: "error", message: String(error) });
        }
      }
    };

    void load();
    void getVersion()
      .then((value) => {
        if (!disposed) setVersion(value);
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
    };
  }, [applyLanguage]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void api
      .checkForUpdates()
      .then((available) => {
        if (!disposed) setUpdateInfo(available);
      })
      .catch((error) => console.error("cliphistory: failed to check for updates", error));

    void onEvent<UpdateInfo>("update://available", (payload) => {
      setUpdateInfo(payload);
    }).then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const checkForUpdates = useCallback(async () => {
    if (checkingUpdates) return;

    setCheckingUpdates(true);
    setStatus(null);

    try {
      const available = await api.checkForUpdates();
      setUpdateInfo(available);
      if (!available) {
        setStatus({ kind: "success", message: t("settings.upToDate") });
      }
    } catch (error) {
      console.error("cliphistory: failed to check for updates", error);
      setStatus({ kind: "error", message: t("settings.updateCheckFailed") });
    } finally {
      setCheckingUpdates(false);
    }
  }, [checkingUpdates, t]);

  const update = (patch: Partial<SettingsModel>) => {
    setStatus(null);
    setForm((current) => (current ? { ...current, ...patch } : current));
  };

  const changeLanguage = async (next: Language) => {
    if (!form || form.language === next) return;

    const previous = form.language;
    setStatus(null);
    setForm((current) => (current ? { ...current, language: next } : current));
    applyLanguage(next);

    try {
      const saved = await api.setLanguage(next);
      setForm((current) =>
        current ? { ...current, language: saved.language } : current,
      );
      applyLanguage(saved.language);
    } catch (error) {
      setForm((current) =>
        current ? { ...current, language: previous } : current,
      );
      applyLanguage(previous);
      setStatus({
        kind: "error",
        message: localizeError(String(error), previous),
      });
    }
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void api.hideSettings();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const save = async () => {
    if (!form || saving) return;

    setSaving(true);
    setStatus(null);

    try {
      const saved = await api.saveSettings(form);
      setForm(saved);
      applyLanguage(isLanguage(saved.language) ? saved.language : language);
      setShortcutOk(await api.shortcutStatus());
      await loadStats();
      setStatus({ kind: "success", message: t("settings.saved") });
    } catch (error) {
      setStatus({
        kind: "error",
        message: localizeError(String(error), language),
      });
    } finally {
      setSaving(false);
    }
  };

  const copyDiagnostics = async () => {
    try {
      await api.copyDiagnostics();
      setStatus({ kind: "success", message: t("settings.diagnosticsCopied") });
    } catch (error) {
      console.error("cliphistory: failed to copy diagnostics", error);
      setStatus({ kind: "error", message: t("settings.diagnosticsCopyFailed") });
    }
  };

  const openLogs = async () => {
    try {
      await api.openLogsFolder();
    } catch (error) {
      console.error("cliphistory: failed to open logs", error);
      setStatus({ kind: "error", message: t("settings.diagnosticsLogsFailed") });
    }
  };

  const openRelease = async () => {
    if (!updateInfo) return;

    try {
      await api.openExternal(updateInfo.url);
    } catch (error) {
      console.error("cliphistory: failed to open release page", error);
      setStatus({ kind: "error", message: t("settings.updateOpenFailed") });
    }
  };

  const clear = async () => {
    if (!confirmClear) {
      setConfirmClear(true);
      return;
    }

    setClearing(true);
    setStatus(null);

    try {
      const removed = await api.clearHistory();
      setConfirmClear(false);
      await loadStats();
      setStatus({
        kind: "success",
        message:
          removed === 0
            ? t("settings.nothingToClear")
            : t(
                removed === 1
                  ? "settings.removedOne"
                  : "settings.removedMany",
                { count: removed },
              ),
      });
    } catch (error) {
      setStatus({
        kind: "error",
        message: localizeError(String(error), language),
      });
    } finally {
      setClearing(false);
    }
  };

  if (!form) {
    return (
      <div className="settings-window">
        <div className="settings-loading">
          <span className="spinner" />
          {t("common.loading")}
        </div>
      </div>
    );
  }

  return (
    <div className="settings-window">
      <header className="settings-header">
        <div className="settings-brand">
          <Logo size={44} />
          <div>
            <h1>ClipHistory</h1>
            <p>{t("settings.tagline")}</p>
          </div>
        </div>

        <div className="settings-header-actions">
          <div
            className="language-switch"
            role="group"
            aria-label={t("language.label")}
          >
            <button
              type="button"
              className={language === "en" ? "active" : ""}
              aria-pressed={language === "en"}
              onClick={() => void changeLanguage("en")}
            >
              {t("language.english")}
            </button>
            <button
              type="button"
              className={language === "pt-BR" ? "active" : ""}
              aria-pressed={language === "pt-BR"}
              onClick={() => void changeLanguage("pt-BR")}
            >
              {t("language.portuguese")}
            </button>
          </div>

          <button
            type="button"
            className="icon-btn"
            title={t("common.close")}
            aria-label={t("common.close")}
            onClick={() => void api.hideSettings()}
          >
            <CloseIcon size={16} />
          </button>
        </div>
      </header>

      <div className="settings-content">
        <section className={`card update-card${updateInfo ? " update-available" : ""}`}>
          <div className="update-heading">
            <div>
              <h2>{updateInfo ? t("settings.updateTitle") : t("settings.updatesTitle")}</h2>
              <p className="card-description">
                {updateInfo ? t("settings.updateText") : t("settings.updatesText")}
              </p>
            </div>
            {updateInfo && <span className="update-version">{updateInfo.version}</span>}
          </div>

          {updateInfo?.notes && (
            <Suspense fallback={<pre className="update-notes">{updateInfo.notes}</pre>}>
              <div className="update-notes">
                <Markdown
                  components={{
                    a: ({ href, children }) => (
                      <a
                        href={href}
                        onClick={(event) => {
                          event.preventDefault();
                          if (href?.startsWith("https://")) {
                            void api.openExternal(href).catch((error) => {
                              console.error("cliphistory: failed to open release link", error);
                              setStatus({
                                kind: "error",
                                message: t("settings.updateOpenFailed"),
                              });
                            });
                          }
                        }}
                      >
                        {children}
                      </a>
                    ),
                  }}
                >
                  {updateInfo.notes}
                </Markdown>
              </div>
            </Suspense>
          )}

          <div className="update-actions">
            <button
              type="button"
              className={`btn ${updateInfo ? "btn-ghost" : "btn-primary"}`}
              disabled={checkingUpdates}
              onClick={() => void checkForUpdates()}
            >
              {checkingUpdates
                ? t("settings.checkingUpdates")
                : t("settings.checkUpdates")}
            </button>
            {updateInfo && (
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => void openRelease()}
              >
                {t("settings.updateOpen")}
              </button>
            )}
          </div>
        </section>

        <section className="card">
          <div className="card-heading">
            <KeyboardIcon />
            <h2>{t("settings.globalShortcut")}</h2>
          </div>
          <p className="card-description">
            {t("settings.globalShortcutDescription")}
          </p>

          <HotkeyRecorder value={form.shortcut} onChange={(shortcut) => update({ shortcut })} />

          {!shortcutOk && (
            <div className="callout warning">
              <WarningIcon />
              <div>
                <strong>{t("settings.shortcutInactiveTitle")}</strong>{" "}
                {t("settings.shortcutInactiveText")}
              </div>
            </div>
          )}

          {isLinux && appInfo && !appInfo.wayland && (
            <div className="callout info">
              <div>{t("settings.x11Note")}</div>
            </div>
          )}

          {isLinux && appInfo?.wayland && (
            <div className="callout warning">
              <WarningIcon />
              <div>
                <strong>{t("settings.waylandTitle")}</strong>{" "}
                {t("settings.waylandText")}
                <code className="command-box">
                  "{appInfo.executable ||
                    (language === "pt-BR"
                      ? "/caminho/para/cliphistory"
                      : "/path/to/cliphistory")}" --toggle
                </code>
                <span className="callout-note">{t("settings.waylandHint")}</span>
                {appInfo.gdkBackend === "wayland" && (
                  <span className="callout-note">
                    {t("settings.waylandForcedNative")}
                  </span>
                )}
                {!appInfo.xwaylandAvailable && (
                  <span className="callout-note">
                    {t("settings.waylandNoXWayland")}
                  </span>
                )}
                {appInfo.xwaylandAvailable && appInfo.gdkBackend !== "wayland" && (
                  <span className="callout-note">
                    {t("settings.waylandUsingX11")}
                  </span>
                )}
              </div>
            </div>
          )}
        </section>

        <section className="card">
          <h2>{t("settings.behavior")}</h2>

          <label className="switch-row">
            <span className="row-copy">
              <span className="row-title">{t("settings.startAtLogin")}</span>
              <span className="row-desc">
                {t("settings.startAtLoginDescription")}
              </span>
            </span>
            <input
              type="checkbox"
              className="switch"
              checked={form.startAtLogin}
              onChange={(event) => update({ startAtLogin: event.target.checked })}
            />
          </label>

          <label className="switch-row">
            <span className="row-copy">
              <span className="row-title">{t("settings.autoPaste")}</span>
              <span className="row-desc">
                {t("settings.autoPasteDescription")}
              </span>
            </span>
            <input
              type="checkbox"
              className="switch"
              checked={form.autoPaste}
              onChange={(event) => update({ autoPaste: event.target.checked })}
            />
          </label>

          <label className="switch-row">
            <span className="row-copy">
              <span className="row-title">{t("settings.hideOnBlur")}</span>
              <span className="row-desc">
                {t("settings.hideOnBlurDescription")}
              </span>
            </span>
            <input
              type="checkbox"
              className="switch"
              checked={form.hideOnBlur}
              onChange={(event) => update({ hideOnBlur: event.target.checked })}
            />
          </label>

          {form.autoPaste && (
            <div className={`callout ${isMac ? "warning" : "info"}`}>
              {isMac && <WarningIcon />}
              <div>
                {isMac ? (
                  <>
                    <strong>{t("settings.macAccessibilityTitle")}</strong>{" "}
                    {t("settings.macAccessibilityText")}
                  </>
                ) : (
                  <>{t("settings.autoPasteNote")}</>
                )}
              </div>
            </div>
          )}

          <div className="slider-row">
            <div className="row-copy">
              <span className="row-title">{t("settings.maxItems")}</span>
              <span className="row-desc">
                {t("settings.maxItemsDescription")}
              </span>
            </div>
            <div className="slider-control">
              <input
                type="range"
                min={10}
                max={500}
                step={10}
                value={form.maxItems}
                onChange={(event) => update({ maxItems: Number(event.target.value) })}
                aria-label={t("settings.maxItemsAria")}
              />
              <output>{form.maxItems}</output>
            </div>
          </div>
        </section>

        <section className="card">
          <h2>{t("settings.history")}</h2>

          <div className="stats-row">
            <div className="stat">
              <span className="stat-value">{stats?.total ?? "—"}</span>
              <span className="stat-label">{t("settings.statsTotal")}</span>
            </div>
            <div className="stat">
              <span className="stat-value">{stats?.pinned ?? "—"}</span>
              <span className="stat-label">{t("settings.statsPinned")}</span>
            </div>
            <div className="stat">
              <span className="stat-value">{stats?.images ?? "—"}</span>
              <span className="stat-label">{t("settings.statsImages")}</span>
            </div>
          </div>

          <p className="card-description">{t("settings.pasteHint")}</p>

          <div className="danger-row">
            <div className="row-copy">
              <span className="row-title">{t("settings.clearHistory")}</span>
              <span className="row-desc">
                {t("settings.clearHistoryDescription")}
              </span>
            </div>
            <div className="danger-actions">
              {confirmClear && (
                <button
                  type="button"
                  className="btn btn-quiet"
                  onClick={() => setConfirmClear(false)}
                >
                  {t("common.cancel")}
                </button>
              )}
              <button
                type="button"
                className={`btn ${confirmClear ? "btn-danger-solid" : "btn-danger"}`}
                onClick={() => void clear()}
                disabled={clearing}
              >
                {clearing
                  ? t("settings.clearing")
                  : confirmClear
                    ? t("settings.clearConfirm")
                    : t("settings.clear")}
              </button>
            </div>
          </div>
        </section>

        <section className="card diagnostics-card">
          <details>
            <summary>
              <span className="diagnostics-title">{t("settings.diagnosticsTitle")}</span>
              <span className="row-desc">{t("settings.diagnosticsDescription")}</span>
            </summary>

            <div className="diagnostics-body">
              {diagnostics ? (
                <pre className="diagnostics-output">{diagnostics.summary}</pre>
              ) : (
                <p className="card-description">
                  {diagnosticsLoading
                    ? t("settings.diagnosticsLoading")
                    : t("settings.diagnosticsUnavailable")}
                </p>
              )}

              {diagnostics?.recentLogs && (
                <details className="diagnostics-logs">
                  <summary>{t("settings.diagnosticsRecentLogs")}</summary>
                  <pre className="diagnostics-output">{diagnostics.recentLogs}</pre>
                </details>
              )}

              <div className="diagnostics-actions">
                <button
                  type="button"
                  className="btn btn-ghost"
                  disabled={diagnosticsLoading}
                  onClick={() => void loadDiagnostics()}
                >
                  {t("settings.diagnosticsRefresh")}
                </button>
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={() => void copyDiagnostics()}
                >
                  {t("settings.diagnosticsCopy")}
                </button>
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={() => void openLogs()}
                >
                  {t("settings.diagnosticsOpenLogs")}
                </button>
              </div>
            </div>
          </details>
        </section>
      </div>

      <footer className="settings-footer">
        {status && (
          <div className={`status ${status.kind}`} role="status">
            {status.kind === "success" ? <CheckIcon /> : <WarningIcon />}
            <span>{status.message}</span>
          </div>
        )}

        <div className="footer-spacer" />

        <span className="version">v{version || __APP_VERSION__} · Tauri 2</span>
        <button type="button" className="btn btn-quiet" onClick={() => void api.hideSettings()}>
          {t("common.close")}
        </button>
        <button
          type="button"
          className="btn btn-primary"
          onClick={() => void save()}
          disabled={saving}
        >
          {saving ? t("common.saving") : t("common.saveChanges")}
        </button>
      </footer>
    </div>
  );
}

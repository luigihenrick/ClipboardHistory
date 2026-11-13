import { memo, useCallback, useEffect, useRef, useState } from "react";

import {
  ClipboardIcon,
  CloseIcon,
  Logo,
  PinIcon,
  SearchIcon,
  SettingsIcon,
  TrashIcon,
} from "../components/Icons";
import { useI18n } from "../i18n";
import { isLanguage } from "../i18n/types";
import { previewText, relativeTime } from "../lib/format";
import { onEvent } from "../lib/events";
import * as api from "../lib/ipc";
import { formatAccelerator } from "../lib/shortcut";
import type { Entry, Settings } from "../types";

interface RowProps {
  entry: Entry;
  index: number;
  selected: boolean;
  onHover: (index: number) => void;
  onCopy: (entry: Entry) => void;
  onPin: (entry: Entry) => void;
  onDelete: (entry: Entry) => void;
}

const EntryRow = memo(function EntryRow({
  entry,
  index,
  selected,
  onHover,
  onCopy,
  onPin,
  onDelete,
}: RowProps) {
  const { t, language } = useI18n();

  const pinLabel = entry.pinned ? t("popup.unpin") : t("popup.pin");

  return (
    <div
      className={`entry${selected ? " selected" : ""}`}
      data-index={index}
      role="option"
      aria-selected={selected}
      onMouseEnter={() => onHover(index)}
      onClick={() => onCopy(entry)}
    >
      {entry.kind === "image" && entry.thumbnail && (
        <div className="entry-thumb">
          <img
            src={entry.thumbnail}
            alt=""
            draggable={false}
            loading="lazy"
            decoding="async"
          />
        </div>
      )}

      <div className="entry-body">
        {entry.kind === "image" ? (
          <p className="entry-preview entry-preview-image">{t("popup.image")}</p>
        ) : (
          <p className="entry-preview">{previewText(entry.text ?? "")}</p>
        )}

        <div className="entry-meta">
          <span>{relativeTime(entry.createdAt, language)}</span>
          {entry.kind === "image" && entry.width && entry.height && (
            <>
              <span className="meta-sep">•</span>
              <span>
                {entry.width} × {entry.height}
              </span>
            </>
          )}
          {entry.pinned && (
            <>
              <span className="meta-sep">•</span>
              <span
                className="pinned-icon"
                role="img"
                title={t("popup.pinnedBadge")}
                aria-label={t("popup.pinnedBadge")}
              >
                <PinIcon size={13} filled />
              </span>
            </>
          )}
        </div>
      </div>

      <div className="entry-actions">
        <button
          type="button"
          className={`icon-btn${entry.pinned ? " active" : ""}`}
          title={pinLabel}
          aria-label={pinLabel}
          onClick={(event) => {
            event.stopPropagation();
            onPin(entry);
          }}
        >
          <PinIcon filled={entry.pinned} />
        </button>

        <button
          type="button"
          className="icon-btn danger"
          title={t("popup.remove")}
          aria-label={t("popup.remove")}
          onClick={(event) => {
            event.stopPropagation();
            onDelete(entry);
          }}
        >
          <TrashIcon />
        </button>
      </div>
    </div>
  );
});

export default function Popup() {
  const { t, language, setLanguage: applyLanguage } = useI18n();
  const [entries, setEntries] = useState<Entry[]>([]);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState(0);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);

  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const requestId = useRef(0);
  const searchRef = useRef(search);
  const skipNextSearchRefresh = useRef(false);

  useEffect(() => {
    searchRef.current = search;
  }, [search]);

  const refresh = useCallback(async (term: string) => {
    const id = ++requestId.current;
    try {
      const data = await api.getEntries(term);
      if (id !== requestId.current) return;
      setEntries(data);
    } catch (error) {
      console.error("cliphistory: failed to load history", error);
    } finally {
      if (id === requestId.current) setLoading(false);
    }
  }, []);

  const focusSearch = useCallback(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  useEffect(() => {
    // Debounce search so typing does not trigger an IPC round-trip per keystroke.
    if (skipNextSearchRefresh.current) {
      skipNextSearchRefresh.current = false;
      return;
    }

    const timeout = window.setTimeout(() => {
      void refresh(search);
    }, 120);

    return () => window.clearTimeout(timeout);
  }, [refresh, search]);

  useEffect(() => {
    void api
      .getSettings()
      .then((loaded) => {
        setSettings(loaded);
        applyLanguage(isLanguage(loaded.language) ? loaded.language : "en");
      })
      .catch((error) => console.error("cliphistory: failed to load settings", error));

    let disposed = false;
    let unlisten: (() => void) | undefined;

    void onEvent("popup://shown", () => {
      skipNextSearchRefresh.current = searchRef.current.length > 0;
      setSearch("");
      setSelected(0);
      setLoading(true);
      void refresh("");
      focusSearch();
    }).then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    // The window is created hidden, but focusing the field on mount keeps dev tools honest.
    focusSearch();

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [applyLanguage, focusSearch, refresh]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let disposed = false;

    const subscribe = async () => {
      const history = await onEvent("history://changed", () => {
        void refresh(searchRef.current);
      });
      const settingsListener = await onEvent<Settings>("settings://changed", (payload) => {
        setSettings(payload);
        applyLanguage(isLanguage(payload.language) ? payload.language : "en");
      });

      if (disposed) {
        history();
        settingsListener();
      } else {
        unlisteners.push(history, settingsListener);
      }
    };

    void subscribe();

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [applyLanguage, refresh]);

  useEffect(() => {
    setSelected((current) => {
      if (entries.length === 0) return 0;
      return Math.min(current, entries.length - 1);
    });
  }, [entries.length]);

  useEffect(() => {
    const node = listRef.current?.querySelector<HTMLElement>(`[data-index="${selected}"]`);
    node?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const copy = useCallback(async (entry: Entry | undefined) => {
    if (!entry) return;
    try {
      await api.copyEntry(entry.id);
    } catch (error) {
      console.error("cliphistory: failed to copy entry", error);
    }
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void api.hidePopup();
        return;
      }

      // Let focused action buttons handle their own keyboard activation.
      const target = event.target;
      if (target instanceof HTMLElement && target.closest("button")) return;

      if (event.key === "ArrowDown") {
        event.preventDefault();
        setSelected((current) =>
          entries.length === 0 ? 0 : (current + 1) % entries.length,
        );
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        setSelected((current) =>
          entries.length === 0 ? 0 : (current - 1 + entries.length) % entries.length,
        );
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        void copy(entries[selected]);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [copy, entries, selected]);

  const handlePin = useCallback(
    async (entry: Entry) => {
      try {
        await api.togglePin(entry.id);
        await refresh(searchRef.current);
      } catch (error) {
        console.error("cliphistory: failed to pin entry", error);
      }
    },
    [refresh],
  );

  const handleDelete = useCallback(
    async (entry: Entry) => {
      try {
        await api.deleteEntry(entry.id);
        await refresh(searchRef.current);
      } catch (error) {
        console.error("cliphistory: failed to delete entry", error);
      }
    },
    [refresh],
  );

  const total = entries.length;

  return (
    <div className="popup">
      <header className="popup-header">
        <div className="brand">
          <Logo size={26} />
          <span className="brand-name">ClipHistory</span>
        </div>

        <div className="header-actions">
          <span className="counter">
            {t(total === 1 ? "popup.itemsOne" : "popup.itemsMany", { count: total })}
          </span>
          <button
            type="button"
            className="icon-btn"
            title={t("common.settings")}
            aria-label={t("popup.openSettings")}
            onClick={() => void api.openSettings()}
          >
            <SettingsIcon />
          </button>
          <button
            type="button"
            className="icon-btn"
            title={t("popup.closeHint")}
            aria-label={t("common.close")}
            onClick={() => void api.hidePopup()}
          >
            <CloseIcon />
          </button>
        </div>
      </header>

      <div className="search-field">
        <SearchIcon className="search-icon" />
        <input
          ref={inputRef}
          className="search-input"
          type="text"
          value={search}
          placeholder={t("popup.searchPlaceholder")}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => {
            setSearch(event.target.value);
            setSelected(0);
          }}
        />
        {search && (
          <button
            type="button"
            className="search-clear"
            aria-label={t("popup.clearSearch")}
            onClick={() => {
              setSearch("");
              setSelected(0);
              focusSearch();
            }}
          >
            <CloseIcon size={12} />
          </button>
        )}
      </div>

      <div
        className="entry-list"
        role="listbox"
        aria-label={t("popup.historyLabel")}
        ref={listRef}
      >
        {loading && entries.length === 0 && (
          <div className="loading-state">
            <span className="spinner" />
            {t("popup.loading")}
          </div>
        )}

        {!loading && total === 0 && (
          <div className="empty-state">
            <ClipboardIcon size={38} />
            {search ? (
              <>
                <p className="empty-title">{t("popup.noResultsTitle")}</p>
                <p className="empty-text">{t("popup.noResultsText")}</p>
              </>
            ) : (
              <>
                <p className="empty-title">{t("popup.emptyTitle")}</p>
                <p className="empty-text">{t("popup.emptyText")}</p>
              </>
            )}
          </div>
        )}

        {entries.map((entry, index) => (
          <EntryRow
            key={entry.id}
            entry={entry}
            index={index}
            selected={selected === index}
            onHover={setSelected}
            onCopy={copy}
            onPin={handlePin}
            onDelete={handleDelete}
          />
        ))}
      </div>

      <footer className="popup-footer">
        <span className="footer-hint">
          <kbd>↑</kbd>
          <kbd>↓</kbd> {t("popup.footerNavigate")}
          <span className="footer-sep">·</span>
          <kbd>↵</kbd> {t("popup.footerCopy")}
          <span className="footer-sep">·</span>
          <kbd>esc</kbd> {t("popup.footerClose")}
        </span>
        {settings && (
          <kbd className="shortcut-pill">
            {formatAccelerator(settings.shortcut, language)}
          </kbd>
        )}
      </footer>
    </div>
  );
}

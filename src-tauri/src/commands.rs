use std::str::FromStr;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

use crate::db::CopyPayload;
use crate::models::{normalize_language, Entry, Settings, Stats};
use crate::{popup, settings as settings_store, tray, AppState};

fn rebuild_tray(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || tray::refresh(&handle));
}

#[tauri::command]
pub async fn get_entries(
    state: State<'_, AppState>,
    search: Option<String>,
) -> Result<Vec<Entry>, String> {
    state.db.list(search.as_deref())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub platform: String,
    pub session_type: Option<String>,
    /// True on Wayland sessions, where global key grabs are not available.
    pub wayland: bool,
    /// Whether an X11 display (XWayland) can actually be reached.
    pub xwayland_available: bool,
    /// Explicit GDK backend override, when set.
    pub gdk_backend: Option<String>,
    /// Absolute path to the running executable (useful for desktop-environment shortcuts).
    pub executable: String,
}

#[tauri::command]
pub async fn get_app_info() -> Result<AppInfo, String> {
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let wayland =
        session_type.as_deref() == Some("wayland") || std::env::var_os("WAYLAND_DISPLAY").is_some();

    let executable = std::env::var("APPIMAGE")
        .ok()
        .filter(|path| !path.is_empty())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .map(|path| path.display().to_string())
        })
        .unwrap_or_default();

    Ok(AppInfo {
        platform: std::env::consts::OS.to_string(),
        session_type,
        wayland,
        xwayland_available: crate::xwayland_available(),
        gdk_backend: std::env::var("GDK_BACKEND").ok(),
        executable,
    })
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "Settings unavailable.".to_string())
}

#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<Stats, String> {
    state.db.stats()
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let settings = settings.sanitize();

    let parsed = Shortcut::from_str(&settings.shortcut)
        .map_err(|_| "Invalid shortcut. Use combinations such as Ctrl+Shift+V.".to_string())?;

    if !parsed
        .mods
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META)
    {
        return Err("The shortcut must include Ctrl, Alt, or ⌘/Super.".to_string());
    }

    let previous = state
        .settings
        .lock()
        .map(|current| current.clone())
        .map_err(|_| "Settings unavailable.".to_string())?;

    let shortcut_changed = settings.shortcut != previous.shortcut;
    let needs_register = shortcut_changed
        || !app
            .global_shortcut()
            .is_registered(settings.shortcut.as_str());

    // Register the new shortcut before dropping the old one, so a rejected combination never
    // leaves the user without a working shortcut.
    if needs_register {
        app.global_shortcut()
            .register(settings.shortcut.as_str())
            .map_err(|error| format!("Could not register \"{}\": {error}", settings.shortcut))?;
    }

    // Apply autostart before persisting: a failure here should abort the whole save.
    if settings.start_at_login != previous.start_at_login {
        let autolaunch = app.autolaunch();
        let result = if settings.start_at_login {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };

        if let Err(error) = result {
            if needs_register {
                let _ = app.global_shortcut().unregister(settings.shortcut.as_str());
            }
            return Err(format!("Failed to configure launch at login: {error}"));
        }
    }

    if let Err(error) = settings_store::save(&app, &settings) {
        // Undo the OS-level side effects so memory, disk and the system stay consistent.
        if needs_register {
            let _ = app.global_shortcut().unregister(settings.shortcut.as_str());
            let _ = app.global_shortcut().register(previous.shortcut.as_str());
        }
        if settings.start_at_login != previous.start_at_login {
            let autolaunch = app.autolaunch();
            let _ = if previous.start_at_login {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
        }
        return Err(error);
    }

    // The new shortcut is live; the previous one can go away now.
    if shortcut_changed {
        let _ = app.global_shortcut().unregister(previous.shortcut.as_str());
    }

    {
        let mut current = state
            .settings
            .lock()
            .map_err(|_| "Settings unavailable.".to_string())?;
        *current = settings.clone();
    }

    if settings.language != previous.language {
        rebuild_tray(&app);
    }

    if settings.max_items != previous.max_items {
        if let Err(error) = state.db.prune_now(settings.max_items) {
            eprintln!("cliphistory: failed to apply the history limit: {error}");
        }
    }

    let _ = app.emit("settings://changed", settings.clone());
    let _ = app.emit("history://changed", ());

    Ok(settings)
}

#[tauri::command]
pub async fn set_language(
    app: AppHandle,
    state: State<'_, AppState>,
    language: String,
) -> Result<Settings, String> {
    let language = normalize_language(&language);

    let current = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "Settings unavailable.".to_string())?;

    if current.language == language {
        return Ok(current);
    }

    let mut updated = current;
    updated.language = language;

    settings_store::save(&app, &updated)?;

    {
        let mut current = state
            .settings
            .lock()
            .map_err(|_| "Settings unavailable.".to_string())?;
        *current = updated.clone();
    }

    rebuild_tray(&app);
    let _ = app.emit("settings://changed", updated.clone());

    Ok(updated)
}

#[tauri::command]
pub async fn shortcut_status(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let shortcut = state
        .settings
        .lock()
        .map(|settings| settings.shortcut.clone())
        .map_err(|_| "Settings unavailable.".to_string())?;

    Ok(app.global_shortcut().is_registered(shortcut.as_str()))
}

#[tauri::command]
pub async fn copy_entry(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let payload = state
        .db
        .payload(id)?
        .ok_or_else(|| "Item not found.".to_string())?;

    match payload {
        CopyPayload::Text(text) => {
            app.clipboard()
                .write_text(text)
                .map_err(|error| error.to_string())?;
        }
        CopyPayload::Image(path) => {
            let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
            let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                .map_err(|error| error.to_string())?
                .into_rgba8();
            let (width, height) = (decoded.width(), decoded.height());
            let clipboard_image = tauri::image::Image::new_owned(decoded.into_raw(), width, height);

            app.clipboard()
                .write_image(&clipboard_image)
                .map_err(|error| error.to_string())?;
        }
    }

    state.db.touch(id).ok();
    popup::hide(&app);

    let auto_paste = state
        .settings
        .lock()
        .map(|settings| settings.auto_paste)
        .unwrap_or(false);

    if auto_paste {
        // On macOS, hiding the whole app returns focus to the application that was active
        // before the popup appeared. On other platforms hiding the window is enough.
        #[cfg(target_os = "macos")]
        {
            let _ = app.hide();
        }

        let paste_app = app.clone();
        let _ = std::thread::spawn(move || crate::paste::paste_clipboard(paste_app));
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_pin(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<bool, String> {
    let pinned = state.db.toggle_pin(id)?;
    let _ = app.emit("history://changed", ());
    Ok(pinned)
}

#[tauri::command]
pub async fn delete_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    state.db.delete(id)?;
    let _ = app.emit("history://changed", ());
    Ok(())
}

#[tauri::command]
pub async fn clear_history(app: AppHandle, state: State<'_, AppState>) -> Result<usize, String> {
    let removed = state.db.clear_unpinned()?;
    let _ = app.emit("history://changed", ());
    Ok(removed)
}

#[tauri::command]
pub async fn hide_popup(app: AppHandle) -> Result<(), String> {
    popup::hide(&app);
    Ok(())
}

#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    popup::open_settings(&app);
    Ok(())
}

#[tauri::command]
pub async fn hide_settings(app: AppHandle) -> Result<(), String> {
    popup::hide_settings(&app);
    Ok(())
}

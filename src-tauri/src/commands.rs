#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;
use std::str::FromStr;
#[cfg(target_os = "linux")]
use std::time::Duration;

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
pub fn get_update(state: State<'_, AppState>) -> Option<crate::updater::UpdateInfo> {
    state
        .available_update
        .lock()
        .ok()
        .and_then(|update| update.clone())
}

#[tauri::command]
pub fn get_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> crate::diagnostics::Diagnostics {
    crate::diagnostics::collect(&app, state.inner())
}

#[tauri::command]
pub fn copy_diagnostics(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let diagnostics = crate::diagnostics::collect(&app, state.inner());
    let text = format!(
        "{}\n\nRecent logs:\n{}",
        diagnostics.summary, diagnostics.recent_logs
    );

    app.clipboard()
        .write_text(text)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_logs_folder() -> Result<(), String> {
    let dir = crate::logging::dir().ok_or_else(|| "Logs unavailable.".to_string())?;
    open::that(dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn report_search_focus(state: State<'_, AppState>, elapsed_ms: u64) {
    if let Ok(mut timings) = state.popup_timings.lock() {
        timings.search_focused_ms = Some(elapsed_ms);
    }
}

fn is_allowed_external_url(url: &str) -> bool {
    url.starts_with("https://")
        && !url
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    if !is_allowed_external_url(&url) {
        return Err("Only HTTPS links can be opened.".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        open_external_url_linux(&url).map_err(|error| {
            crate::logging::error(&error);
            error
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        open::that(url).map_err(|error| error.to_string())
    }
}

/// Opens a URL with the host desktop opener.
///
/// AppImages export bundled `LD_LIBRARY_PATH`/GTK/GIO variables. Passing those to `xdg-open`
/// can make it load AppImage libraries instead of the host's and fail silently. Try the common
/// host launchers with a clean environment so release links also work from the AppImage.
#[cfg(target_os = "linux")]
fn open_external_url_linux(url: &str) -> Result<(), String> {
    const CANDIDATES: [(&str, &[&str]); 6] = [
        ("xdg-open", &[]),
        ("gio", &["open"]),
        ("kde-open6", &[]),
        ("kde-open5", &[]),
        ("gnome-open", &[]),
        ("kde-open", &[]),
    ];

    let mut errors = Vec::new();
    for (program, args) in CANDIDATES {
        let command_path = ["/usr/bin", "/usr/local/bin", "/bin"]
            .iter()
            .map(|directory| PathBuf::from(directory).join(program))
            .find(|path| path.is_file())
            .unwrap_or_else(|| PathBuf::from(program));

        let mut command = Command::new(&command_path);
        command
            .args(args.iter().copied())
            .arg(url)
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .env("XDG_DATA_DIRS", "/usr/local/share:/usr/share")
            .env_remove("LD_LIBRARY_PATH")
            .env_remove("LD_PRELOAD")
            .env_remove("APPDIR")
            .env_remove("APPIMAGE")
            .env_remove("ARGV0")
            .env_remove("OWD")
            .env_remove("GDK_BACKEND")
            .env_remove("GDK_PIXBUF_MODULE_FILE")
            .env_remove("GDK_PIXBUF_MODULEDIR")
            .env_remove("GIO_EXTRA_MODULES")
            .env_remove("GIO_MODULE_DIR")
            .env_remove("GI_TYPELIB_PATH")
            .env_remove("GSETTINGS_SCHEMA_DIR")
            .env_remove("GTK_DATA_PREFIX")
            .env_remove("GTK_IM_MODULE_FILE")
            .env_remove("GTK_THEME")
            .env_remove("GTK_PATH")
            .env_remove("GTK_MODULES")
            .env_remove("WEBKIT_DISABLE_COMPOSITING_MODE");

        match command.spawn() {
            Ok(mut child) => {
                // Give fast failures (missing/incompatible host launcher) a chance to fall
                // through, while not waiting for browsers that stay attached to xdg-open.
                std::thread::sleep(Duration::from_millis(120));
                match child.try_wait() {
                    Ok(Some(status)) if !status.success() => {
                        errors.push(format!("{program} exited with {status}"));
                    }
                    _ => {
                        crate::logging::info(&format!("opened external URL with {program}"));
                        return Ok(());
                    }
                }
            }
            Err(error) => errors.push(format!("{}: {error}", command_path.display())),
        }
    }

    Err(format!(
        "Could not open the release page: {}",
        errors.join("; ")
    ))
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

#[cfg(test)]
mod tests {
    use super::is_allowed_external_url;

    #[test]
    fn allows_https_urls() {
        assert!(is_allowed_external_url(
            "https://github.com/luigihenrick/ClipboardHistory/releases/tag/v1.2.0"
        ));
    }

    #[test]
    fn rejects_non_https_or_unsafe_urls() {
        assert!(!is_allowed_external_url("http://example.com"));
        assert!(!is_allowed_external_url("file:///etc/passwd"));
        assert!(!is_allowed_external_url("javascript:alert(1)"));
        assert!(!is_allowed_external_url("https://example.com/\n--evil"));
        assert!(!is_allowed_external_url(""));
    }
}

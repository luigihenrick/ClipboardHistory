mod clipboard_watcher;
mod commands;
mod db;
mod models;
mod paste;
mod popup;
mod settings;
mod tray;
mod updater;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct AppState {
    pub db: db::Db,
    pub settings: Mutex<models::Settings>,
    /// Used to ignore spurious focus-loss events right after the popup is shown.
    pub popup_shown_at: Mutex<Option<Instant>>,
    /// Set once the popup has been focused at least once since it was shown.
    pub popup_was_focused: AtomicBool,
    /// Window that had focus before the popup opened, used to restore focus for auto-paste.
    #[cfg(target_os = "linux")]
    pub previous_x11_window: Mutex<Option<u32>>,
    /// Latest release found by the startup update check.
    pub available_update: Mutex<Option<updater::UpdateInfo>>,
}

/// Returns true when the argument requests showing the popup at startup.
fn is_toggle_arg(arg: &str) -> bool {
    arg == "--toggle"
}

/// Returns true when the process was started with `--toggle`.
///
/// Pure helper so it can be unit tested without a Tauri runtime.
fn opens_popup_on_start<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| is_toggle_arg(arg.as_ref()))
}

/// Returns true when an X11 display can actually be reached (XWayland on Wayland sessions).
#[cfg(target_os = "linux")]
pub(crate) fn xwayland_available() -> bool {
    std::env::var_os("DISPLAY").is_some() && x11rb::connect(None).is_ok()
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn xwayland_available() -> bool {
    false
}

/// Returns true when the app should force the X11 backend on a Wayland session.
///
/// Native Wayland restricts top-level window placement and global shortcuts. When XWayland is
/// available we run through it by default; an explicit `GDK_BACKEND` (e.g. `wayland`) always
/// wins.
#[cfg(any(target_os = "linux", test))]
fn prefer_x11_backend(
    session_type: Option<&str>,
    has_wayland_display: bool,
    xwayland_available: bool,
    gdk_backend: Option<&str>,
) -> bool {
    let on_wayland = session_type == Some("wayland") || has_wayland_display;

    on_wayland && xwayland_available && gdk_backend.is_none()
}

#[cfg(target_os = "linux")]
fn configure_linux_backend() {
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let gdk_backend = std::env::var("GDK_BACKEND").ok();
    let on_wayland =
        session_type.as_deref() == Some("wayland") || std::env::var_os("WAYLAND_DISPLAY").is_some();
    let xwayland = xwayland_available();

    let use_x11 = prefer_x11_backend(
        session_type.as_deref(),
        on_wayland,
        xwayland,
        gdk_backend.as_deref(),
    );

    if use_x11 {
        std::env::set_var("GDK_BACKEND", "x11");

        #[cfg(debug_assertions)]
        eprintln!("cliphistory: Wayland detected; using the XWayland compatibility backend");
    } else if on_wayland {
        #[cfg(debug_assertions)]
        {
            if gdk_backend.as_deref() == Some("wayland") {
                eprintln!("cliphistory: GDK_BACKEND=wayland is set; running native Wayland");
            } else if !xwayland {
                eprintln!("cliphistory: XWayland is not available; running native Wayland");
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    configure_linux_backend();

    // `--toggle` is used by desktop-environment shortcuts on Wayland, where global key grabs
    // are not available. It also works when the app is not running yet.
    let start_with_popup = opens_popup_on_start(std::env::args());

    tauri::Builder::default()
        // Single-instance must be the first plugin registered.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|arg| is_toggle_arg(arg)) {
                popup::toggle(app);
            } else {
                popup::show(app);
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        #[cfg(debug_assertions)]
                        eprintln!("cliphistory: global shortcut pressed: {shortcut}");
                        popup::toggle(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let handle = app.handle().clone();
            let (loaded, first_run) = settings::load(&handle);

            #[cfg(debug_assertions)]
            eprintln!(
                "cliphistory: initialized (session={:?}, gdk_backend={:?}, display={:?}, xwayland={})",
                std::env::var("XDG_SESSION_TYPE").ok(),
                std::env::var("GDK_BACKEND").ok(),
                std::env::var("DISPLAY").ok(),
                crate::xwayland_available()
            );

            let database = db::Db::open(&handle)
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

            let _ = app.manage(AppState {
                db: database,
                settings: Mutex::new(loaded.clone()),
                popup_shown_at: Mutex::new(None),
                popup_was_focused: AtomicBool::new(false),
                #[cfg(target_os = "linux")]
                previous_x11_window: Mutex::new(None),
                available_update: Mutex::new(None),
            });

            // Configure the popup before its first map so GTK/WM focus-on-map is active even
            // before the first `present()` call from `popup::show`.
            #[cfg(target_os = "linux")]
            if let Some(window) = handle.get_webview_window("popup") {
                if let Ok(gtk_window) = window.gtk_window() {
                    use gtk::prelude::GtkWindowExt;

                    gtk_window.set_accept_focus(true);
                    gtk_window.set_focus_on_map(true);
                }
            }

            tray::create(&handle)?;

            match handle
                .global_shortcut()
                .register(loaded.shortcut.as_str())
            {
                Ok(()) => eprintln!(
                    "cliphistory: global shortcut registered: {}",
                    loaded.shortcut
                ),
                Err(error) => eprintln!(
                    "cliphistory: could not register the global shortcut \"{}\": {error}",
                    loaded.shortcut
                ),
            }

            // Keep the OS autostart entry in sync with the stored preference (e.g. after the
            // app was moved to a different path or the settings file was copied around).
            {
                use tauri_plugin_autostart::ManagerExt;
                match handle.autolaunch().is_enabled() {
                    Ok(enabled) if enabled != loaded.start_at_login => {
                        let result = if loaded.start_at_login {
                            handle.autolaunch().enable()
                        } else {
                            handle.autolaunch().disable()
                        };
                        if let Err(error) = result {
                            eprintln!("cliphistory: failed to sync launch at login: {error}");
                        }
                    }
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("cliphistory: failed to check launch at login: {error}")
                    }
                }
            }

            clipboard_watcher::spawn(handle.clone());

            // Check GitHub for a newer release without delaying startup. The result is stored
            // in AppState and emitted to the Settings window when it arrives.
            updater::spawn_check(handle.clone());

            // Existing image entries may predate previews or carry the old, larger thumbnails.
            // Regenerate them in the background so the popup list can show small previews
            // without delaying startup or the first open.
            {
                let backfill = handle.clone();
                std::thread::spawn(move || {
                    let Some(state) = backfill.try_state::<AppState>() else {
                        return;
                    };

                    match state.db.backfill_thumbnails() {
                        Ok(updated) if updated > 0 => {
                            #[cfg(debug_assertions)]
                            eprintln!("cliphistory: generated image previews: {updated}");

                            let _ = backfill.emit("history://changed", ());
                        }
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!("cliphistory: failed to generate image previews: {error}")
                        }
                    }
                });
            }

            if first_run {
                // Persist defaults so the settings window only opens on the very first launch.
                if let Err(error) = settings::save(&handle, &loaded) {
                    eprintln!("cliphistory: failed to save initial settings: {error}");
                }
            }

            if start_with_popup {
                popup::show(&handle);
            } else if first_run {
                popup::open_settings(&handle);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // The settings window and the popup only hide; the app lives in the tray.
                if window.label() == "settings" || window.label() == "popup" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
            WindowEvent::Focused(false) => {
                if window.label() != "popup" {
                    return;
                }

                let Some(state) = window.app_handle().try_state::<AppState>() else {
                    return;
                };

                let hide_on_blur = state
                    .settings
                    .lock()
                    .map(|settings| settings.hide_on_blur)
                    .unwrap_or(false);

                let shown_at = state
                    .popup_shown_at
                    .lock()
                    .ok()
                    .and_then(|shown_at| *shown_at);

                let was_focused = state.popup_was_focused.load(Ordering::SeqCst);

                if popup::should_hide_on_blur(hide_on_blur, was_focused, shown_at) {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_entries,
            commands::get_app_info,
            commands::get_update,
            commands::open_external_url,
            commands::get_settings,
            commands::get_stats,
            commands::save_settings,
            commands::set_language,
            commands::shortcut_status,
            commands::copy_entry,
            commands::toggle_pin,
            commands::delete_entry,
            commands::clear_history,
            commands::hide_popup,
            commands::open_settings,
            commands::hide_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start ClipHistory");
}

#[cfg(test)]
mod tests {
    use super::{is_toggle_arg, opens_popup_on_start, prefer_x11_backend};

    #[test]
    fn detects_the_toggle_argument() {
        assert!(is_toggle_arg("--toggle"));
        assert!(!is_toggle_arg("--show"));
        assert!(!is_toggle_arg("cliphistory"));
    }

    #[test]
    fn detects_toggle_flag() {
        assert!(opens_popup_on_start(["cliphistory", "--toggle"]));
        assert!(opens_popup_on_start(["cliphistory", "--other", "--toggle"]));
    }

    #[test]
    fn ignores_other_arguments() {
        assert!(!opens_popup_on_start(["cliphistory"]));
        assert!(!opens_popup_on_start(["cliphistory", "--show"]));
    }

    #[test]
    fn popup_window_accepts_focus_and_starts_hidden() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();

        let popup = config["app"]["windows"]
            .as_array()
            .expect("windows array")
            .iter()
            .find(|window| window["label"].as_str() == Some("popup"))
            .expect("popup window");

        assert_eq!(popup["focus"].as_bool(), Some(true));
        assert_eq!(popup["visible"].as_bool(), Some(false));
        assert_eq!(popup["center"].as_bool(), Some(true));
    }

    #[test]
    fn release_config_bundles_every_platform() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();

        assert_eq!(config["bundle"]["targets"].as_str(), Some("all"));
        assert_eq!(
            config["bundle"]["macOS"]["signingIdentity"].as_str(),
            Some("-")
        );
    }

    #[test]
    fn package_and_tauri_versions_match() {
        let tauri: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let package: serde_json::Value =
            serde_json::from_str(include_str!("../../package.json")).unwrap();

        assert_eq!(package["version"], tauri["version"]);
    }

    #[test]
    fn prefers_x11_on_wayland_when_xwayland_is_available() {
        assert!(prefer_x11_backend(Some("wayland"), true, true, None));
        assert!(prefer_x11_backend(None, true, true, None));
    }

    #[test]
    fn respects_an_explicit_gdk_backend() {
        assert!(!prefer_x11_backend(
            Some("wayland"),
            true,
            true,
            Some("wayland")
        ));
        assert!(!prefer_x11_backend(
            Some("wayland"),
            true,
            true,
            Some("x11")
        ));
    }

    #[test]
    fn stays_native_without_xwayland() {
        assert!(!prefer_x11_backend(Some("wayland"), true, false, None));
    }

    #[test]
    fn does_not_switch_on_x11_sessions() {
        assert!(!prefer_x11_backend(Some("x11"), false, true, None));
    }
}

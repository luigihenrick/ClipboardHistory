use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use crate::popup;
use crate::AppState;

fn labels(app: &AppHandle) -> (&'static str, &'static str, &'static str, &'static str) {
    let language = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.language.clone())
        .unwrap_or_else(|_| "en".to_string());

    if language == "pt-BR" {
        (
            "Abrir histórico",
            "Configurações…",
            "Sair",
            "ClipHistory — histórico da área de transferência",
        )
    } else {
        (
            "Show history",
            "Settings…",
            "Quit",
            "ClipHistory — clipboard history",
        )
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let (show_label, settings_label, quit_label, tooltip) = labels(app);

    let show = MenuItem::with_id(app, "show-history", show_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "open-settings", settings_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip(tooltip)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show-history" => popup::show(app),
            "open-settings" => popup::open_settings(app),
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

/// Recreates the tray icon so its menu reflects the current language.
pub fn refresh(app: &AppHandle) {
    let _ = app.remove_tray_by_id("main-tray");
    if let Err(error) = create(app) {
        eprintln!("cliphistory: failed to rebuild the tray icon: {error}");
    }
}

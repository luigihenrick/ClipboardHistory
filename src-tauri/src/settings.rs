use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::Settings;

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

/// Loads settings from disk.
///
/// Returns `(settings, first_run)` where `first_run` is true when no settings file exists yet.
pub fn load(app: &AppHandle) -> (Settings, bool) {
    let Ok(path) = settings_path(app) else {
        return (Settings::default(), true);
    };

    if !path.exists() {
        return (Settings::default(), true);
    }

    match fs::read_to_string(&path)
        .ok()
        .and_then(|contents| serde_json::from_str::<Settings>(&contents).ok())
    {
        Some(settings) => (settings.sanitize(), false),
        // A corrupt file should not prevent the app from starting.
        None => (Settings::default(), false),
    }
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

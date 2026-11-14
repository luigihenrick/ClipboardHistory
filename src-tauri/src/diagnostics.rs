use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::models::Stats;
use crate::updater::UpdateInfo;
use crate::{logging, AppState};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupTimings {
    pub shown_at_ms: Option<u64>,
    pub window_shown_ms: Option<u64>,
    pub window_focused_ms: Option<u64>,
    pub search_focused_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDiagnostic {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub work_x: i32,
    pub work_y: i32,
    pub work_width: u32,
    pub work_height: u32,
    pub scale_factor: f64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub app_version: String,
    pub tauri_version: String,
    pub os: String,
    pub arch: String,
    pub session_type: Option<String>,
    pub desktop: Option<String>,
    pub gdk_backend: Option<String>,
    pub wayland: bool,
    pub xwayland_available: bool,
    pub executable: String,
    pub data_dir: String,
    pub database_path: String,
    pub database_size: u64,
    pub log_dir: String,
    pub monitors: Vec<MonitorDiagnostic>,
    pub stats: Stats,
    pub update: Option<UpdateInfo>,
    pub timings: PopupTimings,
    pub generated_at_ms: u64,
    pub recent_logs: String,
    pub summary: String,
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

pub fn collect(app: &AppHandle, state: &AppState) -> Diagnostics {
    let app_version = app.package_info().version.to_string();
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let gdk_backend = std::env::var("GDK_BACKEND").ok();
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

    let data_dir = app
        .path()
        .app_data_dir()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let database_path = app
        .path()
        .app_data_dir()
        .ok()
        .map(|path| path.join("cliphistory.db"))
        .unwrap_or_default();
    let database_size = fs::metadata(&database_path)
        .map(|metadata| metadata.len())
        .unwrap_or_default();
    let log_dir = logging::dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    let mut monitors = Vec::new();
    if let Some(window) = app.get_webview_window("popup") {
        let primary = window.primary_monitor().ok().flatten();
        if let Ok(available) = window.available_monitors() {
            for (index, monitor) in available.iter().enumerate() {
                let name = monitor
                    .name()
                    .cloned()
                    .unwrap_or_else(|| format!("monitor-{index}"));
                let work = monitor.work_area();
                let is_primary = primary
                    .as_ref()
                    .map(|primary| {
                        primary.name() == monitor.name()
                            && primary.position() == monitor.position()
                            && primary.size() == monitor.size()
                    })
                    .unwrap_or(false);

                monitors.push(MonitorDiagnostic {
                    name,
                    x: monitor.position().x,
                    y: monitor.position().y,
                    width: monitor.size().width,
                    height: monitor.size().height,
                    work_x: work.position.x,
                    work_y: work.position.y,
                    work_width: work.size.width,
                    work_height: work.size.height,
                    scale_factor: monitor.scale_factor(),
                    primary: is_primary,
                });
            }
        }
    }

    let stats = state.db.stats().unwrap_or(Stats {
        total: 0,
        pinned: 0,
        images: 0,
    });
    let update = state
        .available_update
        .lock()
        .ok()
        .and_then(|update| update.clone());
    let timings = state
        .popup_timings
        .lock()
        .map(|timings| timings.clone())
        .unwrap_or_default();

    let mut summary = format!(
        "ClipHistory {app_version}\nTauri {}\nOS: {} {}\nSession: {:?}\nDesktop: {:?}\nGDK_BACKEND: {:?}\nWayland: {}\nXWayland available: {}\nExecutable: {executable}\nData dir: {data_dir}\nDatabase: {} ({})\nLogs: {log_dir}\nHistory: {} total, {} pinned, {} images\n",
        tauri::VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH,
        session_type,
        desktop,
        gdk_backend,
        wayland,
        crate::xwayland_available(),
        database_path.display(),
        format_bytes(database_size),
        stats.total,
        stats.pinned,
        stats.images,
    );

    if monitors.is_empty() {
        summary.push_str("Monitors: none reported\n");
    } else {
        summary.push_str("Monitors:\n");
        for monitor in &monitors {
            summary.push_str(&format!(
                "- {}{} {}x{} at {},{} scale {}; work {}x{} at {},{}\n",
                monitor.name,
                if monitor.primary { " (primary)" } else { "" },
                monitor.width,
                monitor.height,
                monitor.x,
                monitor.y,
                monitor.scale_factor,
                monitor.work_width,
                monitor.work_height,
                monitor.work_x,
                monitor.work_y,
            ));
        }
    }

    if let Some(update) = &update {
        summary.push_str(&format!("Update available: {}\n", update.version));
    } else {
        summary.push_str("Update available: none\n");
    }
    summary.push_str(&timings_summary(&timings));

    Diagnostics {
        app_version,
        tauri_version: tauri::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        session_type,
        desktop,
        gdk_backend,
        wayland,
        xwayland_available: crate::xwayland_available(),
        executable,
        data_dir,
        database_path: database_path.display().to_string(),
        database_size,
        log_dir,
        monitors,
        stats,
        update,
        timings,
        generated_at_ms: now_ms(),
        recent_logs: logging::tail(250),
        summary,
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;

    if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn timings_summary(timings: &PopupTimings) -> String {
    let value = |value: Option<u64>| {
        value
            .map(|milliseconds| format!("{milliseconds} ms"))
            .unwrap_or_else(|| "n/a".to_string())
    };

    format!(
        "Last popup: window shown {}, window focused {}, search focused {}\n",
        value(timings.window_shown_ms),
        value(timings.window_focused_ms),
        value(timings.search_focused_ms),
    )
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, timings_summary, PopupTimings};

    #[test]
    fn formats_database_sizes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(2048), "2.0 KiB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2.0 MiB");
    }

    #[test]
    fn summarizes_popup_timings() {
        let summary = timings_summary(&PopupTimings {
            shown_at_ms: Some(1),
            window_shown_ms: Some(30),
            window_focused_ms: Some(40),
            search_focused_ms: Some(55),
        });

        assert!(summary.contains("30 ms"));
        assert!(summary.contains("40 ms"));
        assert!(summary.contains("55 ms"));
    }

    #[test]
    fn shows_missing_timings_as_not_available() {
        assert!(timings_summary(&PopupTimings::default()).contains("n/a"));
    }
}

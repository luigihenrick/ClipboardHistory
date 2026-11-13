use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/luigihenrick/ClipboardHistory/releases/latest";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Release tag, for example `v1.2.1`.
    pub version: String,
    pub name: String,
    pub notes: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
}

fn parse_version(raw: &str) -> Option<semver::Version> {
    semver::Version::parse(raw.trim().trim_start_matches('v')).ok()
}

/// Returns true when `remote` is a valid version newer than `current`.
pub(crate) fn is_newer(remote: &str, current: &str) -> bool {
    match (parse_version(remote), parse_version(current)) {
        (Some(remote), Some(current)) => remote > current,
        _ => false,
    }
}

fn fetch_latest_release(current: &str) -> Option<GitHubRelease> {
    let user_agent = format!("ClipHistory/{current}");

    let output = std::process::Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--max-time",
            "5",
            "--user-agent",
            user_agent.as_str(),
            "--header",
            "Accept: application/vnd.github+json",
            LATEST_RELEASE_API,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

/// Checks GitHub for a release newer than the running version. Network failures are silent:
/// the update card is simply not shown.
pub fn check_for_update(current: &str) -> Option<UpdateInfo> {
    let release = fetch_latest_release(current)?;

    if !is_newer(&release.tag_name, current) {
        return None;
    }

    let version = release.tag_name;
    let name = release.name.unwrap_or_else(|| version.clone());

    Some(UpdateInfo {
        version,
        name,
        notes: release.body.unwrap_or_default(),
        url: release.html_url,
    })
}

/// Runs the update check in the background at startup and notifies the frontend when done.
pub fn spawn_check(app: AppHandle) {
    let current = app.package_info().version.to_string();

    std::thread::spawn(move || {
        let Some(update) = check_for_update(&current) else {
            return;
        };

        if let Some(state) = app.try_state::<crate::AppState>() {
            if let Ok(mut slot) = state.available_update.lock() {
                *slot = Some(update.clone());
            }
        }

        let _ = app.emit("update://available", update);
    });
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn detects_newer_versions() {
        assert!(is_newer("v1.2.0", "1.1.1"));
        assert!(is_newer("1.1.2", "1.1.1"));
        assert!(is_newer("v2.0.0", "1.99.99"));
    }

    #[test]
    fn ignores_same_or_older_versions() {
        assert!(!is_newer("v1.1.1", "1.1.1"));
        assert!(!is_newer("v1.1.0", "1.1.1"));
        assert!(!is_newer("v0.9.0", "1.0.0"));
    }

    #[test]
    fn handles_invalid_versions() {
        assert!(!is_newer("nightly", "1.1.1"));
        assert!(!is_newer("v1.2.0", "dev"));
        assert!(!is_newer("", ""));
    }
}

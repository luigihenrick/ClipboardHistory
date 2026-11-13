use std::thread;
use std::time::Duration;

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use tauri::AppHandle;

/// How long to wait for the window manager to give focus back to the previous application
/// after the popup is hidden.
const FOCUS_DELAY: Duration = Duration::from_millis(250);

/// Optional native-Wayland fallback. XTest (used by enigo) may not reach native Wayland
/// windows; ydotool injects input through uinput when it is installed and its daemon is running.
#[cfg(target_os = "linux")]
fn try_ydotool() -> bool {
    let on_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|value| value == "wayland")
        .unwrap_or(false)
        || std::env::var_os("WAYLAND_DISPLAY").is_some();

    if !on_wayland {
        return false;
    }

    let Ok(version_output) = std::process::Command::new("ydotool")
        .arg("--version")
        .output()
    else {
        return false;
    };

    let mut version_text = String::from_utf8_lossy(&version_output.stdout).into_owned();
    version_text.push(' ');
    version_text.push_str(&String::from_utf8_lossy(&version_output.stderr));
    let major = parse_ydotool_major(&version_text);

    std::process::Command::new("ydotool")
        .args(ydotool_args(major))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// ydotool 0.1.x (shipped by Ubuntu 22.04/24.04) expects symbolic names; ydotool 1.x
/// expects Linux input keycodes. Using keycodes with 0.1.x types their first characters
/// (for example, "2442") instead of pressing Ctrl+V.
#[cfg(target_os = "linux")]
fn ydotool_args(major: Option<u32>) -> &'static [&'static str] {
    if major.unwrap_or(0) >= 1 {
        &["key", "29:1", "47:1", "47:0", "29:0"]
    } else {
        &["key", "ctrl+v"]
    }
}

#[cfg(target_os = "linux")]
fn parse_ydotool_major(output: &str) -> Option<u32> {
    output.split_whitespace().find_map(|token| {
        let token =
            token.trim_matches(|character: char| !character.is_ascii_digit() && character != '.');
        let mut parts = token.split('.');
        let major = parts.next()?.parse::<u32>().ok()?;
        let minor = parts.next()?;

        if !minor.is_empty() && minor.chars().all(|character| character.is_ascii_digit()) {
            Some(major)
        } else {
            None
        }
    })
}

#[cfg(target_os = "macos")]
fn modifier() -> Key {
    Key::Meta
}

#[cfg(not(target_os = "macos"))]
fn modifier() -> Key {
    Key::Control
}

fn send_paste(enigo: &mut Enigo) -> enigo::InputResult<()> {
    let modifier = modifier();
    enigo.key(modifier, Press)?;
    enigo.key(Key::Unicode('v'), Click)?;
    enigo.key(modifier, Release)
}

/// Simulates Ctrl+V (`⌘V` on macOS) in whatever window currently has focus.
///
/// The caller must hide the popup first. This function blocks its own thread while waiting for
/// focus to return, so it is always spawned from the command handler.
pub fn paste_clipboard(app: AppHandle) {
    // On X11/XWayland, explicitly return focus to the window that was active before the popup
    // opened. Hiding the popup is not enough on every window manager.
    crate::popup::restore_previous_window(&app);

    thread::sleep(FOCUS_DELAY);

    #[cfg(target_os = "linux")]
    if try_ydotool() {
        return;
    }

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(error) => {
            eprintln!("cliphistory: could not simulate paste: {error}");
            return;
        }
    };

    if let Err(error) = send_paste(&mut enigo) {
        // Best effort: never leave the modifier logically pressed.
        let _ = enigo.key(modifier(), Release);
        eprintln!("cliphistory: failed to simulate the paste shortcut: {error}");
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::{parse_ydotool_major, ydotool_args};

    #[test]
    fn parses_ydotool_018_as_old() {
        assert_eq!(parse_ydotool_major("ydotool 0.1.8"), Some(0));
        assert_eq!(parse_ydotool_major("ydotool version 0.1.8-3"), Some(0));
    }

    #[test]
    fn parses_ydotool_1x_as_new() {
        assert_eq!(parse_ydotool_major("ydotool 1.0.4"), Some(1));
        assert_eq!(parse_ydotool_major("ydotool version 1.2.0"), Some(1));
    }

    #[test]
    fn ignores_output_without_a_version() {
        assert_eq!(parse_ydotool_major("usage: ydotool <cmd>"), None);
        assert_eq!(parse_ydotool_major(""), None);
    }

    #[test]
    fn old_ydotool_uses_symbolic_shortcut() {
        let expected: &[&str] = &["key", "ctrl+v"];

        assert_eq!(ydotool_args(Some(0)), expected);
        assert_eq!(ydotool_args(None), expected);
    }

    #[test]
    fn new_ydotool_uses_linux_keycodes() {
        let expected: &[&str] = &["key", "29:1", "47:1", "47:0", "29:0"];

        assert_eq!(ydotool_args(Some(1)), expected);
        assert_eq!(ydotool_args(Some(2)), expected);
    }
}

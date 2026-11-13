use std::thread;
use std::time::Duration;

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

/// How long to wait for the window manager to give focus back to the previous application
/// after the popup is hidden.
const FOCUS_DELAY: Duration = Duration::from_millis(250);

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
pub fn paste_clipboard() {
    thread::sleep(FOCUS_DELAY);

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

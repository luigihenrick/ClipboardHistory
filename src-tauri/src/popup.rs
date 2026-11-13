use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

pub(crate) const BLUR_GRACE: Duration = Duration::from_millis(600);

const FOCUS_CHECK_INTERVAL: Duration = Duration::from_millis(150);
const FOCUS_GIVE_UP_TICKS: u32 = 30;

/// Returns true when a focus-loss event should hide the popup.
///
/// Blur events that arrive before the popup has been focused at least once are ignored: on slow
/// X11/XWayland window managers the window can map late and emit a spurious focus-out while it
/// is still being focused. Blur events right after showing are also ignored.
pub(crate) fn should_hide_on_blur(
    hide_on_blur: bool,
    was_focused: bool,
    shown_at: Option<Instant>,
) -> bool {
    if !hide_on_blur || !was_focused {
        return false;
    }

    shown_at
        .map(|shown_at| shown_at.elapsed() >= BLUR_GRACE)
        .unwrap_or(true)
}

/// Returns true while the popup should keep requesting focus.
///
/// The window can be mapped several hundred milliseconds after `show()`, so a couple of
/// retries are not enough on slow X11/XWayland window managers.
fn should_retry_focus(attempt: u32) -> bool {
    attempt < FOCUS_GIVE_UP_TICKS
}

/// Decides whether the focus monitor should request focus again.
///
/// Focus is only requested until the popup is seen focused for the first time. After that the
/// user owns focus: if they click elsewhere we do not steal it back.
fn should_request_focus(was_focused: bool, focused: bool, attempt: u32) -> bool {
    !was_focused && !focused && should_retry_focus(attempt)
}

fn center_popup(window: &tauri::WebviewWindow) {
    if let Err(error) = window.center() {
        #[cfg(debug_assertions)]
        eprintln!("cliphistory: failed to center popup: {error}");
    }
}

/// Last-resort focus fallback for X11/XWayland window managers that ignore GTK's `present()`.
///
/// Raising the window and forcing the X input focus is more aggressive than `_NET_ACTIVE_WINDOW`,
/// but it is only used while the popup is visible and has not received focus yet.
#[cfg(target_os = "linux")]
fn force_x11_focus(window: &tauri::WebviewWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, InputFocus, StackMode,
    };

    let Ok(handle) = window.window_handle() else {
        return;
    };

    let xid = match handle.as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window as u32,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        _ => return,
    };
    if xid == 0 {
        return;
    }

    let Ok((connection, screen_num)) = x11rb::connect(None) else {
        return;
    };

    let _ =
        connection.configure_window(xid, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));

    // Ask the window manager to activate the window as a pager request (source 2), which
    // bypasses some focus-stealing-prevention heuristics on X11/XWayland.
    if let Ok(atom) = connection.intern_atom(false, b"_NET_ACTIVE_WINDOW") {
        if let Ok(reply) = atom.reply() {
            let root = connection.setup().roots[screen_num].root;
            let event =
                ClientMessageEvent::new(32, xid, reply.atom, [2u32, x11rb::CURRENT_TIME, 0, 0, 0]);
            let _ = connection.send_event(
                false,
                root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            );
        }
    }

    let _ = connection.set_input_focus(InputFocus::PARENT, xid, x11rb::CURRENT_TIME);
    let _ = connection.flush();
}

/// Asks the window manager to focus the popup.
///
/// On Linux, Tauri's `set_focus` ends up calling GTK's `present_with_time(GDK_CURRENT_TIME)`,
/// which GNOME/Mutter ignores due to focus-stealing prevention. Calling GTK's `present()`
/// directly (with focus-on-map enabled) is the known workaround.
fn focus_popup(app: &AppHandle, window: &tauri::WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        let app_handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            use gtk::prelude::GtkWindowExt;

            if let Some(window) = app_handle.get_webview_window("popup") {
                if !window.is_visible().unwrap_or(false) {
                    return;
                }

                match window.gtk_window() {
                    Ok(gtk_window) => {
                        gtk_window.set_accept_focus(true);
                        gtk_window.set_focus_on_map(true);
                        gtk_window.present();
                        force_x11_focus(&window);
                    }
                    Err(error) => {
                        #[cfg(debug_assertions)]
                        eprintln!("cliphistory: failed to access GTK window: {error}");
                    }
                }
            }
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = app;
        let _ = window.set_focus();
    }
}

/// Keeps requesting focus until the popup is focused, then hides it when focus is lost.
///
/// Some X11/XWayland window managers map the window late and do not deliver a reliable
/// focus-out event. Focus requests stop as soon as the window reports focus, so this does not
/// keep stealing focus from the user.
fn spawn_focus_monitor(app: AppHandle) {
    std::thread::spawn(move || {
        let mut was_focused = false;
        let mut focus_attempts = 0;

        loop {
            std::thread::sleep(FOCUS_CHECK_INTERVAL);

            let Some(window) = app.get_webview_window("popup") else {
                return;
            };
            if !window.is_visible().unwrap_or(false) {
                return;
            }

            let focused = window.is_focused().unwrap_or(false);
            if focused {
                if !was_focused {
                    if let Some(state) = app.try_state::<crate::AppState>() {
                        state.popup_was_focused.store(true, Ordering::SeqCst);
                    }

                    #[cfg(debug_assertions)]
                    eprintln!("cliphistory: popup focused");
                }
                was_focused = true;
                focus_attempts = 0;
                continue;
            }

            if !was_focused {
                if should_request_focus(was_focused, focused, focus_attempts) {
                    focus_attempts += 1;
                    focus_popup(&app, &window);
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("cliphistory: popup never gained focus; stopping focus monitor");
                    return;
                }
                continue;
            }

            let hide_on_blur = app
                .try_state::<crate::AppState>()
                .and_then(|state| {
                    state
                        .settings
                        .lock()
                        .ok()
                        .map(|settings| settings.hide_on_blur)
                })
                .unwrap_or(false);

            if !hide_on_blur {
                return;
            }

            #[cfg(debug_assertions)]
            eprintln!("cliphistory: popup lost focus");

            let shown_at = app.try_state::<crate::AppState>().and_then(|state| {
                state
                    .popup_shown_at
                    .lock()
                    .ok()
                    .and_then(|shown_at| *shown_at)
            });

            if should_hide_on_blur(hide_on_blur, was_focused, shown_at) {
                #[cfg(debug_assertions)]
                eprintln!("cliphistory: hiding popup after focus loss");
                let _ = window.hide();
                return;
            }
        }
    });
}

pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window("popup") else {
        eprintln!("cliphistory: window \"popup\" not found");
        return;
    };

    #[cfg(debug_assertions)]
    eprintln!("cliphistory: showing popup");

    // Re-activate the app on macOS when it was hidden after an automatic paste.
    #[cfg(target_os = "macos")]
    {
        let _ = app.show();
    }

    // Mark the show before focusing so spurious focus-loss events during startup are
    // ignored by the blur handler. The "was focused" flag also starts false: a focus-out
    // emitted while the window is still mapping must not close it before the focus retries.
    if let Some(state) = app.try_state::<crate::AppState>() {
        if let Ok(mut shown_at) = state.popup_shown_at.lock() {
            *shown_at = Some(Instant::now());
        }
        state.popup_was_focused.store(false, Ordering::SeqCst);
    }

    // Always show the popup centered; window managers are free to place/raise it, and this is
    // the most reliable behavior across Wayland and X11.
    center_popup(&window);

    // Start the frontend refresh before mapping the window so the list is ready sooner.
    let _ = window.emit("popup://shown", ());

    let _ = window.unminimize();
    if let Err(error) = window.show() {
        eprintln!("cliphistory: failed to show popup: {error}");
    }

    focus_popup(app, &window);
    spawn_focus_monitor(app.clone());
}

pub fn toggle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let visible = window.is_visible().unwrap_or(false);

        #[cfg(debug_assertions)]
        eprintln!("cliphistory: toggling popup (visible: {visible})");

        if visible {
            let _ = window.hide();
        } else {
            show(app);
        }
    }
}

pub fn open_settings(app: &AppHandle) {
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.hide();
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{
        should_hide_on_blur, should_request_focus, should_retry_focus, BLUR_GRACE,
        FOCUS_CHECK_INTERVAL, FOCUS_GIVE_UP_TICKS,
    };

    #[test]
    fn ignores_blur_while_the_grace_period_is_active() {
        assert!(!should_hide_on_blur(true, true, Some(Instant::now())));
    }

    #[test]
    fn ignores_blur_before_the_popup_was_focused() {
        assert!(!should_hide_on_blur(true, false, None));
        assert!(!should_hide_on_blur(
            true,
            false,
            Some(Instant::now() - Duration::from_secs(10))
        ));
    }

    #[test]
    fn hides_after_the_grace_period() {
        let shown_at = Instant::now() - BLUR_GRACE - Duration::from_millis(1);

        assert!(should_hide_on_blur(true, true, Some(shown_at)));
    }

    #[test]
    fn hides_when_there_is_no_show_timestamp() {
        assert!(should_hide_on_blur(true, true, None));
    }

    #[test]
    fn respects_the_hide_on_blur_setting() {
        assert!(!should_hide_on_blur(false, true, None));
        assert!(!should_hide_on_blur(
            false,
            true,
            Some(Instant::now() - Duration::from_secs(10))
        ));
    }

    #[test]
    fn keeps_requesting_focus_until_the_timeout() {
        assert!(should_retry_focus(0));
        assert!(should_retry_focus(FOCUS_GIVE_UP_TICKS - 1));
        assert!(!should_retry_focus(FOCUS_GIVE_UP_TICKS));
    }

    #[test]
    fn requests_focus_until_the_popup_is_focused() {
        // Both the global shortcut and `--toggle` end up in `show`, which starts this loop.
        assert!(should_request_focus(false, false, 0));
        assert!(should_request_focus(false, false, FOCUS_GIVE_UP_TICKS - 1));

        // Stop as soon as the window reports focus, or after the user has taken focus away.
        assert!(!should_request_focus(false, true, 0));
        assert!(!should_request_focus(true, false, 0));
        assert!(!should_request_focus(false, false, FOCUS_GIVE_UP_TICKS));
    }

    #[test]
    fn focus_retries_cover_slow_window_mapping() {
        let total = FOCUS_CHECK_INTERVAL * FOCUS_GIVE_UP_TICKS;

        assert!(
            total >= Duration::from_secs(4),
            "focus retries should last at least 4s, got {total:?}"
        );
    }
}

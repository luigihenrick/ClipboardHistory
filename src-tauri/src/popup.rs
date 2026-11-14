use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

pub(crate) const BLUR_GRACE: Duration = Duration::from_millis(600);

const FOCUS_CHECK_INTERVAL: Duration = Duration::from_millis(150);
// AppImage cold starts can take several seconds to map and focus the window. Keep retrying
// longer than the old 4.5s window so the first open still receives keyboard focus.
const FOCUS_GIVE_UP_TICKS: u32 = 80;

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

/// Geometry of a monitor as reported by the platform. Kept separate from Tauri's `Monitor` so
/// the primary-monitor selection can be unit-tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MonitorRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    primary: bool,
}

/// Returns the monitor explicitly marked as primary by the platform.
///
/// Do not infer the primary monitor from the desktop origin: a secondary monitor can be at
/// `(0, 0)` while the primary one is to its right.
fn primary_monitor_rect(monitors: &[MonitorRect]) -> Option<MonitorRect> {
    monitors
        .iter()
        .copied()
        .find(|monitor| monitor.primary && monitor.width > 0 && monitor.height > 0)
}

/// Rejects empty monitor/work-area rectangles. After a monitor wakes up, the platform can
/// briefly report a zero width or height; using it would place the popup at the monitor's left
/// edge instead of its center.
fn usable_area(x: i32, y: i32, width: u32, height: u32) -> Option<(i32, i32, u32, u32)> {
    (width > 0 && height > 0).then_some((x, y, width, height))
}

fn monitor_area(monitor: &tauri::Monitor) -> Option<(i32, i32, u32, u32)> {
    let area = monitor.work_area();
    usable_area(
        area.position.x,
        area.position.y,
        area.size.width,
        area.size.height,
    )
    .or_else(|| {
        let position = monitor.position();
        let size = monitor.size();
        usable_area(position.x, position.y, size.width, size.height)
    })
}

/// X11/XWayland exposes the primary output through RandR. This is more reliable than Tauri's
/// `primary_monitor()` on Linux, which can be missing or point at the wrong monitor.
#[cfg(target_os = "linux")]
fn x11_primary_monitor_rect() -> Option<MonitorRect> {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as RandrConnectionExt;

    let (connection, screen_num) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen_num)?.root;
    let reply = connection
        .randr_get_monitors(root, true)
        .ok()?
        .reply()
        .ok()?;

    let monitors: Vec<MonitorRect> = reply
        .monitors
        .into_iter()
        .map(|monitor| MonitorRect {
            x: monitor.x as i32,
            y: monitor.y as i32,
            width: monitor.width as u32,
            height: monitor.height as u32,
            primary: monitor.primary,
        })
        .collect();

    primary_monitor_rect(&monitors)
}

/// Returns the primary monitor's usable area as `(x, y, width, height)`.
fn primary_monitor_area(window: &tauri::WebviewWindow) -> Option<(i32, i32, u32, u32)> {
    #[cfg(target_os = "linux")]
    if let Some(primary) = x11_primary_monitor_rect() {
        if let Ok(monitors) = window.available_monitors() {
            if let Some(monitor) = monitors.iter().find(|monitor| {
                monitor.position().x == primary.x
                    && monitor.position().y == primary.y
                    && monitor.size().width == primary.width
                    && monitor.size().height == primary.height
            }) {
                if let Some(area) = monitor_area(monitor) {
                    return Some(area);
                }
            }
        }

        // RandR knows the primary monitor even if Tauri does not expose a matching monitor.
        return usable_area(primary.x, primary.y, primary.width, primary.height);
    }

    match window.primary_monitor() {
        Ok(Some(monitor)) => monitor_area(&monitor),
        Ok(None) => None,
        Err(error) => {
            eprintln!("cliphistory: failed to query the primary monitor: {error}");
            None
        }
    }
}

fn popup_size(window: &tauri::WebviewWindow) -> tauri::PhysicalSize<u32> {
    window
        .outer_size()
        .or_else(|_| window.inner_size())
        .ok()
        .filter(|size| size.width > 0 && size.height > 0)
        .unwrap_or(tauri::PhysicalSize::new(440, 560))
}

/// Calculates the primary-monitor position for the popup.
fn popup_position(window: &tauri::WebviewWindow) -> Option<tauri::PhysicalPosition<i32>> {
    let (area_x, area_y, area_width, area_height) = primary_monitor_area(window)?;
    let size = popup_size(window);
    let (x, y) = centered_position(
        area_x,
        area_y,
        area_width,
        area_height,
        size.width,
        size.height,
    );

    Some(tauri::PhysicalPosition::new(x, y))
}

/// Centers the popup on the primary monitor's work area, regardless of where the cursor or the
/// window currently is.
fn center_on_primary_monitor(window: &tauri::WebviewWindow) {
    let Some((area_x, area_y, area_width, area_height)) = primary_monitor_area(window) else {
        #[cfg(target_os = "linux")]
        eprintln!(
            "cliphistory: no primary monitor reported by X11 RandR or the windowing backend; \
             falling back to window.center()"
        );
        #[cfg(not(target_os = "linux"))]
        eprintln!(
            "cliphistory: no primary monitor reported by the platform; \
             falling back to window.center()"
        );

        if let Err(error) = window.center() {
            eprintln!("cliphistory: failed to center popup: {error}");
        }
        return;
    };

    let size = popup_size(window);
    let (x, y) = centered_position(
        area_x,
        area_y,
        area_width,
        area_height,
        size.width,
        size.height,
    );

    if let Err(error) = window.set_position(tauri::PhysicalPosition::new(x, y)) {
        eprintln!(
            "cliphistory: failed to center popup on the primary monitor at ({area_x}, {area_y}): \
             {error}"
        );
    }
}

fn apply_popup_position(
    window: &tauri::WebviewWindow,
    position: Option<tauri::PhysicalPosition<i32>>,
) {
    match position {
        Some(position) => {
            if let Err(error) = window.set_position(position) {
                eprintln!(
                    "cliphistory: failed to set the explicit popup position: {error}; \
                     centering on the primary monitor"
                );

                center_on_primary_monitor(window);
            }
        }
        None => center_on_primary_monitor(window),
    }
}

/// Calculates the top-left position that centers a window inside a monitor's work area.
fn centered_position(
    area_x: i32,
    area_y: i32,
    area_width: u32,
    area_height: u32,
    window_width: u32,
    window_height: u32,
) -> (i32, i32) {
    let x = area_x + (area_width.saturating_sub(window_width) / 2) as i32;
    let y = area_y + (area_height.saturating_sub(window_height) / 2) as i32;
    (x, y)
}

/// Retries for several seconds because idle monitor wake-up and AppImage cold starts can make
/// the window manager place the popup much later than the first mapped frame.
const POSITION_RETRY_DELAYS: [u64; 6] = [100, 200, 400, 800, 1500, 2500];

/// Window managers can override the position while mapping the window. Recompute and re-apply
/// the target over a few seconds so a monitor that was asleep does not leave the popup at the
/// left edge.
fn schedule_position_retry(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        for delay in POSITION_RETRY_DELAYS {
            std::thread::sleep(Duration::from_millis(delay));
            let app_for_main = app.clone();
            let _ = app.run_on_main_thread(move || {
                if let Some(window) = app_for_main.get_webview_window("popup") {
                    if window.is_visible().unwrap_or(false) {
                        let position = popup_position(&window);
                        apply_popup_position(&window, position);
                    }
                }
            });
        }
    });
}

/// Last-resort focus fallback for X11/XWayland window managers that ignore GTK's `present()`.
///
/// Raising the window and forcing the X input focus is more aggressive than `_NET_ACTIVE_WINDOW`,
/// but it is only used while the popup is visible and has not received focus yet.
#[cfg(target_os = "linux")]
fn activate_x11_window(xid: u32) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, InputFocus, StackMode,
    };

    if xid <= 1 {
        return;
    }

    let Ok((connection, screen_num)) = x11rb::connect(None) else {
        return;
    };
    let Some(screen) = connection.setup().roots.get(screen_num) else {
        return;
    };
    let root = screen.root;

    let _ =
        connection.configure_window(xid, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));

    // Ask the window manager to activate the window as a pager request (source 2), which
    // bypasses some focus-stealing-prevention heuristics on X11/XWayland.
    if let Ok(atom) = connection.intern_atom(false, b"_NET_ACTIVE_WINDOW") {
        if let Ok(reply) = atom.reply() {
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

#[cfg(target_os = "linux")]
fn window_xid(window: &tauri::WebviewWindow) -> Option<u32> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Xlib(handle) => Some(handle.window as u32),
        RawWindowHandle::Xcb(handle) => Some(handle.window.get()),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn force_x11_focus(window: &tauri::WebviewWindow) {
    if let Some(xid) = window_xid(window) {
        activate_x11_window(xid);
    }
}

#[cfg(target_os = "linux")]
fn active_x11_window() -> Option<u32> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

    let (connection, screen_num) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen_num)?.root;
    let atom = connection
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let reply = connection
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    if let Some(xid) = reply.value32().and_then(|mut values| values.next()) {
        if xid > 1 {
            return Some(xid);
        }
    }

    let focus = connection.get_input_focus().ok()?.reply().ok()?.focus;
    (focus > 1).then_some(focus)
}

/// Remembers the window that was active before the popup opened, so auto-paste can return
/// focus to it after the popup is hidden.
pub(crate) fn remember_previous_window(app: &AppHandle) {
    #[cfg(target_os = "linux")]
    if let Some(state) = app.try_state::<crate::AppState>() {
        let popup_xid = app
            .get_webview_window("popup")
            .and_then(|window| window_xid(&window));

        if let Some(active) = active_x11_window() {
            if Some(active) != popup_xid {
                if let Ok(mut previous) = state.previous_x11_window.lock() {
                    *previous = Some(active);
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    let _ = app;
}

/// Restores focus to the window that was active before the popup opened.
pub(crate) fn restore_previous_window(app: &AppHandle) {
    #[cfg(target_os = "linux")]
    {
        let previous = app.try_state::<crate::AppState>().and_then(|state| {
            state
                .previous_x11_window
                .lock()
                .ok()
                .and_then(|mut previous| previous.take())
        });

        if let Some(xid) = previous {
            activate_x11_window(xid);
        }
    }

    #[cfg(not(target_os = "linux"))]
    let _ = app;
}

/// Asks the window manager to focus the popup.
///
/// On Linux, Tauri's `set_focus` ends up calling GTK's `present_with_time(GDK_CURRENT_TIME)`,
/// which GNOME/Mutter ignores due to focus-stealing prevention. Calling GTK's `present()`
/// directly (with focus-on-map enabled) is the known workaround.
fn focus_popup(app: &AppHandle, window: &tauri::WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        let window = window.clone();
        let _ = app.run_on_main_thread(move || {
            use gtk::prelude::GtkWindowExt;

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

                    // Tell the webview that the native window actually got focus. This is more
                    // reliable than the DOM focus event during slow AppImage cold starts.
                    let _ = app.emit("popup://focused", ());

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

    // Remember which app/window was active before the popup opens so auto-paste can return
    // focus there after hiding it.
    if !window.is_visible().unwrap_or(false) {
        remember_previous_window(app);
    }

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

    // Always open centered on the primary monitor. Capture the position before showing so the
    // post-map retries reapply the same target.
    let position = popup_position(&window);
    apply_popup_position(&window, position.clone());

    // Start the frontend refresh before mapping the window so the list is ready sooner.
    let _ = window.emit("popup://shown", ());

    let _ = window.unminimize();
    if let Err(error) = window.show() {
        eprintln!("cliphistory: failed to show popup: {error}");
    }

    // The window manager may place the window while mapping it; apply the same position once
    // more and verify again shortly after the first frame.
    apply_popup_position(&window, position.clone());
    schedule_position_retry(app);

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
        centered_position, primary_monitor_rect, should_hide_on_blur, should_request_focus,
        should_retry_focus, usable_area, MonitorRect, BLUR_GRACE, FOCUS_CHECK_INTERVAL,
        FOCUS_GIVE_UP_TICKS, POSITION_RETRY_DELAYS,
    };

    #[test]
    fn centers_window_in_the_primary_monitor() {
        assert_eq!(centered_position(0, 0, 1920, 1080, 440, 560), (740, 260));
    }

    #[test]
    fn centers_window_on_an_offset_monitor() {
        assert_eq!(
            centered_position(1920, 0, 2560, 1440, 440, 560),
            (2980, 440)
        );
    }

    #[test]
    fn centers_window_on_a_negative_origin_monitor() {
        assert_eq!(
            centered_position(-1920, -100, 2560, 1440, 440, 560),
            (-860, 340)
        );
    }

    #[test]
    fn keeps_the_window_origin_when_it_is_larger_than_the_work_area() {
        assert_eq!(centered_position(100, 50, 300, 200, 440, 560), (100, 50));
    }

    #[test]
    fn finds_the_primary_monitor_when_the_secondary_is_at_the_origin() {
        let secondary = MonitorRect {
            x: 0,
            y: 751,
            width: 1920,
            height: 1200,
            primary: false,
        };
        let primary = MonitorRect {
            x: 1920,
            y: 0,
            width: 3440,
            height: 1440,
            primary: true,
        };

        assert_eq!(primary_monitor_rect(&[secondary, primary]), Some(primary));
    }

    #[test]
    fn does_not_assume_the_monitor_at_the_origin_is_primary() {
        let secondary = MonitorRect {
            x: 0,
            y: 751,
            width: 1920,
            height: 1200,
            primary: false,
        };

        assert_eq!(primary_monitor_rect(&[secondary]), None);
    }

    #[test]
    fn ignores_a_zero_sized_primary_monitor_after_wakeup() {
        let waking_primary = MonitorRect {
            x: 1920,
            y: 0,
            width: 0,
            height: 1440,
            primary: true,
        };

        assert_eq!(primary_monitor_rect(&[waking_primary]), None);
    }

    #[test]
    fn rejects_zero_sized_work_areas() {
        assert_eq!(usable_area(1920, 0, 0, 1440), None);
        assert_eq!(usable_area(1920, 0, 3440, 0), None);
        assert_eq!(
            usable_area(1920, 0, 3440, 1440),
            Some((1920, 0, 3440, 1440))
        );
    }

    #[test]
    fn position_retries_cover_idle_monitor_wakeup() {
        let total: u64 = POSITION_RETRY_DELAYS.iter().sum();

        assert!(
            total >= 5_000,
            "position retries should last at least 5s, got {total}ms"
        );
    }

    #[test]
    fn centers_on_a_primary_monitor_to_the_right_of_the_secondary() {
        assert_eq!(
            centered_position(1920, 0, 3440, 1440, 440, 560),
            (3420, 440)
        );
    }

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
    fn focus_retries_cover_appimage_cold_starts() {
        let total = FOCUS_CHECK_INTERVAL * FOCUS_GIVE_UP_TICKS;

        assert!(
            total >= Duration::from_secs(10),
            "focus retries should last at least 10s, got {total:?}"
        );
    }
}

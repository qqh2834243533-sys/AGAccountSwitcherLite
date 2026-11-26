use std::{
    env, fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{GetLastError, ERROR_ACCESS_DENIED, ERROR_NOT_ENOUGH_QUOTA, HWND, LPARAM},
    UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowLongW, GetWindowThreadProcessId, IsWindowVisible,
        PostMessageW, GWL_EXSTYLE, WM_CLOSE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GracefulExit {
    /// All verified client writer processes exited cleanly on their own.
    ProcessExited,
    /// Target processes had no visible user windows (already tray-resident) or all
    /// visible user windows closed/hid to tray cleanly after `WM_CLOSE`.
    HiddenToTray,
}

fn is_ignored_helper_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "Electron_NotifyIconHostWindow"
            | "Chrome_WidgetWin_0"
            | "tooltips_class32"
            | "IME"
            | "MSCTFIME UI"
            | "UAC_InputIndicatorOverlayWnd"
            | "UAC Input Indicator"
    ) || (class_name.starts_with("UAC") && class_name.contains("Indicator"))
}

/// Identify visible user-facing application windows and dialogs while excluding
/// OS-injected accessibility overlays, click-through layers, IME helpers, and tray hosts.
fn is_app_user_window(window: HWND) -> bool {
    if unsafe { IsWindowVisible(window) } == 0 {
        return false;
    }

    let mut cls = [0u16; 128];
    let len = unsafe { GetClassNameW(window, cls.as_mut_ptr(), cls.len() as i32) };
    if len > 0 {
        let class_name = String::from_utf16_lossy(&cls[..len as usize]);
        if is_ignored_helper_class(class_name.trim()) {
            return false;
        }
    }

    let ex_style = unsafe { GetWindowLongW(window, GWL_EXSTYLE) } as u32;
    let is_app_window = (ex_style & WS_EX_APPWINDOW) != 0;
    let is_non_interactive_overlay =
        (ex_style & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT)) != 0;

    is_app_window || !is_non_interactive_overlay
}

#[derive(Clone, Copy)]
struct AppWindow {
    handle: HWND,
    pid: u32,
    thread: u32,
}

impl AppWindow {
    fn still_visible(self) -> bool {
        let mut pid = 0;
        let thread = unsafe { GetWindowThreadProcessId(self.handle, &mut pid) };
        pid == self.pid && thread == self.thread && is_app_user_window(self.handle)
    }
}

fn visible_windows(pids: &[u32]) -> Result<Vec<AppWindow>, &'static str> {
    struct Ctx<'a> {
        pids: &'a [u32],
        windows: Vec<AppWindow>,
    }

    unsafe extern "system" fn visit(window: HWND, data: LPARAM) -> i32 {
        let ctx = unsafe { &mut *(data as *mut Ctx<'_>) };
        let mut pid = 0u32;
        let thread = unsafe { GetWindowThreadProcessId(window, &mut pid) };
        if thread != 0 && ctx.pids.contains(&pid) && is_app_user_window(window) {
            ctx.windows.push(AppWindow {
                handle: window,
                pid,
                thread,
            });
        }
        1
    }

    let mut ctx = Ctx {
        pids,
        windows: Vec::new(),
    };
    if unsafe { EnumWindows(Some(visit), (&mut ctx as *mut Ctx<'_>) as LPARAM) } == 0 {
        return Err("client_state_unknown");
    }
    Ok(ctx.windows)
}

fn post_close_message(window: HWND) -> Result<(), u32> {
    if unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } != 0 {
        Ok(())
    } else {
        Err(unsafe { GetLastError() })
    }
}

fn post_close_to_visible_windows(
    windows: &[AppWindow],
    mut post: impl FnMut(HWND) -> Result<(), u32>,
) -> Result<(), &'static str> {
    // Use the initial window list, so a new dialog opened in response to WM_CLOSE
    // is checked by the visibility gate and never receives another close request.
    for window in windows {
        for attempt in 0..2 {
            if !window.still_visible() {
                break;
            }
            match post(window.handle) {
                Ok(()) => break,
                Err(_) if !window.still_visible() => break,
                Err(ERROR_NOT_ENOUGH_QUOTA) if attempt == 0 => {
                    thread::sleep(Duration::from_millis(50))
                }
                Err(ERROR_ACCESS_DENIED) => return Err("close_message_denied"),
                Err(_) => return Err("close_failed"),
            }
        }
    }
    Ok(())
}

/// Phase 1 of silent shutdown: send `WM_CLOSE` to visible Antigravity windows so
/// renderer state flushes cleanly (zero mouse, zero keyboard, zero tray popup).
///
/// Safety gate (1-A): If any visible user window remains open when `timeout` expires
/// (for instance, blocked by an unsaved-changes prompt or hung UI), this function
/// returns `Err("close_timeout")` so the caller aborts without calling `TerminateProcess`.
pub fn close_windows_gracefully(
    app_pids: &[u32],
    is_closed: impl Fn() -> Result<(), &'static str>,
    timeout: Duration,
) -> Result<GracefulExit, &'static str> {
    let windows = visible_windows(app_pids)?;
    if windows.is_empty() {
        return match is_closed() {
            Ok(()) => Ok(GracefulExit::ProcessExited),
            Err("client_running") => Ok(GracefulExit::HiddenToTray),
            Err(reason) => Err(reason),
        };
    }

    post_close_to_visible_windows(&windows, post_close_message)?;

    let deadline = Instant::now() + timeout;
    let settle_window = Duration::from_millis(450).min(timeout);
    let mut windows_hidden_since: Option<Instant> = None;

    while Instant::now() < deadline {
        match is_closed() {
            Ok(()) => return Ok(GracefulExit::ProcessExited),
            Err("client_running") => {
                if visible_windows(app_pids)?.is_empty() {
                    let hidden_at = *windows_hidden_since.get_or_insert_with(Instant::now);
                    if hidden_at.elapsed() >= settle_window {
                        return Ok(GracefulExit::HiddenToTray);
                    }
                } else {
                    windows_hidden_since = None;
                }
                thread::sleep(Duration::from_millis(80));
            }
            Err(reason) => return Err(reason),
        }
    }

    match is_closed() {
        Ok(()) => Ok(GracefulExit::ProcessExited),
        Err("client_running") => {
            if !visible_windows(app_pids)?.is_empty() {
                return Err("close_timeout");
            }
            if let Some(hidden_at) = windows_hidden_since {
                let remaining_settle = settle_window.saturating_sub(hidden_at.elapsed());
                if !remaining_settle.is_zero() {
                    thread::sleep(remaining_settle);
                }
            }
            if !visible_windows(app_pids)?.is_empty() {
                return Err("close_timeout");
            }
            match is_closed() {
                Ok(()) => Ok(GracefulExit::ProcessExited),
                Err("client_running") => Ok(GracefulExit::HiddenToTray),
                Err(reason) => Err(reason),
            }
        }
        Err(reason) => Err(reason),
    }
}

/// Remove stale `%APPDATA%\Antigravity\DevToolsActivePort` after shutdown so CDP
/// watchers (such as `agy-claude-style-watcher`) wait for the fresh port written
/// by the relaunched Antigravity instance instead of probing a dead port.
pub fn clear_stale_devtools_port() {
    if let Some(appdata) = env::var_os("APPDATA") {
        let port_file = PathBuf::from(appdata)
            .join("Antigravity")
            .join("DevToolsActivePort");
        if port_file.is_file() {
            let _ = fs::remove_file(port_file);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, PeekMessageW, ShowWindow, MSG, PM_REMOVE, SW_HIDE,
        WS_POPUP, WS_VISIBLE,
    };

    static WINDOW_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn posts_wm_close_only_to_visible_user_windows_and_ignores_overlay_toolwindows() {
        let _guard = WINDOW_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let visible = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                null(),
                WS_POPUP | WS_VISIBLE,
                -32000,
                -32000,
                1,
                1,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!visible.is_null());
        let hidden = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                null(),
                WS_POPUP,
                -32000,
                -32000,
                1,
                1,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!hidden.is_null());
        let overlay = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
                class.as_ptr(),
                null(),
                WS_POPUP | WS_VISIBLE,
                -32000,
                -32000,
                1,
                1,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!overlay.is_null());

        let windows = visible_windows(&[std::process::id()]).unwrap();
        let [target] = windows.as_slice() else {
            panic!("expected only the user window")
        };
        let stale = AppWindow {
            thread: 0,
            ..*target
        };
        assert_eq!(
            post_close_to_visible_windows(&[stale], |_| panic!("changed window owner")),
            Ok(())
        );
        let mut calls = 0;
        assert_eq!(
            post_close_to_visible_windows(&windows, |_| {
                calls += 1;
                if calls == 1 {
                    Err(ERROR_NOT_ENOUGH_QUOTA)
                } else {
                    Ok(())
                }
            }),
            Ok(())
        );
        assert_eq!(calls, 2);
        assert_eq!(
            post_close_to_visible_windows(&windows, |_| Err(ERROR_ACCESS_DENIED)),
            Err("close_message_denied")
        );
        post_close_to_visible_windows(&windows, post_close_message).unwrap();

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        let visible_got_close =
            unsafe { PeekMessageW(&mut msg, visible, WM_CLOSE, WM_CLOSE, PM_REMOVE) } != 0;
        let hidden_got_close =
            unsafe { PeekMessageW(&mut msg, hidden, WM_CLOSE, WM_CLOSE, PM_REMOVE) } != 0;
        let overlay_got_close =
            unsafe { PeekMessageW(&mut msg, overlay, WM_CLOSE, WM_CLOSE, PM_REMOVE) } != 0;

        unsafe {
            DestroyWindow(overlay);
            DestroyWindow(hidden);
            DestroyWindow(visible);
        }

        assert!(visible_got_close);
        assert!(!hidden_got_close);
        assert!(!overlay_got_close);
    }

    #[test]
    fn aborts_with_close_timeout_when_visible_window_remains_open() {
        let _guard = WINDOW_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let blocking_window = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                null(),
                WS_POPUP | WS_VISIBLE,
                -32000,
                -32000,
                1,
                1,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!blocking_window.is_null());

        let result = close_windows_gracefully(
            &[std::process::id()],
            || Err("client_running"),
            Duration::from_millis(220),
        );

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        let _ = unsafe { PeekMessageW(&mut msg, blocking_window, WM_CLOSE, WM_CLOSE, PM_REMOVE) };
        unsafe {
            DestroyWindow(blocking_window);
        }

        assert_eq!(result, Err("close_timeout"));
    }

    #[test]
    fn distinguishes_hidden_to_tray_from_full_process_exit() {
        let _guard = WINDOW_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let tray_window = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                null(),
                WS_POPUP | WS_VISIBLE,
                -32000,
                -32000,
                1,
                1,
                null_mut(),
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!tray_window.is_null());

        let checks = AtomicUsize::new(0);
        let result = close_windows_gracefully(
            &[std::process::id()],
            || {
                if checks.fetch_add(1, Ordering::SeqCst) == 0 {
                    unsafe { ShowWindow(tray_window, SW_HIDE) };
                }
                Err("client_running")
            },
            Duration::from_millis(150),
        );

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        let _ = unsafe { PeekMessageW(&mut msg, tray_window, WM_CLOSE, WM_CLOSE, PM_REMOVE) };
        unsafe {
            DestroyWindow(tray_window);
        }

        assert_eq!(result, Ok(GracefulExit::HiddenToTray));

        let exited = close_windows_gracefully(&[u32::MAX], || Ok(()), Duration::from_millis(100));
        assert_eq!(exited, Ok(GracefulExit::ProcessExited));
    }
}

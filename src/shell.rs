use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env, fs, io,
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_HOTKEY_ALREADY_REGISTERED, ERROR_SUCCESS},
    Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    },
    System::{
        Registry::{
            RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
            RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, KEY_WRITE,
            REG_BINARY, REG_SZ,
        },
        Threading::GetCurrentThreadId,
    },
    UI::{
        Input::KeyboardAndMouse::{
            RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
            MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
        },
        WindowsAndMessaging::{
            GetMessageW, PeekMessageW, PostThreadMessageW, MSG, PM_NOREMOVE, WM_APP, WM_HOTKEY,
        },
    },
};

pub const AUTOSTART_ARG: &str = "--autostart";
pub const DEFAULT_HOTKEY: &str = "Alt+Shift+G";

const SHELL_CONFIG_FILE: &str = "shell.json";
const RUN_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_SUBKEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const INTL_SUBKEY: &str = r"Control Panel\International";
const RUN_VALUE_NAME: &str = "AGAccountSwitcherLite";
const WM_HOTKEY_CMD: u32 = WM_APP + 1;
const PRODUCTION_HOTKEY_ID: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotkeyStatus {
    Active,
    Disabled,
    Conflict,
}

impl HotkeyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Conflict => "conflict",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    List,
    #[default]
    Matrix,
}

impl LayoutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Matrix => "matrix",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "list" => Ok(Self::List),
            "matrix" => Ok(Self::Matrix),
            _ => Err("invalid_preferences"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err("invalid_preferences"),
        }
    }

    pub fn to_tauri_theme(self) -> Option<tauri::Theme> {
        match self {
            Self::System => None,
            Self::Light => Some(tauri::Theme::Light),
            Self::Dark => Some(tauri::Theme::Dark),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiPrefs {
    pub layout: LayoutMode,
    pub theme: ThemeMode,
    pub particles_follow: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            layout: LayoutMode::Matrix,
            theme: ThemeMode::System,
            particles_follow: true,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellStatus {
    pub autostart: bool,
    pub autostart_blocked: bool,
    pub hotkey: Option<String>,
    pub hotkey_status: &'static str,
    pub layout: &'static str,
    pub theme: &'static str,
    pub particles_follow: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellConfig {
    hotkey: Option<String>,
    layout: LayoutMode,
    theme: ThemeMode,
    particles_follow: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawShellConfig {
    #[serde(default = "default_raw_hotkey")]
    hotkey: Option<String>,
    layout: Option<String>,
    theme: Option<String>,
    particles_follow: Option<bool>,
}

fn default_raw_hotkey() -> Option<String> {
    Some(DEFAULT_HOTKEY.to_owned())
}

fn default_shell_config() -> ShellConfig {
    let prefs = UiPrefs::default();
    ShellConfig {
        hotkey: Some(DEFAULT_HOTKEY.to_owned()),
        layout: prefs.layout,
        theme: prefs.theme,
        particles_follow: prefs.particles_follow,
    }
}

fn load_shell_config(data_root: &Path) -> ShellConfig {
    let path = data_root.join(SHELL_CONFIG_FILE);
    let Ok(raw) = fs::read_to_string(path) else {
        return default_shell_config();
    };
    let Ok(parsed) = serde_json::from_str::<RawShellConfig>(&raw) else {
        return default_shell_config();
    };
    let hotkey = match parsed.hotkey {
        None => None,
        Some(text) if text.trim().is_empty() => None,
        Some(text) => match Shortcut::parse(&text) {
            Ok(shortcut) => Some(shortcut.canonical()),
            Err(_) => Some(DEFAULT_HOTKEY.to_owned()),
        },
    };
    let layout = parsed
        .layout
        .as_deref()
        .and_then(|s| LayoutMode::parse(s).ok())
        .unwrap_or_default();
    let theme = parsed
        .theme
        .as_deref()
        .and_then(|s| ThemeMode::parse(s).ok())
        .unwrap_or_default();
    let particles_follow = parsed.particles_follow.unwrap_or(true);

    ShellConfig {
        hotkey,
        layout,
        theme,
        particles_follow,
    }
}

fn save_shell_config(
    data_root: &Path,
    hotkey: Option<&str>,
    prefs: UiPrefs,
) -> Result<(), &'static str> {
    fs::create_dir_all(data_root).map_err(|_| "data_folder_unavailable")?;
    let payload = ShellConfig {
        hotkey: hotkey.map(str::to_owned),
        layout: prefs.layout,
        theme: prefs.theme,
        particles_follow: prefs.particles_follow,
    };
    let serialized = serde_json::to_string_pretty(&payload).map_err(|_| "operation_failed")?;
    let final_path = data_root.join(SHELL_CONFIG_FILE);
    let temp_path = data_root.join(format!("{SHELL_CONFIG_FILE}.{}.tmp", std::process::id()));
    fs::write(&temp_path, &serialized).map_err(|_| "data_folder_unavailable")?;
    if fs::rename(&temp_path, &final_path).is_err() {
        let write_result = fs::write(&final_path, &serialized);
        let _ = fs::remove_file(&temp_path);
        write_result.map_err(|_| "data_folder_unavailable")?;
    }
    Ok(())
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

struct OwnedRegKey(HKEY);

impl Drop for OwnedRegKey {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }
}

fn read_reg_sz(subkey: &str, value_name: &str) -> Option<String> {
    let subkey_w = wide(subkey);
    let name_w = wide(value_name);
    let mut raw_key: HKEY = null_mut();
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut raw_key,
        )
    } != ERROR_SUCCESS
    {
        return None;
    }
    let key = OwnedRegKey(raw_key);
    let mut kind = 0u32;
    let mut byte_len = 0u32;
    if unsafe {
        RegQueryValueExW(
            key.0,
            name_w.as_ptr(),
            null(),
            &mut kind,
            null_mut(),
            &mut byte_len,
        )
    } != ERROR_SUCCESS
        || kind != REG_SZ
        || byte_len < 2
    {
        return None;
    }
    let mut words = vec![0u16; (byte_len as usize).div_ceil(2)];
    let mut actual_bytes = (words.len() * 2) as u32;
    if unsafe {
        RegQueryValueExW(
            key.0,
            name_w.as_ptr(),
            null(),
            &mut kind,
            words.as_mut_ptr().cast(),
            &mut actual_bytes,
        )
    } != ERROR_SUCCESS
        || kind != REG_SZ
    {
        return None;
    }
    let word_count = (actual_bytes as usize) / 2;
    let slice = &words[..word_count];
    let end = slice.iter().position(|&ch| ch == 0).unwrap_or(slice.len());
    Some(String::from_utf16_lossy(&slice[..end]))
}

fn is_startup_blocked_by_system(approved_subkey: &str, value_name: &str) -> bool {
    let subkey_w = wide(approved_subkey);
    let name_w = wide(value_name);
    let mut raw_key: HKEY = null_mut();
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut raw_key,
        )
    } != ERROR_SUCCESS
    {
        return false;
    }
    let key = OwnedRegKey(raw_key);
    let mut kind = 0u32;
    let mut buf = [0u8; 24];
    let mut byte_len = buf.len() as u32;
    if unsafe {
        RegQueryValueExW(
            key.0,
            name_w.as_ptr(),
            null(),
            &mut kind,
            buf.as_mut_ptr(),
            &mut byte_len,
        )
    } != ERROR_SUCCESS
        || kind != REG_BINARY
        || byte_len == 0
    {
        return false;
    }
    // Windows Task Manager / Startup Apps sets an odd non-zero flag byte (e.g. 0x03)
    // when a user disables an item in system settings, versus 0x02 / 0x06 when enabled.
    buf[0] != 0 && (buf[0] & 1) == 1
}

fn format_autostart_command(executable: &Path) -> String {
    format!("\"{}\" {AUTOSTART_ARG}", executable.display())
}

fn write_reg_sz(subkey: &str, value_name: &str, value: &str) -> Result<(), &'static str> {
    let subkey_w = wide(subkey);
    let name_w = wide(value_name);
    let value_w = wide(value);
    let mut raw_key: HKEY = null_mut();
    if unsafe { RegCreateKeyW(HKEY_CURRENT_USER, subkey_w.as_ptr(), &mut raw_key) } != ERROR_SUCCESS
    {
        return Err("autostart_unavailable");
    }
    let key = OwnedRegKey(raw_key);
    let status = unsafe {
        RegSetValueExW(
            key.0,
            name_w.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr().cast(),
            (value_w.len() * 2) as u32,
        )
    };
    if status != ERROR_SUCCESS {
        return Err("autostart_unavailable");
    }
    Ok(())
}

fn delete_reg_value(subkey: &str, value_name: &str) -> Result<(), &'static str> {
    let subkey_w = wide(subkey);
    let name_w = wide(value_name);
    let mut raw_key: HKEY = null_mut();
    let open_status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            KEY_SET_VALUE | KEY_WRITE,
            &mut raw_key,
        )
    };
    if open_status == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    if open_status != ERROR_SUCCESS {
        return Err("autostart_unavailable");
    }
    let key = OwnedRegKey(raw_key);
    let del_status = unsafe { RegDeleteValueW(key.0, name_w.as_ptr()) };
    if del_status == ERROR_SUCCESS || del_status == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err("autostart_unavailable")
    }
}

fn inspect_autostart_in(
    run_subkey: &str,
    approved_subkey: &str,
    value_name: &str,
    current_exe: Option<&Path>,
) -> (bool, bool) {
    let Some(existing) = read_reg_sz(run_subkey, value_name).filter(|s| !s.trim().is_empty())
    else {
        return (false, false);
    };
    if let Some(exe) = current_exe {
        let expected = format_autostart_command(exe);
        if existing != expected {
            let _ = write_reg_sz(run_subkey, value_name, &expected);
        }
    }
    let blocked = is_startup_blocked_by_system(approved_subkey, value_name);
    (true, blocked)
}

fn set_autostart_in(
    run_subkey: &str,
    value_name: &str,
    executable: &Path,
    enabled: bool,
) -> Result<(), &'static str> {
    if enabled {
        write_reg_sz(
            run_subkey,
            value_name,
            &format_autostart_command(executable),
        )
    } else {
        delete_reg_value(run_subkey, value_name)
    }
}

fn is_chinese_locale() -> bool {
    read_reg_sz(INTL_SUBKEY, "LocaleName")
        .is_some_and(|locale| locale.to_ascii_lowercase().starts_with("zh"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: HOT_KEY_MODIFIERS,
    vk: u32,
    key_name: &'static str,
}

impl Shortcut {
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("invalid_hotkey");
        }
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut win = false;
        let mut key: Option<(u32, &'static str)> = None;

        for part in trimmed.split('+') {
            let token = part.trim();
            if token.is_empty() {
                return Err("invalid_hotkey");
            }
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => {
                    if ctrl {
                        return Err("invalid_hotkey");
                    }
                    ctrl = true;
                }
                "alt" | "menu" => {
                    if alt {
                        return Err("invalid_hotkey");
                    }
                    alt = true;
                }
                "shift" => {
                    if shift {
                        return Err("invalid_hotkey");
                    }
                    shift = true;
                }
                "win" | "meta" | "super" => {
                    if win {
                        return Err("invalid_hotkey");
                    }
                    win = true;
                }
                _ => {
                    if key.is_some() {
                        return Err("invalid_hotkey");
                    }
                    key = Some(parse_main_key(token)?);
                }
            }
        }

        if !(ctrl || alt || win) {
            return Err("invalid_hotkey");
        }
        let (vk, key_name) = key.ok_or("invalid_hotkey")?;
        if is_reserved_combo(ctrl, alt, shift, win, key_name) {
            return Err("invalid_hotkey");
        }

        let mut modifiers: HOT_KEY_MODIFIERS = 0;
        if ctrl {
            modifiers |= MOD_CONTROL;
        }
        if alt {
            modifiers |= MOD_ALT;
        }
        if shift {
            modifiers |= MOD_SHIFT;
        }
        if win {
            modifiers |= MOD_WIN;
        }
        Ok(Self {
            modifiers,
            vk,
            key_name,
        })
    }

    pub fn canonical(self) -> String {
        let mut parts = Vec::with_capacity(5);
        if (self.modifiers & MOD_CONTROL) != 0 {
            parts.push("Ctrl");
        }
        if (self.modifiers & MOD_ALT) != 0 {
            parts.push("Alt");
        }
        if (self.modifiers & MOD_SHIFT) != 0 {
            parts.push("Shift");
        }
        if (self.modifiers & MOD_WIN) != 0 {
            parts.push("Win");
        }
        parts.push(self.key_name);
        parts.join("+")
    }
}

fn parse_main_key(token: &str) -> Result<(u32, &'static str), &'static str> {
    const LETTERS: [(&str, u32); 26] = [
        ("A", 0x41),
        ("B", 0x42),
        ("C", 0x43),
        ("D", 0x44),
        ("E", 0x45),
        ("F", 0x46),
        ("G", 0x47),
        ("H", 0x48),
        ("I", 0x49),
        ("J", 0x4A),
        ("K", 0x4B),
        ("L", 0x4C),
        ("M", 0x4D),
        ("N", 0x4E),
        ("O", 0x4F),
        ("P", 0x50),
        ("Q", 0x51),
        ("R", 0x52),
        ("S", 0x53),
        ("T", 0x54),
        ("U", 0x55),
        ("V", 0x56),
        ("W", 0x57),
        ("X", 0x58),
        ("Y", 0x59),
        ("Z", 0x5A),
    ];
    const DIGITS: [(&str, u32); 10] = [
        ("0", 0x30),
        ("1", 0x31),
        ("2", 0x32),
        ("3", 0x33),
        ("4", 0x34),
        ("5", 0x35),
        ("6", 0x36),
        ("7", 0x37),
        ("8", 0x38),
        ("9", 0x39),
    ];
    const FKEYS: [(&str, u32); 12] = [
        ("F1", 0x70),
        ("F2", 0x71),
        ("F3", 0x72),
        ("F4", 0x73),
        ("F5", 0x74),
        ("F6", 0x75),
        ("F7", 0x76),
        ("F8", 0x77),
        ("F9", 0x78),
        ("F10", 0x79),
        ("F11", 0x7A),
        ("F12", 0x7B),
    ];

    let upper = token.to_ascii_uppercase();
    if let Some(&(name, vk)) = LETTERS.iter().find(|&&(name, _)| name == upper) {
        return Ok((vk, name));
    }
    if let Some(&(name, vk)) = DIGITS.iter().find(|&&(name, _)| name == upper) {
        return Ok((vk, name));
    }
    if let Some(&(name, vk)) = FKEYS.iter().find(|&&(name, _)| name == upper) {
        return Ok((vk, name));
    }
    match upper.as_str() {
        "SPACE" => Ok((0x20, "Space")),
        "TAB" => Ok((0x09, "Tab")),
        "ENTER" | "RETURN" => Ok((0x0D, "Enter")),
        "BACKSPACE" => Ok((0x08, "Backspace")),
        "INSERT" | "INS" => Ok((0x2D, "Insert")),
        "DELETE" | "DEL" => Ok((0x2E, "Delete")),
        "HOME" => Ok((0x24, "Home")),
        "END" => Ok((0x23, "End")),
        "PAGEUP" | "PGUP" => Ok((0x21, "PageUp")),
        "PAGEDOWN" | "PGDN" => Ok((0x22, "PageDown")),
        "UP" | "ARROWUP" => Ok((0x26, "Up")),
        "DOWN" | "ARROWDOWN" => Ok((0x28, "Down")),
        "LEFT" | "ARROWLEFT" => Ok((0x25, "Left")),
        "RIGHT" | "ARROWRIGHT" => Ok((0x27, "Right")),
        "`" | "BACKQUOTE" | "GRAVE" => Ok((0xC0, "`")),
        "-" | "MINUS" => Ok((0xBD, "-")),
        "=" | "EQUAL" | "PLUS" => Ok((0xBB, "=")),
        "[" | "BRACKETLEFT" => Ok((0xDB, "[")),
        "]" | "BRACKETRIGHT" => Ok((0xDD, "]")),
        "\\" | "BACKSLASH" => Ok((0xDC, "\\")),
        ";" | "SEMICOLON" => Ok((0xBA, ";")),
        "'" | "QUOTE" => Ok((0xDE, "'")),
        "," | "COMMA" => Ok((0xBC, ",")),
        "." | "PERIOD" => Ok((0xBE, ".")),
        "/" | "SLASH" => Ok((0xBF, "/")),
        _ => Err("invalid_hotkey"),
    }
}

fn is_reserved_combo(ctrl: bool, alt: bool, shift: bool, win: bool, key: &str) -> bool {
    if ctrl && alt && !shift && !win && key == "Delete" {
        return true;
    }
    if alt && !ctrl && !shift && !win && matches!(key, "F4" | "Tab" | "Space") {
        return true;
    }
    if ctrl
        && !alt
        && !shift
        && !win
        && matches!(
            key,
            "A" | "C"
                | "F"
                | "N"
                | "P"
                | "Q"
                | "R"
                | "S"
                | "T"
                | "V"
                | "W"
                | "X"
                | "Y"
                | "Z"
                | "Tab"
        )
    {
        return true;
    }
    if win && matches!(key, "D" | "E" | "L" | "R" | "Tab") {
        return true;
    }
    false
}

enum WorkerCmd {
    Apply(
        Option<Shortcut>,
        SyncSender<Result<HotkeyStatus, &'static str>>,
    ),
    SetSuspended(bool, SyncSender<HotkeyStatus>),
    Shutdown,
}

struct HotkeyWorker {
    thread_id: u32,
    queue: Arc<Mutex<VecDeque<WorkerCmd>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl HotkeyWorker {
    fn start(
        initial: Option<Shortcut>,
        hotkey_id: i32,
        on_trigger: Arc<dyn Fn() + Send + Sync>,
    ) -> (Self, HotkeyStatus) {
        let queue = Arc::new(Mutex::new(VecDeque::<WorkerCmd>::new()));
        let worker_queue = Arc::clone(&queue);
        let (ready_tx, ready_rx) = mpsc::sync_channel::<(u32, HotkeyStatus)>(1);

        let handle = thread::spawn(move || {
            let mut msg: MSG = unsafe { std::mem::zeroed() };
            unsafe {
                // Force creation of the Win32 thread message queue before publishing `thread_id`.
                let _ = PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
            }
            let tid = unsafe { GetCurrentThreadId() };

            let mut desired = initial;
            let mut registered = false;
            let mut suspended = false;
            let mut status = match desired {
                None => HotkeyStatus::Disabled,
                Some(sc) => {
                    if bind_os_hotkey(hotkey_id, sc).is_ok() {
                        registered = true;
                        HotkeyStatus::Active
                    } else {
                        HotkeyStatus::Conflict
                    }
                }
            };

            let _ = ready_tx.send((tid, status));

            loop {
                let ret = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
                if ret <= 0 {
                    break;
                }
                if msg.message == WM_HOTKEY && msg.wParam as i32 == hotkey_id && !suspended {
                    on_trigger();
                } else if msg.message == WM_HOTKEY_CMD {
                    let mut exit_loop = false;
                    loop {
                        let cmd = worker_queue
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .pop_front();
                        let Some(cmd) = cmd else {
                            break;
                        };
                        match cmd {
                            WorkerCmd::Apply(next, reply) => {
                                let prev_desired = desired;
                                let prev_was_active =
                                    registered || (suspended && status == HotkeyStatus::Active);
                                if registered {
                                    unbind_os_hotkey(hotkey_id);
                                    registered = false;
                                }
                                suspended = false;
                                match next {
                                    None => {
                                        desired = None;
                                        status = HotkeyStatus::Disabled;
                                        let _ = reply.send(Ok(status));
                                    }
                                    Some(sc) => match bind_os_hotkey(hotkey_id, sc) {
                                        Ok(()) => {
                                            desired = Some(sc);
                                            registered = true;
                                            status = HotkeyStatus::Active;
                                            let _ = reply.send(Ok(status));
                                        }
                                        Err(reason) => {
                                            // Restore previous working shortcut so a failed edit never breaks it.
                                            desired = prev_desired;
                                            if let (true, Some(prev_sc)) =
                                                (prev_was_active, prev_desired)
                                            {
                                                if bind_os_hotkey(hotkey_id, prev_sc).is_ok() {
                                                    registered = true;
                                                    status = HotkeyStatus::Active;
                                                } else {
                                                    status = HotkeyStatus::Conflict;
                                                }
                                            }
                                            let _ = reply.send(Err(reason));
                                        }
                                    },
                                }
                            }
                            WorkerCmd::SetSuspended(pause, reply) => {
                                suspended = pause;
                                if pause {
                                    if registered {
                                        unbind_os_hotkey(hotkey_id);
                                        registered = false;
                                    }
                                } else if !registered {
                                    if let Some(sc) = desired {
                                        if bind_os_hotkey(hotkey_id, sc).is_ok() {
                                            registered = true;
                                            status = HotkeyStatus::Active;
                                        } else {
                                            status = HotkeyStatus::Conflict;
                                        }
                                    } else {
                                        status = HotkeyStatus::Disabled;
                                    }
                                }
                                let _ = reply.send(status);
                            }
                            WorkerCmd::Shutdown => {
                                exit_loop = true;
                                break;
                            }
                        }
                    }
                    if exit_loop {
                        break;
                    }
                }
            }

            if registered {
                unbind_os_hotkey(hotkey_id);
            }
        });

        let (thread_id, initial_status) = ready_rx.recv().unwrap_or((0, HotkeyStatus::Conflict));
        (
            Self {
                thread_id,
                queue,
                join: Mutex::new(Some(handle)),
            },
            initial_status,
        )
    }

    fn post(&self, cmd: WorkerCmd) -> bool {
        if self.thread_id == 0 {
            return false;
        }
        self.queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(cmd);
        unsafe { PostThreadMessageW(self.thread_id, WM_HOTKEY_CMD, 0, 0) != 0 }
    }

    fn apply(&self, next: Option<Shortcut>) -> Result<HotkeyStatus, &'static str> {
        let (tx, rx) = mpsc::sync_channel(1);
        if !self.post(WorkerCmd::Apply(next, tx)) {
            return Err("operation_failed");
        }
        rx.recv().map_err(|_| "operation_failed")?
    }

    fn set_suspended(&self, suspended: bool) -> Option<HotkeyStatus> {
        let (tx, rx) = mpsc::sync_channel(1);
        if !self.post(WorkerCmd::SetSuspended(suspended, tx)) {
            return None;
        }
        rx.recv().ok()
    }

    fn shutdown(&self) {
        let handle = self.join.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(handle) = handle {
            let _ = self.post(WorkerCmd::Shutdown);
            let _ = handle.join();
        }
    }
}

impl Drop for HotkeyWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn bind_os_hotkey(id: i32, shortcut: Shortcut) -> Result<(), &'static str> {
    let ok = unsafe {
        RegisterHotKey(
            null_mut(),
            id,
            shortcut.modifiers | MOD_NOREPEAT,
            shortcut.vk,
        )
    };
    if ok != 0 {
        return Ok(());
    }
    let code = io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or_default() as u32;
    if code == ERROR_HOTKEY_ALREADY_REGISTERED {
        Err("hotkey_conflict")
    } else {
        Err("hotkey_unavailable")
    }
}

fn unbind_os_hotkey(id: i32) {
    unsafe {
        let _ = UnregisterHotKey(null_mut(), id);
    }
}

const LIGHT_TOPBAR_COLORREF: u32 = 0x00F6_ECE8; // RGB(232, 236, 246) -> 0x00BBGGRR
const DARK_TOPBAR_COLORREF: u32 = 0x0023_1813; // RGB(19, 24, 35) -> 0x00BBGGRR

/// Compute an even 1:1 integer physical-pixel square side adapted to the monitor's
/// usable work area and hardware DPI scale factor.
pub fn compute_square_physical_side(work_width: u32, work_height: u32, scale_factor: f64) -> u32 {
    let scale = if scale_factor.is_finite() && scale_factor > 0.25 {
        scale_factor
    } else {
        1.0
    };
    let work_short_phys = work_width.min(work_height).max(360);
    let logical_short = (work_short_phys as f64) / scale;
    let target_logical = (logical_short * 0.55).round().clamp(468.0, 548.0);
    let max_phys = work_short_phys.saturating_sub(40).max(360);
    let raw_phys = ((target_logical * scale).round() as u32).clamp(360, max_phys);
    // Round to an even integer so 1:1 physical dimensions divide cleanly at center.
    raw_phys & !1
}

pub fn adapt_window_to_monitor(window: &tauri::WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };
    let work = monitor.work_area().size;
    let side = compute_square_physical_side(work.width, work.height, monitor.scale_factor());
    let desired = tauri::PhysicalSize::new(side, side);
    if window
        .inner_size()
        .map(|cur| cur.width != side || cur.height != side)
        .unwrap_or(true)
    {
        let _ = window.set_size(tauri::Size::Physical(desired));
    }
}

pub fn configure_borderless_dwm(window: &tauri::WebviewWindow, theme: ThemeMode) {
    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let raw_hwnd = hwnd.0;
    let corner_pref = DWMWCP_ROUND;
    let border_none = DWMWA_COLOR_NONE;
    let is_dark = match theme {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => matches!(window.theme(), Ok(tauri::Theme::Dark)),
    };
    let caption_color = if is_dark {
        DARK_TOPBAR_COLORREF
    } else {
        LIGHT_TOPBAR_COLORREF
    };
    unsafe {
        let _ = DwmSetWindowAttribute(
            raw_hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as _,
            (&raw const corner_pref).cast(),
            std::mem::size_of_val(&corner_pref) as u32,
        );
        let _ = DwmSetWindowAttribute(
            raw_hwnd,
            DWMWA_BORDER_COLOR as _,
            (&raw const border_none).cast(),
            std::mem::size_of_val(&border_none) as u32,
        );
        let _ = DwmSetWindowAttribute(
            raw_hwnd,
            DWMWA_CAPTION_COLOR as _,
            (&raw const caption_color).cast(),
            std::mem::size_of_val(&caption_color) as u32,
        );
    }
}

pub fn apply_window_theme(app: &AppHandle, theme: ThemeMode) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_theme(theme.to_tauri_theme());
        configure_borderless_dwm(&window, theme);
    }
}

fn current_theme_mode(app: &AppHandle) -> ThemeMode {
    app.try_state::<Shell>()
        .map(|s| s.theme_mode())
        .unwrap_or_default()
}

pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let theme = current_theme_mode(app);
        adapt_window_to_monitor(&window);
        configure_borderless_dwm(&window, theme);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn minimize_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

pub fn start_dragging(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.start_dragging();
    }
}

pub fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    if visible && !minimized && focused {
        let _ = window.hide();
    } else {
        let theme = current_theme_mode(app);
        adapt_window_to_monitor(&window);
        configure_borderless_dwm(&window, theme);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn format_show_label(chinese: bool, hotkey: Option<&str>) -> String {
    match (chinese, hotkey) {
        (true, Some(hk)) => format!("显示主窗口 ({hk})"),
        (true, None) => "显示主窗口".to_owned(),
        (false, Some(hk)) => format!("Show window ({hk})"),
        (false, None) => "Show window".to_owned(),
    }
}

fn format_tooltip(chinese: bool, hotkey: Option<&str>) -> String {
    match (chinese, hotkey) {
        (true, Some(hk)) => format!("AG Account Switcher Lite ({hk})"),
        (false, Some(hk)) => format!("AG Account Switcher Lite ({hk})"),
        (_, None) => "AG Account Switcher Lite".to_owned(),
    }
}

pub struct OpGuard<'a>(&'a AtomicUsize);

impl Drop for OpGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct Shell {
    data_root: PathBuf,
    chinese: bool,
    hotkey_worker: HotkeyWorker,
    hotkey_state: Mutex<(Option<String>, HotkeyStatus)>,
    prefs: Mutex<UiPrefs>,
    show_item: MenuItem<Wry>,
    autostart_item: CheckMenuItem<Wry>,
    tray: TrayIcon<Wry>,
    quitting: AtomicBool,
    active_ops: AtomicUsize,
}

impl Shell {
    pub fn init(app: &AppHandle, start_hidden: bool) -> tauri::Result<Self> {
        let data_root = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| env::temp_dir().join("org.ag-account-switcher.lite"));
        let chinese = is_chinese_locale();
        let config = load_shell_config(&data_root);
        let initial_prefs = UiPrefs {
            layout: config.layout,
            theme: config.theme,
            particles_follow: config.particles_follow,
        };
        let parsed_shortcut = config
            .hotkey
            .as_deref()
            .and_then(|s| Shortcut::parse(s).ok());
        let initial_label = parsed_shortcut.map(Shortcut::canonical);

        let trigger_app = app.clone();
        let (hotkey_worker, initial_status) = HotkeyWorker::start(
            parsed_shortcut,
            PRODUCTION_HOTKEY_ID,
            Arc::new(move || toggle_window(&trigger_app)),
        );

        let current_exe = env::current_exe().ok();
        let (autostart_enabled, _) = inspect_autostart_in(
            RUN_SUBKEY,
            APPROVED_SUBKEY,
            RUN_VALUE_NAME,
            current_exe.as_deref(),
        );

        let show_item = MenuItem::with_id(
            app,
            "show",
            format_show_label(chinese, initial_label.as_deref()),
            true,
            None::<&str>,
        )?;
        let autostart_item = CheckMenuItem::with_id(
            app,
            "autostart",
            if chinese {
                "开机自启"
            } else {
                "Launch at startup"
            },
            true,
            autostart_enabled,
            None::<&str>,
        )?;
        let separator = PredefinedMenuItem::separator(app)?;
        let quit_item = MenuItem::with_id(
            app,
            "quit",
            if chinese { "退出" } else { "Quit" },
            true,
            None::<&str>,
        )?;

        let menu = Menu::with_items(app, &[&show_item, &autostart_item, &separator, &quit_item])?;

        let mut builder = TrayIconBuilder::with_id("main-tray")
            .menu(&menu)
            .tooltip(format_tooltip(chinese, initial_label.as_deref()))
            .show_menu_on_left_click(false)
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    toggle_window(tray.app_handle());
                }
            })
            .on_menu_event(|app, event| match event.id().as_ref() {
                "show" => show_window(app),
                "autostart" => {
                    if let Some(shell) = app.try_state::<Shell>() {
                        let (currently_enabled, _) = shell.inspect_autostart();
                        let _ = shell.set_autostart(!currently_enabled);
                    }
                }
                "quit" => {
                    if let Some(shell) = app.try_state::<Shell>() {
                        if shell.has_active_ops() {
                            show_window(app);
                            return;
                        }
                        shell.prepare_quit();
                    }
                    app.exit(0);
                }
                _ => {}
            });

        if let Some(icon) = app.default_window_icon().cloned() {
            builder = builder.icon(icon);
        }
        let tray = builder.build(app)?;

        apply_window_theme(app, initial_prefs.theme);
        if let Some(window) = app.get_webview_window("main") {
            adapt_window_to_monitor(&window);
            let _ = window.center();
        }
        if !start_hidden {
            show_window(app);
        }

        Ok(Self {
            data_root,
            chinese,
            hotkey_worker,
            hotkey_state: Mutex::new((initial_label, initial_status)),
            prefs: Mutex::new(initial_prefs),
            show_item,
            autostart_item,
            tray,
            quitting: AtomicBool::new(false),
            active_ops: AtomicUsize::new(0),
        })
    }

    pub fn track_operation(&self) -> OpGuard<'_> {
        self.active_ops.fetch_add(1, Ordering::SeqCst);
        OpGuard(&self.active_ops)
    }

    pub fn has_active_ops(&self) -> bool {
        self.active_ops.load(Ordering::SeqCst) > 0
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.prefs.lock().unwrap_or_else(|e| e.into_inner()).theme
    }

    fn prepare_quit(&self) {
        self.quitting.store(true, Ordering::SeqCst);
        self.hotkey_worker.shutdown();
    }

    fn inspect_autostart(&self) -> (bool, bool) {
        let exe = env::current_exe().ok();
        inspect_autostart_in(RUN_SUBKEY, APPROVED_SUBKEY, RUN_VALUE_NAME, exe.as_deref())
    }

    pub fn status(&self) -> ShellStatus {
        let (autostart, autostart_blocked) = self.inspect_autostart();
        let _ = self.autostart_item.set_checked(autostart);
        let (hotkey, status) = self
            .hotkey_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let prefs = *self.prefs.lock().unwrap_or_else(|e| e.into_inner());
        ShellStatus {
            autostart,
            autostart_blocked,
            hotkey,
            hotkey_status: status.as_str(),
            layout: prefs.layout.as_str(),
            theme: prefs.theme.as_str(),
            particles_follow: prefs.particles_follow,
        }
    }

    pub fn set_autostart(&self, enabled: bool) -> Result<ShellStatus, &'static str> {
        let exe = env::current_exe().map_err(|_| "autostart_unavailable")?;
        let result = set_autostart_in(RUN_SUBKEY, RUN_VALUE_NAME, &exe, enabled);
        let current = self.status();
        result.map(|()| current)
    }

    pub fn set_hotkey(&self, raw: Option<String>) -> Result<ShellStatus, &'static str> {
        let parsed = match raw.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(text) => match Shortcut::parse(text) {
                Ok(sc) => Some(sc),
                Err(err) => {
                    self.set_hotkey_suspended(false);
                    return Err(err);
                }
            },
        };
        let next_status = self.hotkey_worker.apply(parsed)?;
        let canonical = parsed.map(Shortcut::canonical);
        let prefs = *self.prefs.lock().unwrap_or_else(|e| e.into_inner());
        save_shell_config(&self.data_root, canonical.as_deref(), prefs)?;

        {
            let mut guard = self.hotkey_state.lock().unwrap_or_else(|e| e.into_inner());
            *guard = (canonical.clone(), next_status);
        }
        let _ = self
            .show_item
            .set_text(format_show_label(self.chinese, canonical.as_deref()));
        let _ = self
            .tray
            .set_tooltip(Some(format_tooltip(self.chinese, canonical.as_deref())));

        Ok(self.status())
    }

    pub fn set_preferences(
        &self,
        app: &AppHandle,
        layout: LayoutMode,
        theme: ThemeMode,
        particles_follow: bool,
    ) -> Result<ShellStatus, &'static str> {
        let next_prefs = UiPrefs {
            layout,
            theme,
            particles_follow,
        };
        let hotkey = self
            .hotkey_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .0
            .clone();
        save_shell_config(&self.data_root, hotkey.as_deref(), next_prefs)?;
        {
            let mut guard = self.prefs.lock().unwrap_or_else(|e| e.into_inner());
            *guard = next_prefs;
        }
        apply_window_theme(app, theme);
        Ok(self.status())
    }

    pub fn set_hotkey_suspended(&self, suspended: bool) {
        if let Some(updated_status) = self.hotkey_worker.set_suspended(suspended) {
            if !suspended {
                let mut guard = self.hotkey_state.lock().unwrap_or_else(|e| e.into_inner());
                guard.1 = updated_status;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows_sys::Win32::System::Registry::RegDeleteTreeW;

    struct RegFixture {
        root: String,
        run_key: String,
        approved_key: String,
    }

    impl RegFixture {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root = format!(
                r"Software\AGAccountSwitcherLiteTests\{}-{stamp}-{id}",
                std::process::id()
            );
            Self {
                run_key: format!(r"{root}\Run"),
                approved_key: format!(r"{root}\StartupApproved"),
                root,
            }
        }

        fn write_approved_bytes(&self, value_name: &str, bytes: &[u8]) {
            let subkey_w = wide(&self.approved_key);
            let name_w = wide(value_name);
            let mut raw_key: HKEY = null_mut();
            assert_eq!(
                unsafe { RegCreateKeyW(HKEY_CURRENT_USER, subkey_w.as_ptr(), &mut raw_key) },
                ERROR_SUCCESS
            );
            let key = OwnedRegKey(raw_key);
            assert_eq!(
                unsafe {
                    RegSetValueExW(
                        key.0,
                        name_w.as_ptr(),
                        0,
                        REG_BINARY,
                        bytes.as_ptr(),
                        bytes.len() as u32,
                    )
                },
                ERROR_SUCCESS
            );
        }
    }

    impl Drop for RegFixture {
        fn drop(&mut self) {
            let root_w = wide(&self.root);
            unsafe {
                let _ = RegDeleteTreeW(HKEY_CURRENT_USER, root_w.as_ptr());
            }
        }
    }

    #[test]
    fn autostart_quotes_spaced_paths_heals_drift_and_detects_system_block() {
        let fixture = RegFixture::new();
        let old_exe = Path::new(r"C:\Portable Tools\Old Dir\ag-account-switcher-lite.exe");
        let new_exe = Path::new(r"D:\Apps\AG Switcher\ag-account-switcher-lite.exe");

        assert_eq!(
            inspect_autostart_in(
                &fixture.run_key,
                &fixture.approved_key,
                RUN_VALUE_NAME,
                Some(old_exe)
            ),
            (false, false)
        );

        // Idempotent disable when absent.
        set_autostart_in(&fixture.run_key, RUN_VALUE_NAME, old_exe, false).unwrap();

        // Enable writes quoted path + `--autostart`.
        set_autostart_in(&fixture.run_key, RUN_VALUE_NAME, old_exe, true).unwrap();
        assert_eq!(
            read_reg_sz(&fixture.run_key, RUN_VALUE_NAME).as_deref(),
            Some(r#""C:\Portable Tools\Old Dir\ag-account-switcher-lite.exe" --autostart"#)
        );

        // Moving the portable binary self-heals the Run command on inspection.
        assert_eq!(
            inspect_autostart_in(
                &fixture.run_key,
                &fixture.approved_key,
                RUN_VALUE_NAME,
                Some(new_exe)
            ),
            (true, false)
        );
        assert_eq!(
            read_reg_sz(&fixture.run_key, RUN_VALUE_NAME).as_deref(),
            Some(r#""D:\Apps\AG Switcher\ag-account-switcher-lite.exe" --autostart"#)
        );

        // Simulate Windows Startup Apps enabled (0x02) vs disabled (0x03) flag bytes.
        fixture.write_approved_bytes(RUN_VALUE_NAME, &[0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            inspect_autostart_in(
                &fixture.run_key,
                &fixture.approved_key,
                RUN_VALUE_NAME,
                Some(new_exe)
            ),
            (true, false)
        );
        fixture.write_approved_bytes(RUN_VALUE_NAME, &[0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            inspect_autostart_in(
                &fixture.run_key,
                &fixture.approved_key,
                RUN_VALUE_NAME,
                Some(new_exe)
            ),
            (true, true)
        );

        // Disabling cleanly deletes the Run value.
        set_autostart_in(&fixture.run_key, RUN_VALUE_NAME, new_exe, false).unwrap();
        assert_eq!(read_reg_sz(&fixture.run_key, RUN_VALUE_NAME), None);
    }

    #[test]
    fn shortcut_parser_normalizes_valid_combos_and_rejects_unsafe_ones() {
        let parsed = Shortcut::parse(" shift + alt + g ").unwrap();
        assert_eq!(parsed.canonical(), "Alt+Shift+G");

        let complex = Shortcut::parse("win+ctrl+Backquote").unwrap();
        assert_eq!(complex.canonical(), "Ctrl+Win+`");

        let fn_key = Shortcut::parse("Ctrl+Alt+f12").unwrap();
        assert_eq!(fn_key.canonical(), "Ctrl+Alt+F12");

        for invalid in [
            "",
            "G",
            "Shift+G",
            "Ctrl+",
            "Ctrl++G",
            "Alt+Alt+G",
            "Alt+F4",
            "Alt+Tab",
            "Alt+Space",
            "Ctrl+C",
            "Ctrl+V",
            "Ctrl+S",
            "Win+L",
            "Win+R",
            "Ctrl+Alt+Delete",
        ] {
            assert_eq!(
                Shortcut::parse(invalid),
                Err("invalid_hotkey"),
                "expected `{invalid}` to be rejected"
            );
        }
    }

    #[test]
    fn shell_config_round_trips_and_recovers_from_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let initial = load_shell_config(temp.path());
        assert_eq!(initial.hotkey.as_deref(), Some(DEFAULT_HOTKEY));
        assert_eq!(initial.layout, LayoutMode::Matrix);
        assert_eq!(initial.theme, ThemeMode::System);
        assert!(initial.particles_follow);

        let custom_prefs = UiPrefs {
            layout: LayoutMode::List,
            theme: ThemeMode::Dark,
            particles_follow: false,
        };
        save_shell_config(temp.path(), Some("Ctrl+Alt+S"), custom_prefs).unwrap();
        let loaded = load_shell_config(temp.path());
        assert_eq!(loaded.hotkey.as_deref(), Some("Ctrl+Alt+S"));
        assert_eq!(loaded.layout, LayoutMode::List);
        assert_eq!(loaded.theme, ThemeMode::Dark);
        assert!(!loaded.particles_follow);

        save_shell_config(temp.path(), None, UiPrefs::default()).unwrap();
        let disabled = load_shell_config(temp.path());
        assert_eq!(disabled.hotkey, None);
        assert_eq!(disabled.layout, LayoutMode::Matrix);
        assert_eq!(disabled.theme, ThemeMode::System);
        assert!(disabled.particles_follow);

        // Legacy v0.1.x config with only `hotkey` preserves the shortcut and applies UI defaults.
        fs::write(
            temp.path().join(SHELL_CONFIG_FILE),
            r#"{"hotkey":"Ctrl+Shift+A"}"#,
        )
        .unwrap();
        let legacy = load_shell_config(temp.path());
        assert_eq!(legacy.hotkey.as_deref(), Some("Ctrl+Shift+A"));
        assert_eq!(legacy.layout, LayoutMode::Matrix);
        assert_eq!(legacy.theme, ThemeMode::System);
        assert!(legacy.particles_follow);

        fs::write(temp.path().join(SHELL_CONFIG_FILE), "{broken-json").unwrap();
        assert_eq!(
            load_shell_config(temp.path()).hotkey.as_deref(),
            Some(DEFAULT_HOTKEY)
        );

        fs::write(
            temp.path().join(SHELL_CONFIG_FILE),
            r#"{"hotkey":"Ctrl+C","layout":"unknown","theme":"neon"}"#,
        )
        .unwrap();
        let sanitized = load_shell_config(temp.path());
        assert_eq!(sanitized.hotkey.as_deref(), Some(DEFAULT_HOTKEY));
        assert_eq!(sanitized.layout, LayoutMode::Matrix);
        assert_eq!(sanitized.theme, ThemeMode::System);

        assert_eq!(LayoutMode::parse(" LIST "), Ok(LayoutMode::List));
        assert_eq!(LayoutMode::parse("matrix"), Ok(LayoutMode::Matrix));
        assert_eq!(LayoutMode::parse("grid"), Err("invalid_preferences"));
        assert_eq!(ThemeMode::parse("Light"), Ok(ThemeMode::Light));
        assert_eq!(ThemeMode::parse("dark"), Ok(ThemeMode::Dark));
        assert_eq!(ThemeMode::parse("system"), Ok(ThemeMode::System));
        assert_eq!(ThemeMode::parse("sepia"), Err("invalid_preferences"));

        // 1:1 square monitor & DPI adaptation invariants across 1080p, 1440p, and 4K.
        for (w, h, scale) in [
            (1920, 1032, 1.0),
            (1920, 1032, 1.25),
            (1920, 1008, 1.5),
            (2560, 1380, 1.5),
            (3840, 2100, 1.5),
            (3840, 2100, 2.0),
            (1366, 720, 1.0),
        ] {
            let side = compute_square_physical_side(w, h, scale);
            assert_eq!(side % 2, 0, "physical side must be an even integer");
            assert!(
                side <= w.min(h),
                "square side {side} must fit inside {w}x{h}"
            );
        }
        assert_eq!(compute_square_physical_side(2560, 1380, 1.5), 758);
    }

    #[test]
    fn hotkey_worker_handles_conflict_suspension_rollback_and_clean_release() {
        let candidates = [
            "Ctrl+Alt+Shift+F12",
            "Ctrl+Alt+Shift+F11",
            "Ctrl+Alt+Shift+F10",
            "Ctrl+Alt+Shift+9",
        ];
        let mut chosen = None;
        let mut holder = None;
        for text in candidates {
            let sc = Shortcut::parse(text).unwrap();
            let (w, status) = HotkeyWorker::start(Some(sc), 701, Arc::new(|| {}));
            if status == HotkeyStatus::Active {
                chosen = Some(sc);
                holder = Some(w);
                break;
            }
        }
        let Some(sc) = chosen else {
            // Headless desktop without hotkey availability; skip conflict assertion gracefully.
            return;
        };
        let primary = holder.unwrap();

        // A second worker attempting the exact same OS hotkey must report `Conflict` (1409)
        // without panicking, and `apply` onto a conflicting key must preserve its own previous key.
        let (secondary, sec_status) = HotkeyWorker::start(Some(sc), 702, Arc::new(|| {}));
        assert_eq!(sec_status, HotkeyStatus::Conflict);
        drop(secondary);

        // Temporarily suspending `primary` releases the OS hotkey so another handle can claim it,
        // and dropping that handle lets `primary` resume cleanly.
        assert_eq!(primary.set_suspended(true), Some(HotkeyStatus::Active));
        let (probe, probe_status) = HotkeyWorker::start(Some(sc), 703, Arc::new(|| {}));
        assert_eq!(probe_status, HotkeyStatus::Active);

        // If `probe` still holds `sc` while a third worker tries to switch from None -> `sc`,
        // `apply` returns `Err("hotkey_conflict")`.
        let (third, third_status) = HotkeyWorker::start(None, 704, Arc::new(|| {}));
        assert_eq!(third_status, HotkeyStatus::Disabled);
        assert_eq!(third.apply(Some(sc)), Err("hotkey_conflict"));
        drop(third);
        drop(probe);

        assert_eq!(primary.set_suspended(false), Some(HotkeyStatus::Active));
        assert_eq!(primary.apply(None), Ok(HotkeyStatus::Disabled));
    }
}

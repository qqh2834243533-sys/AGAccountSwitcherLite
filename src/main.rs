#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod service;
mod shell;
mod switch_lock;
mod vault;

use serde::Serialize;
use std::{env, fs, path::Path};
use tauri::{AppHandle, Manager, WindowEvent};
use vault::Vault;

type Reply<T> = Result<T, String>;

async fn with_vault<T: Send + 'static>(
    app: AppHandle,
    track_mutation: bool,
    action: impl FnOnce(&Vault<'_>, &Path) -> Result<T, &'static str> + Send + 'static,
) -> Reply<T> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "data_folder_unavailable")?;
    tauri::async_runtime::spawn_blocking(move || {
        let _op = track_mutation
            .then(|| app.try_state::<shell::Shell>())
            .flatten()
            .map(|state| state.inner().track_operation());
        fs::create_dir_all(&root).map_err(|_| "data_folder_unavailable")?;
        let _guard = switch_lock::SwitchLock::acquire(&root).map_err(|reason| {
            if reason == "another_account_switch_in_progress" {
                "switch_in_progress"
            } else {
                "lock_unavailable"
            }
        })?;
        action(&Vault::production(), &root).map_err(String::from)
    })
    .await
    .map_err(|_| "operation_failed")?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Overview {
    app_installed: bool,
    client_running: bool,
    #[serde(flatten)]
    accounts: service::Status,
    #[serde(flatten)]
    shell: shell::ShellStatus,
}

#[tauri::command]
async fn overview(app: AppHandle) -> Reply<Overview> {
    let shell_status = app
        .try_state::<shell::Shell>()
        .map(|s| s.status())
        .ok_or_else(|| String::from("operation_failed"))?;
    with_vault(app, false, move |vault, root| {
        let installed = client::installed();
        if let Some(executable) = installed.as_deref() {
            client::observe_running_launch_plan(executable, root);
        }
        Ok(Overview {
            app_installed: installed.is_some(),
            client_running: client::require_closed().is_err(),
            accounts: service::status(vault)?,
            shell: shell_status,
        })
    })
    .await
}

#[tauri::command]
async fn register_account(app: AppHandle, label: String) -> Reply<()> {
    with_vault(app, true, move |vault, _root| {
        client::installed().ok_or("app_missing")?;
        service::register(vault, label)
    })
    .await
}

#[tauri::command]
async fn remove_account(app: AppHandle, slot: u8) -> Reply<()> {
    with_vault(app, true, move |vault, _root| service::remove(vault, slot)).await
}

#[tauri::command]
async fn switch_account(app: AppHandle, slot: u8) -> Reply<()> {
    with_vault(app, true, move |vault, root| {
        let executable = client::installed().ok_or("app_missing")?;
        service::switch(
            vault,
            slot,
            || client::prepare_switch(&executable, root),
            client::require_closed,
            |plan| client::open(&executable, &plan),
        )
    })
    .await
}

#[tauri::command]
async fn restore_session(app: AppHandle) -> Reply<()> {
    with_vault(app, true, |vault, _root| {
        service::restore(vault, client::require_closed)
    })
    .await
}

#[tauri::command]
async fn discard_recovery(app: AppHandle) -> Reply<()> {
    with_vault(app, true, |vault, _root| {
        service::discard_recovery(vault, client::require_closed)
    })
    .await
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Reply<shell::ShellStatus> {
    let state = app.try_state::<shell::Shell>().ok_or("operation_failed")?;
    state.set_autostart(enabled).map_err(String::from)
}

#[tauri::command]
fn set_hotkey(app: AppHandle, hotkey: Option<String>) -> Reply<shell::ShellStatus> {
    let state = app.try_state::<shell::Shell>().ok_or("operation_failed")?;
    state.set_hotkey(hotkey).map_err(String::from)
}

#[tauri::command]
fn set_hotkey_suspended(app: AppHandle, suspended: bool) {
    if let Some(state) = app.try_state::<shell::Shell>() {
        state.set_hotkey_suspended(suspended);
    }
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    shell::hide_window(&app);
}

fn has_autostart_flag<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|arg| arg.as_ref().eq_ignore_ascii_case(shell::AUTOSTART_ARG))
}

fn main() {
    let start_hidden = env::args_os()
        .skip(1)
        .any(|arg| arg.eq_ignore_ascii_case(shell::AUTOSTART_ARG));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _| {
            if !has_autostart_flag(argv.iter().skip(1)) {
                shell::show_window(app);
            }
        }))
        .setup(move |app| {
            let state = shell::Shell::init(app.handle(), start_hidden)?;
            app.manage(state);
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let keep_in_tray = window
                        .app_handle()
                        .try_state::<shell::Shell>()
                        .is_some_and(|state| !state.is_quitting());
                    if keep_in_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            overview,
            register_account,
            remove_account,
            switch_account,
            restore_session,
            discard_recovery,
            set_autostart,
            set_hotkey,
            set_hotkey_suspended,
            hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start AG Account Switcher Lite");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_autostart_flag_case_insensitively() {
        assert!(has_autostart_flag(["--autostart"]));
        assert!(has_autostart_flag(["--foo", "--AutoStart"]));
        assert!(!has_autostart_flag(["--minimized"]));
        assert!(!has_autostart_flag(std::iter::empty::<&str>()));
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod service;
mod switch_lock;
mod vault;

use serde::Serialize;
use std::{fs, path::Path};
use tauri::{AppHandle, Manager};
use vault::Vault;

type Reply<T> = Result<T, String>;

async fn with_vault<T: Send + 'static>(
    app: AppHandle,
    action: impl FnOnce(&Vault<'_>, &Path) -> Result<T, &'static str> + Send + 'static,
) -> Reply<T> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "data_folder_unavailable")?;
    tauri::async_runtime::spawn_blocking(move || {
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
}

#[tauri::command]
async fn overview(app: AppHandle) -> Reply<Overview> {
    with_vault(app, |vault, root| {
        let installed = client::installed();
        if let Some(executable) = installed.as_deref() {
            client::observe_running_launch_plan(executable, root);
        }
        Ok(Overview {
            app_installed: installed.is_some(),
            client_running: client::require_closed().is_err(),
            accounts: service::status(vault)?,
        })
    })
    .await
}

#[tauri::command]
async fn register_account(app: AppHandle, label: String) -> Reply<()> {
    with_vault(app, move |vault, _root| {
        client::installed().ok_or("app_missing")?;
        service::register(vault, label)
    })
    .await
}

#[tauri::command]
async fn remove_account(app: AppHandle, slot: u8) -> Reply<()> {
    with_vault(app, move |vault, _root| service::remove(vault, slot)).await
}

#[tauri::command]
async fn switch_account(app: AppHandle, slot: u8) -> Reply<()> {
    with_vault(app, move |vault, root| {
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
    with_vault(app, |vault, _root| {
        service::restore(vault, client::require_closed)
    })
    .await
}

#[tauri::command]
async fn discard_recovery(app: AppHandle) -> Reply<()> {
    with_vault(app, |vault, _root| {
        service::discard_recovery(vault, client::require_closed)
    })
    .await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            overview,
            register_account,
            remove_account,
            switch_account,
            restore_session,
            discard_recovery,
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start AG Account Switcher Lite");
}

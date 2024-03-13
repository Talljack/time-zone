// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;
mod windows;
mod utils;

use parking_lot::Mutex;
// use serde_json::json;
use once_cell::sync::OnceCell;
use tauri::AppHandle;

pub static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    version: String,
    current_version: String,
    body: Option<String>,
}

pub static UPDATE_RESULT: Mutex<Option<Option<UpdateResult>>> = Mutex::new(None);


#[tauri::command]
fn get_update_result() -> (bool, Option<UpdateResult>) {
    if UPDATE_RESULT.lock().is_none() {
        return (false, None);
    }
    return (true, UPDATE_RESULT.lock().clone().unwrap());
}


fn main() {
    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.handle();
            APP_HANDLE.get_or_init(|| app.handle().clone());
            tray::create_tray(&app_handle)?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_update_result])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

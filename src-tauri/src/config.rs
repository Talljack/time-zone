use parking_lot::Mutex;
use tauri::Manager;
use tauri::{path::BaseDirectory, AppHandle};

use crate::APP_HANDLE;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub time_zone: Option<String>,
}

static CONFIG_CACHE: Mutex<Option<Config>> = Mutex::new(None);

pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    let app_handle = APP_HANDLE.get().unwrap();
    get_config_from_file(app_handle)
}

pub fn get_config_from_file(app_handle: &AppHandle) -> Result<Config, Box<dyn std::error::Error>> {
    let conf = _get_config_by_app(&app_handle);
    match conf {
        Ok(conf) => Ok(conf),
        Err(e) => {
            println!("get config failed: {}", e);
            Err(e)
        }
    }
}

pub fn _get_config_by_app(app_handle: &AppHandle) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(config_cache) = &*CONFIG_CACHE.lock() {
        return Ok(config_cache.clone());
    }
    let config_content = get_config_content_by_app(app_handle)?;
    let config: Config = serde_json::from_str(&config_content)?;
    *CONFIG_CACHE.lock() = Some(config.clone());
    Ok(config)
}

pub fn get_config_content_by_app(app_handle: &AppHandle) -> Result<String, String> {
    let app_config_dir = app_handle
        .path()
        .resolve("com.time-zone.dev", BaseDirectory::Config)
        .unwrap();
    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir).unwrap();
    }
    let app_path = app_config_dir.join("config.json");
    if app_path.exists() {
        match std::fs::read_to_string(app_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        }
    } else {
        std::fs::write(app_path, "{}").unwrap();
        Ok("{}".to_string())
    }
}

pub fn set_config_content<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    content: Config,
) -> Result<String, String> {
    let app_config_dir = app_handle
        .path()
        .resolve("com.time-zone.dev", BaseDirectory::Config)
        .unwrap();
    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir).unwrap();
    }
    *CONFIG_CACHE.lock() = Some(content.clone());
    let content_str = serde_json::to_string(&content).unwrap();
    let app_path = app_config_dir.join("config.json");
    if app_path.exists() {
        std::fs::write(app_path, content_str).unwrap();
    } else {
        std::fs::write(app_path, "{}").unwrap();
    }
    Ok("set config success".to_string())
}

#[tauri::command]
pub fn clear_config_cache() {
    CONFIG_CACHE.lock().take();
}

#[tauri::command]
pub fn get_config_content() -> Result<String, String> {
    if let Some(app) = APP_HANDLE.get() {
        get_config_content_by_app(app)
    } else {
        Err("app handle not found".to_string())
    }
}

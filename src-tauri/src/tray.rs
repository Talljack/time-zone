use tauri::{
    menu::{Menu, MenuItem, Submenu},
    tray::{ClickType, TrayIcon},
    Runtime,
};
use tauri::Manager;
use crate::UPDATE_RESULT;
use chrono::{DateTime, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use crate::windows::{
    show_updater_window,
    SETTINGS_WIN_NAME
};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use crate::config::{get_config, set_config_content, Config};

// const tray_time_zone: &str = "America/New_York";

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    // let config = get_config().unwrap();
    let check_for_updates_i = MenuItem::with_id(
        app,
        "check_for_updates",
        "Check for Updates...",
        true,
        None::<String>,
    )?;
    if let Some(Some(_)) = *UPDATE_RESULT.lock() {
        check_for_updates_i
            .set_text("💡 New version available!")
            .unwrap();
    }
    let sub_setting_i = Submenu::with_items(app, "Settings", true, &[])?;
    for item in TZ_VARIANTS {
        let sub_item = MenuItem::with_id(app, item, item.to_string(), true, None::<String>)?;
        let _ = sub_setting_i.append(&sub_item);
    }
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<String>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<String>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;
    let menu = Menu::with_items(
        app,
        &[
            &check_for_updates_i,
            &sub_setting_i,
            &show_i,
            &hide_i,
            &quit_i,
        ],
    )?;

    let tray = app.tray().expect("Failed to create tray icon");
    tray.set_menu(Some(menu.clone()))?;
    tray.on_menu_event(move|app, event| {
        // println!("event: {}", event.id.as_ref());
        let id_str = event.id.as_ref();
        println!("id_str: {}", id_str);
        if id_str == "check_for_updates" {
            show_updater_window();
        } else if id_str == "show" {
            let window = app.get_webview_window(SETTINGS_WIN_NAME).unwrap();
            window.set_focus().unwrap();
            window.unminimize().unwrap();
            window.show().unwrap();
        } else if id_str == "hide" {
            let window = app.get_webview_window(SETTINGS_WIN_NAME).unwrap();
            window.set_focus().unwrap();
            window.unminimize().unwrap();
            window.hide().unwrap();
        } else if id_str == "quit" {
            app.exit(0);
        } else if TZ_VARIANTS.contains(&Tz::from_str(&id_str).unwrap()) {
            let old_config = match get_config() {
                Ok(config) => config,
                Err(e) => {
                    println!("failed to get config: {}", e);
                    Config {
                        time_zone: None
                    }
                }
            };
            let old_time_zone = match old_config.time_zone {
                Some(time_zone) => time_zone,
                None => "".to_string()
            };
            if old_time_zone == id_str {
                return;
            }

            let new_config = Config {
                time_zone: Some(id_str.to_string()),
            };
            let now = Utc::now();
            let time_zone = Tz::from_str(&id_str).unwrap();
            let now = now.with_timezone(&time_zone);
            let format_time = now.format("%m-%d %H:%M");
            let _ = app.tray().unwrap().set_title(Some(format_time.to_string()));
            if let Err(e) = set_config_content(app, new_config) {
                println!("set config failed: {}", e);
            }
        }
    });
    tray.on_tray_icon_event(|tray, event| {
        if event.click_type == ClickType::Left {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window(SETTINGS_WIN_NAME) {
                window.unminimize().unwrap();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    });
    let tray = TrayIcon::clone(&tray);
    let shared_tray = Arc::new(Mutex::new(tray.clone()));
    tray_update::<R>(shared_tray);
    let _ = tray.set_icon_as_template(false);
    tray.set_show_menu_on_left_click(false)?;

    Ok(())
}

fn update_time_zone<R: Runtime>(tray: Arc<Mutex<TrayIcon<R>>>) {
    // Get the current time
    let now = Utc::now();
    let config = get_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            println!("failed to get config: {}", e);
            Config {
                time_zone: None
            }
        }
    };
    let time_zone = match config.time_zone {
        Some(time_zone) => time_zone,
        None => "".to_string()
    };
    let time_zone = Tz::from_str(&time_zone).unwrap();
    let now = now.with_timezone(&time_zone);
    // let new_york_time: DateTime<Tz> = now.with_timezone(&Tz::America__New_York);
    let format_time = now.format("%m-%d %H:%M");

    // Lock the tray icon and update the title
    let locked_tray = tray.lock().unwrap();
    let _ = locked_tray.set_title(Some(format_time.to_string()));
}

fn tray_update<R: Runtime>(tray: Arc<Mutex<TrayIcon<R>>>) {
    thread::spawn(move || {
        loop {
            update_time_zone(tray.clone());
            // Sleep for 2 seconds
            thread::sleep(Duration::from_secs(2));
        }
    });
}
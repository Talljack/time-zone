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
    show_updater_window, get_settings_window,
    SETTINGS_WIN_NAME
};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

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
    // tray.on_menu_event(move |app, event| match event.id.as_ref() {
    //     "check_for_updates" => {
    //         show_updater_window();
    //     }
    //     // TZ_VARIANTS[0] => {
    //     //     // get_settings_window();
    //     //     println!("click")
    //     // }
    //     TZ_VARIANTS => {

    //     }
    //     "show" => {
    //         let window = app.get_webview_window(SETTINGS_WIN_NAME).unwrap();
    //         window.set_focus().unwrap();
    //         window.unminimize().unwrap();
    //         window.show().unwrap();
    //     }
    //     "hide" => {
    //         let window = app.get_webview_window(SETTINGS_WIN_NAME).unwrap();
    //         window.set_focus().unwrap();
    //         window.unminimize().unwrap();
    //         window.hide().unwrap();
    //     }
    //     "quit" => app.exit(0),
    //     _ => {}
    // });
    tray.on_menu_event(move|app, event| {
        // println!("event: {}", event.id.as_ref());
        let id_str = event.id.as_ref();
        let id_tz = Tz::from_str(&id_str).unwrap();
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
        } else if TZ_VARIANTS.contains(&id_tz) {
            println!("TZ: {}", id_str);
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

fn tray_update<R: Runtime>(tray: Arc<Mutex<TrayIcon<R>>>) {
    thread::spawn(move || {
        loop {
            // Get the current time
            let now = Utc::now();
            let new_york_time: DateTime<Tz> = now.with_timezone(&Tz::America__New_York);
            let format_time = new_york_time.format("%m-%d %H:%M");

            // Lock the tray icon and update the title
            let locked_tray = tray.lock().unwrap();
            let _ = locked_tray.set_title(Some(format_time.to_string()));

            // Sleep for 60 seconds
            thread::sleep(Duration::from_secs(60));
        }
    });
}
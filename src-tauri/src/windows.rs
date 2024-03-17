use crate::{UpdateResult, APP_HANDLE};
use serde_json::json;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSWindow;

pub const SETTINGS_WIN_NAME: &str = "setting_win_name";
pub const UPDATER_WIN_NAME: &str = "updater";

pub fn post_process_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    window.set_visible_on_all_workspaces(true).unwrap();
    let _ = window.current_monitor();
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSWindowCollectionBehavior;
        use cocoa::base::id;
        unsafe {
            let ns_window = window.ns_window().unwrap() as cocoa::base::id;
            NSWindow::setAllowsAutomaticWindowTabbing_(ns_window, cocoa::base::NO);
        }
        let ns_win = window.ns_window().unwrap() as id;
        unsafe {
            let mut collection_behavior = ns_win.collectionBehavior();
            collection_behavior |=
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces;

            ns_win.setCollectionBehavior_(collection_behavior);
        }
    }
}

pub fn build_window<'a, R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: tauri::WebviewWindowBuilder<'a, R, M>,
) -> tauri::WebviewWindow<R> {
    #[cfg(target_os = "macos")]
    {
        let window = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
            .build()
            .unwrap();
        post_process_window(&window);
        window
    }
    #[cfg(not(target_os = "macos"))]
    {
        let window = builder.transparent(true).decorations(true).build().unwrap();
        post_process_window(&window);
        window
    }
}

pub fn get_settings_window() -> tauri::WebviewWindow {
    let handle = APP_HANDLE.get().expect("cannot find app handle");
    let window = match handle.get_webview_window(SETTINGS_WIN_NAME) {
        Some(window) => {
            window.unminimize().unwrap();
            window.set_focus().unwrap();
            window
        }
        None => {
            let builder = tauri::WebviewWindowBuilder::new(
                handle,
                SETTINGS_WIN_NAME,
                tauri::WebviewUrl::App("src/index.html".into()),
            )
            .title("OpenAI Translator Settings")
            .fullscreen(false)
            .inner_size(660.0, 800.0)
            .min_inner_size(660.0, 600.0)
            .resizable(true)
            .skip_taskbar(true)
            .focused(true);

            return build_window(builder);
        }
    };

    window
}

pub fn show_updater_window() {
    let window = get_updater_window();
    window.center().unwrap();
    window.show().unwrap();
    let window_clone = window.clone();
    window.listen("check_update", move |event| {
        let handle = APP_HANDLE.get().unwrap();
        let window_clone = window_clone.clone();
        tauri::async_runtime::spawn(async move {
            let builder = handle.updater_builder();
            let updater = builder.build().unwrap();

            match updater.check().await {
                Ok(Some(update)) => {
                    handle
                        .emit(
                            "update_result",
                            json!({
                                "result": UpdateResult {
                                    version: update.version,
                                    current_version: update.current_version,
                                    body: update.body,
                                }
                            }),
                        )
                        .unwrap();
                }
                Ok(None) => {
                    handle
                        .emit(
                            "update_result",
                            json!({
                                "result": None::<UpdateResult>
                            }),
                        )
                        .unwrap();
                }
                Err(_) => {}
            }
            window_clone.unlisten(event.id())
        });
    });
}

pub fn get_updater_window() -> tauri::WebviewWindow {
    let handle = APP_HANDLE.get().unwrap();
    let window = match handle.get_webview_window(UPDATER_WIN_NAME) {
        Some(window) => {
            window.unminimize().unwrap();
            window.set_focus().unwrap();
            window
        }
        None => {
            let builder = tauri::WebviewWindowBuilder::new(
                handle,
                UPDATER_WIN_NAME,
                tauri::WebviewUrl::App("src/tauri/index.html".into()),
            )
            .title("OpenAI Translator Updater")
            .fullscreen(false)
            .inner_size(500.0, 500.0)
            .min_inner_size(200.0, 200.0)
            .resizable(true)
            .skip_taskbar(true)
            .focused(true);

            return build_window(builder);
        }
    };

    window
}

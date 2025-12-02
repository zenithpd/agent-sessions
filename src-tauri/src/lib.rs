#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod process;
mod session;
mod terminal;

use tauri::{
    Manager,
    tray::TrayIconBuilder,
    menu::{MenuBuilder, MenuItemBuilder},
};
use std::sync::Mutex;

use session::{get_sessions, SessionsResponse};

// Store tray icon ID for updates
static TRAY_ID: Mutex<Option<String>> = Mutex::new(None);

#[tauri::command]
fn get_all_sessions() -> SessionsResponse {
    get_sessions()
}

#[tauri::command]
fn focus_session(pid: u32, project_path: String) -> Result<(), String> {
    terminal::focus_terminal_for_pid(pid)
        .or_else(|_| terminal::focus_terminal_by_path(&project_path))
}

#[tauri::command]
fn update_tray_title(app: tauri::AppHandle, total: usize, waiting: usize) -> Result<(), String> {
    let title = if waiting > 0 {
        format!("{} ({} waiting)", total, waiting)
    } else if total > 0 {
        format!("{}", total)
    } else {
        String::new()
    };

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_title(Some(&title))
            .map_err(|e| format!("Failed to set tray title: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_all_sessions, focus_session, update_tray_title])
        .setup(|app| {
            // Create menu for tray
            let show_item = MenuItemBuilder::with_id("show", "Show Window")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon with menu
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Store tray ID
            *TRAY_ID.lock().unwrap() = Some("main-tray".to_string());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

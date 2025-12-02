#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod process;
mod session;
mod terminal;

use session::{get_sessions, SessionsResponse};

#[tauri::command]
fn get_all_sessions() -> SessionsResponse {
    get_sessions()
}

#[tauri::command]
fn focus_session(pid: u32, project_path: String) -> Result<(), String> {
    terminal::focus_terminal_for_pid(pid)
        .or_else(|_| terminal::focus_terminal_by_path(&project_path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_all_sessions, focus_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

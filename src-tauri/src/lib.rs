// 预处理命令模块
mod convert;
mod inspection;
mod models;

use std::process::Child;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

use crate::inspection::InspectionManager;
use crate::models::*;

/// 全局应用状态：巡检管理器 + 当前运行中的子进程
struct AppState {
    manager: InspectionManager,
    running: Arc<Mutex<Option<Child>>>,
}

// ===================== Tauri 命令 =====================

#[tauri::command]
fn start_inspection(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    req: InspectionRequest,
) -> Result<String, String> {
    let mgr = state.manager.clone();
    let running = state.running.clone();
    mgr.start(app, running, req)
}

#[tauri::command]
fn stop_inspection(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    state.manager.stop(state.running.clone())
}

#[tauri::command]
fn read_report(state: State<'_, Arc<AppState>>, path: String) -> Result<String, String> {
    state.manager.read_report(&path)
}

#[tauri::command]
fn export_report(
    state: State<'_, Arc<AppState>>,
    src: String,
    format: String,
    dest_dir: String,
) -> Result<String, String> {
    state.manager.export_report(&src, &format, &dest_dir)
}

#[tauri::command]
fn get_history(state: State<'_, Arc<AppState>>) -> Result<Vec<HistoryItem>, String> {
    state.manager.get_history()
}

#[tauri::command]
fn delete_history_item(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    state.manager.delete_history_item(&id)
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    state.manager.get_settings()
}

#[tauri::command]
fn save_settings(state: State<'_, Arc<AppState>>, settings: Settings) -> Result<(), String> {
    state.manager.save_settings(&settings)
}

#[tauri::command]
fn get_os_info(state: State<'_, Arc<AppState>>) -> OsInfo {
    state.manager.os_info()
}

#[tauri::command]
fn check_permission(state: State<'_, Arc<AppState>>) -> PermissionInfo {
    state.manager.check_permission()
}

#[tauri::command]
fn pick_directory(app: AppHandle, state: State<'_, Arc<AppState>>) -> Option<String> {
    state.manager.pick_dir(&app)
}

#[tauri::command]
fn open_path(app: AppHandle, state: State<'_, Arc<AppState>>, path: String) -> Result<(), String> {
    state.manager.open_path(&app, &path)
}

#[tauri::command]
fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let manager = InspectionManager::new(app.handle()).map_err(|e| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
                    as Box<dyn std::error::Error>
            })?;
            app.manage(Arc::new(AppState {
                manager,
                running: Arc::new(Mutex::new(None)),
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_inspection,
            stop_inspection,
            read_report,
            export_report,
            get_history,
            delete_history_item,
            get_settings,
            save_settings,
            get_os_info,
            check_permission,
            pick_directory,
            open_path,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

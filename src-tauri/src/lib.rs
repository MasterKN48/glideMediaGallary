mod db;
mod scanner;

use db::{MediaItem, get_db_path, init_db, add_folder as db_add_folder, remove_folder as db_remove_folder, get_folders, get_all_media, delete_media_by_prefix};
use scanner::{scan_directory_in_background, get_or_create_thumbnail};
use tauri::{AppHandle, command};

#[command]
fn init_app(app_handle: AppHandle) -> Result<(Vec<String>, Vec<MediaItem>), String> {
    let db_path = get_db_path(&app_handle)?;
    init_db(&db_path).map_err(|e| format!("DB init error: {}", e))?;
    
    let folders = get_folders(&db_path).map_err(|e| format!("DB get_folders error: {}", e))?;
    let media = get_all_media(&db_path).map_err(|e| format!("DB get_all_media error: {}", e))?;
    
    Ok((folders, media))
}

#[command]
fn add_folder(app_handle: AppHandle, path: String) -> Result<Vec<String>, String> {
    let db_path = get_db_path(&app_handle)?;
    db_add_folder(&db_path, &path).map_err(|e| format!("DB add_folder error: {}", e))?;
    
    // Trigger background scan
    scan_directory_in_background(app_handle.clone(), path);
    
    let folders = get_folders(&db_path).map_err(|e| format!("DB get_folders error: {}", e))?;
    Ok(folders)
}

#[command]
fn remove_folder(app_handle: AppHandle, path: String) -> Result<Vec<String>, String> {
    let db_path = get_db_path(&app_handle)?;
    db_remove_folder(&db_path, &path).map_err(|e| format!("DB remove_folder error: {}", e))?;
    delete_media_by_prefix(&db_path, &path).map_err(|e| format!("DB delete_media error: {}", e))?;
    
    let folders = get_folders(&db_path).map_err(|e| format!("DB get_folders error: {}", e))?;
    Ok(folders)
}

#[command]
fn start_scan(app_handle: AppHandle, path: String) -> Result<(), String> {
    scan_directory_in_background(app_handle, path);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            init_app,
            add_folder,
            remove_folder,
            start_scan,
            get_or_create_thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

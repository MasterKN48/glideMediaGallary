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

#[command]
fn get_media_metadata(path: String) -> Result<std::collections::HashMap<String, String>, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("File does not exist".into());
    }

    let mut metadata = std::collections::HashMap::new();

    // Check file stats
    if let Ok(meta) = std::fs::metadata(p) {
        metadata.insert("size".to_string(), meta.len().to_string());
    }

    // Try reading EXIF
    if let Ok(file) = std::fs::File::open(p) {
        let mut reader = std::io::BufReader::new(file);
        let exif_reader = exif::Reader::new();
        if let Ok(exif) = exif_reader.read_from_container(&mut reader) {
            for field in exif.fields() {
                let tag_name = format!("{:?}", field.tag);
                let val_str = format!("{}", field.display_value().with_unit(&exif));
                metadata.insert(tag_name, val_str);
            }
        }
    }

    Ok(metadata)
}

#[command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(&["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(&["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new("/"));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
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
            get_or_create_thumbnail,
            reveal_in_finder,
            get_media_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

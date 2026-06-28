use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::fs;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MediaItem {
    pub file_path: String,
    pub filename: String,
    pub media_type: String, // "image" | "video" | "audio"
    pub size: u64,
    pub modified_time: i64,
    pub created_time: i64,
    pub thumbnail_path: Option<String>,
    pub orientation: u32,
}

pub fn get_db_path(app_handle: &tauri::AppHandle) -> std::result::Result<PathBuf, String> {
    use tauri::Manager;
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    // Create directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    Ok(app_dir.join("media_gallery.db"))
}

pub fn init_db(db_path: &PathBuf) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS media (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            filename TEXT NOT NULL,
            media_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            modified_time INTEGER NOT NULL,
            created_time INTEGER NOT NULL,
            thumbnail_path TEXT,
            orientation INTEGER DEFAULT 1
        )",
        [],
    )?;

    Ok(())
}

pub fn add_folder(db_path: &PathBuf, path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT OR IGNORE INTO folders (path) VALUES (?1)",
        params![path],
    )?;
    Ok(())
}

pub fn remove_folder(db_path: &PathBuf, path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM folders WHERE path = ?1", params![path])?;
    Ok(())
}

pub fn get_folders(db_path: &PathBuf) -> Result<Vec<String>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT path FROM folders")?;
    let folder_iter = stmt.query_map([], |row| row.get(0))?;
    
    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder?);
    }
    Ok(folders)
}

pub fn insert_media_batch(db_path: &PathBuf, items: &[MediaItem]) -> Result<()> {
    let mut conn = Connection::open(db_path)?;
    let tx = conn.transaction()?;
    
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO media 
            (file_path, filename, media_type, size, modified_time, created_time, thumbnail_path, orientation) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        
        for item in items {
            stmt.execute(params![
                item.file_path,
                item.filename,
                item.media_type,
                item.size,
                item.modified_time,
                item.created_time,
                item.thumbnail_path,
                item.orientation,
            ])?;
        }
    }
    
    tx.commit()?;
    Ok(())
}

pub fn get_all_media(db_path: &PathBuf) -> Result<Vec<MediaItem>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_path, filename, media_type, size, modified_time, created_time, thumbnail_path, orientation FROM media"
    )?;
    
    let item_iter = stmt.query_map([], |row| {
        Ok(MediaItem {
            file_path: row.get(0)?,
            filename: row.get(1)?,
            media_type: row.get(2)?,
            size: row.get(3)?,
            modified_time: row.get(4)?,
            created_time: row.get(5)?,
            thumbnail_path: row.get(6)?,
            orientation: row.get(7)?,
        })
    })?;
    
    let mut items = Vec::new();
    for item in item_iter {
        items.push(item?);
    }
    Ok(items)
}

pub fn delete_media_by_prefix(db_path: &PathBuf, prefix: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "DELETE FROM media WHERE file_path LIKE ?1 || '%'",
        params![prefix],
    )?;
    Ok(())
}

pub fn update_media_thumbnail(db_path: &PathBuf, file_path: &str, thumbnail_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "UPDATE media SET thumbnail_path = ?1 WHERE file_path = ?2",
        params![thumbnail_path, file_path],
    )?;
    Ok(())
}

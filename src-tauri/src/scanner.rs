use jwalk::WalkDir;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tauri::{AppHandle, Emitter, Manager};
use exif::{In, Reader, Tag};

use crate::db::{self, MediaItem};

// Map file extension to media type
fn get_media_type(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" | "bmp" | "tiff" | "dng" | "arw" | "cr2" | "nef" => Some("image"),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "3gp" => Some("video"),
        "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => Some("audio"),
        _ => None,
    }
}

// Generate unique hash filename for a given file path
fn hash_path(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Check if year is a leap year
fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

// Convert UTC date components to a UNIX timestamp
fn utc_to_timestamp(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> i64 {
    let mut days = 0i64;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        if m == 2 && is_leap_year(year) {
            days += 29;
        } else {
            days += month_days[m as usize];
        }
    }
    days += (day as i64) - 1;
    let hours = days * 24 + (hour as i64);
    let minutes = hours * 60 + (min as i64);
    let seconds = minutes * 60 + (sec as i64);
    seconds
}

// Parse EXIF date string "YYYY:MM:DD HH:MM:SS"
fn parse_exif_date_str(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() >= 19 {
        let year: i32 = s[0..4].parse().ok()?;
        let month: u32 = s[5..7].parse().ok()?;
        let day: u32 = s[8..10].parse().ok()?;
        let hour: u32 = s[11..13].parse().ok()?;
        let min: u32 = s[14..16].parse().ok()?;
        let sec: u32 = s[17..19].parse().ok()?;
        Some(utc_to_timestamp(year, month, day, hour, min, sec))
    } else {
        None
    }
}

// Extract EXIF date and EXIF embedded thumbnail
fn get_exif_metadata(path: &Path) -> (Option<i64>, Option<Vec<u8>>) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None),
    };
    let mut bufreader = BufReader::new(file);
    let mut reader = Reader::new();
    let exif = match reader.read_from_container(&mut bufreader) {
        Ok(e) => e,
        Err(_) => return (None, None),
    };

    let mut timestamp = None;
    for tag in &[Tag::DateTimeOriginal, Tag::DateTimeDigitized, Tag::DateTime] {
        if let Some(field) = exif.get_field(*tag, In::PRIMARY) {
            let val_str = format!("{}", field.display_value().with_unit(&exif));
            if let Some(ts) = parse_exif_date_str(&val_str) {
                timestamp = Some(ts);
                break;
            }
        }
    }

    let mut thumb = None;
    if let (Some(offset_field), Some(len_field)) = (
        exif.get_field(Tag::JPEGInterchangeFormat, In::THUMBNAIL),
        exif.get_field(Tag::JPEGInterchangeFormatLength, In::THUMBNAIL)
    ) {
        if let (Some(offset), Some(len)) = (offset_field.value.get_uint(0), len_field.value.get_uint(0)) {
            let offset = offset as usize;
            let len = len as usize;
            let buf = exif.buf();
            if offset + len <= buf.len() {
                thumb = Some(buf[offset..(offset + len)].to_vec());
            }
        }
    }

    (timestamp, thumb)
}

// Perform scanning in a separate thread
pub fn scan_directory_in_background(
    app: AppHandle,
    scan_path: String,
) {
    std::thread::spawn(move || {
        let db_path = match db::get_db_path(&app) {
            Ok(p) => p,
            Err(e) => {
                let _ = app.emit("scan_error", format!("Database path error: {}", e));
                return;
            }
        };

        let cache_dir = match app.path().app_cache_dir() {
            Ok(p) => p,
            Err(e) => {
                let _ = app.emit("scan_error", format!("Cache directory error: {}", e));
                return;
            }
        };
        let thumb_dir = cache_dir.join("thumbnails");
        if !thumb_dir.exists() {
            let _ = fs::create_dir_all(&thumb_dir);
        }

        let _ = app.emit("scan_status", format!("Scanning started for: {}", scan_path));

        let mut batch = Vec::with_capacity(100);
        let mut total_scanned = 0;

        for entry in WalkDir::new(&scan_path) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = match path.extension().and_then(|s| s.to_str()) {
                Some(e) => e,
                None => continue,
            };

            let media_type = match get_media_type(ext) {
                Some(t) => t,
                None => continue,
            };

            let file_path_str = path.to_string_lossy().to_string();
            let filename = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            let modified_time = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64))
                .unwrap_or(0);
            
            let mut created_time = metadata.created()
                .ok()
                .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64))
                .unwrap_or(modified_time);

            let mut thumbnail_path = None;

            // Extract EXIF data if it is an image
            if media_type == "image" {
                let (exif_date, exif_thumb) = get_exif_metadata(&path);
                if let Some(date) = exif_date {
                    created_time = date;
                }
                
                // If there is an EXIF thumbnail, write it to cache instantly
                if let Some(bytes) = exif_thumb {
                    let thumb_filename = format!("{}.jpg", hash_path(&file_path_str));
                    let full_thumb_path = thumb_dir.join(&thumb_filename);
                    if let Ok(mut file) = File::create(&full_thumb_path) {
                        let _ = file.write_all(&bytes);
                        thumbnail_path = Some(full_thumb_path.to_string_lossy().to_string());
                    }
                }
            }

            let item = MediaItem {
                file_path: file_path_str,
                filename,
                media_type: media_type.to_string(),
                size,
                modified_time,
                created_time,
                thumbnail_path,
            };

            batch.push(item);
            total_scanned += 1;

            if batch.len() >= 100 {
                let _ = db::insert_media_batch(&db_path, &batch);
                let _ = app.emit("scanned_batch", batch.clone());
                batch.clear();
            }
        }

        // Insert remaining items
        if !batch.is_empty() {
            let _ = db::insert_media_batch(&db_path, &batch);
            let _ = app.emit("scanned_batch", batch);
        }

        let _ = app.emit("scan_status", format!("Scanning completed. Total items scanned: {}", total_scanned));
    });
}

#[tauri::command]
pub fn get_or_create_thumbnail(app: AppHandle, file_path: String) -> Result<String, String> {
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        let _ = fs::create_dir_all(&thumb_dir);
    }
    
    let thumb_filename = format!("{}.jpg", hash_path(&file_path));
    let full_thumb_path = thumb_dir.join(&thumb_filename);
    
    if full_thumb_path.exists() {
        return Ok(full_thumb_path.to_string_lossy().to_string());
    }
    
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("Original file does not exist".to_string());
    }
    
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default();
    let media_type = get_media_type(ext).unwrap_or_default();
    
    if media_type == "image" {
        let img = image::ImageReader::open(path)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;
            
        let thumbnail = img.thumbnail(250, 250);
        
        let mut out_file = File::create(&full_thumb_path).map_err(|e| e.to_string())?;
        thumbnail.write_to(&mut out_file, image::ImageFormat::Jpeg)
            .map_err(|e| e.to_string())?;
            
        let db_path = db::get_db_path(&app)?;
        let full_thumb_str = full_thumb_path.to_string_lossy().to_string();
        let _ = db::update_media_thumbnail(&db_path, &file_path, &full_thumb_str);
        
        Ok(full_thumb_str)
    } else {
        Err("Unsupported media type for thumbnail generation".to_string())
    }
}

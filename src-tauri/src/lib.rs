mod scanner;
mod exif;
mod inference;
mod renamer;
mod thumbnail;
mod log_ops;

use scanner::ImageEntry;
use inference::InferResult;
use log_ops::RenameEntry;
use std::path::Path;

#[tauri::command]
fn scan_folder(folder_path: String) -> Result<Vec<ImageEntry>, String> {
    scanner::scan_folder(&folder_path)
}

#[tauri::command]
fn infer_date(folder_path: String, entries: Vec<ImageEntry>) -> Result<InferResult, String> {
    Ok(inference::infer_date(&folder_path, &entries))
}

#[tauri::command]
fn rename_folder(
    old_path: String,
    location: String,
    title: String,
    date: String,
) -> Result<String, String> {
    let new_path = renamer::rename_folder(&old_path, &location, &title, &date)?;
    let _ = log_ops::append_log(&old_path, &new_path);
    Ok(new_path)
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| format!("打开文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
fn extract_gps(folder_path: String) -> Result<Option<(f64, f64)>, String> {
    let path_obj = Path::new(&folder_path);
    if !path_obj.is_dir() {
        if let Some(coords) = exif::extract_gps(&folder_path) {
            return Ok(Some(coords));
        }
        return Ok(None);
    }
    let entries = scanner::scan_folder(&folder_path)?;
    for entry in &entries {
        if let Some(coords) = exif::extract_gps(&entry.path) {
            return Ok(Some(coords));
        }
    }
    Ok(None)
}

#[tauri::command]
fn get_thumbnail(path: String) -> Result<String, String> {
    thumbnail::generate_thumbnail(&path)
}

#[tauri::command]
fn list_history(folder_path: String) -> Result<Vec<RenameEntry>, String> {
    log_ops::list_history(&folder_path)
}

#[tauri::command]
fn undo_rename(folder_path: String) -> Result<(String, String), String> {
    log_ops::undo_last(&folder_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            infer_date,
            rename_folder,
            open_file,
            extract_gps,
            get_thumbnail,
            list_history,
            undo_rename,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
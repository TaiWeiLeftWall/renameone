mod scanner;
mod exif;
mod inference;
mod renamer;

use scanner::ImageEntry;
use inference::InferResult;

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
    renamer::rename_folder(&old_path, &location, &title, &date)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageEntry {
    pub path: String,
    pub filename: String,
}

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "tiff", "tif", "webp", "heic", "heif", "mp4", "mov", "avi", "mkv", "3gp", "webm", "m4v"];

pub fn scan_folder(folder_path: &str) -> Result<Vec<ImageEntry>, String> {
    let path = Path::new(folder_path);
    if !path.exists() {
        return Err(format!("文件夹不存在: {}", folder_path));
    }
    if !path.is_dir() {
        return Err(format!("路径不是文件夹: {}", folder_path));
    }

    let mut entries = Vec::new();
    for entry in WalkDir::new(folder_path).min_depth(1).max_depth(3) {
        let entry = entry.map_err(|e| format!("读取文件夹失败: {}", e))?;
        if entry.file_type().is_file() {
            let ext = entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                entries.push(ImageEntry {
                    path: entry.path().to_string_lossy().to_string(),
                    filename: entry.file_name().to_string_lossy().to_string(),
                });
            }
        }
    }

    Ok(entries)
}
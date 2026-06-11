use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameEntry {
    pub timestamp: String,
    pub old_path: String,
    pub new_path: String,
    pub old_name: String,
    pub new_name: String,
}

fn get_log_dir(app_handle: &AppHandle, custom_path: Option<&str>) -> PathBuf {
    if let Some(path) = custom_path {
        if !path.is_empty() {
            let p = PathBuf::from(path);
            if p.is_absolute() { return p; }
        }
    }
    app_handle.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn get_log_path(app_handle: &AppHandle, custom_path: Option<&str>) -> PathBuf {
    get_log_dir(app_handle, custom_path).join("_archive_log.json")
}

pub fn append_log(app_handle: &AppHandle, old_path: &str, new_path: &str, custom_path: Option<&str>) -> Result<(), String> {
    let log_path = get_log_path(app_handle, custom_path);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let old_name = Path::new(old_path).file_name()
        .ok_or("无法获取原文件夹名")?.to_string_lossy().to_string();
    let new_name = Path::new(new_path).file_name()
        .ok_or("无法获取新文件夹名")?.to_string_lossy().to_string();
    let entry = RenameEntry {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        old_path: old_path.to_string(),
        new_path: new_path.to_string(),
        old_name,
        new_name,
    };
    let mut entries: Vec<RenameEntry> = if log_path.exists() {
        let content = std::fs::read_to_string(&log_path).map_err(|e| format!("读取日志失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    entries.push(entry);
    let content = serde_json::to_string_pretty(&entries).map_err(|e| format!("序列化日志失败: {}", e))?;
    std::fs::write(&log_path, content).map_err(|e| format!("写入日志失败: {}", e))?;
    Ok(())
}

pub fn list_history(app_handle: &AppHandle, custom_path: Option<&str>) -> Result<Vec<RenameEntry>, String> {
    let log_path = get_log_path(app_handle, custom_path);
    if !log_path.exists() { return Ok(Vec::new()); }
    let content = std::fs::read_to_string(&log_path).map_err(|e| format!("读取日志失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析日志失败: {}", e))
}

pub fn undo_last(app_handle: &AppHandle, custom_path: Option<&str>) -> Result<(String, String), String> {
    let log_path = get_log_path(app_handle, custom_path);
    let content = std::fs::read_to_string(&log_path).map_err(|e| format!("读取日志失败: {}", e))?;
    let mut entries: Vec<RenameEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("解析日志失败: {}", e))?;
    let last = entries.pop().ok_or("没有可撤销的操作")?;
    if Path::new(&last.new_path).exists() {
        std::fs::rename(&last.new_path, &last.old_path)
            .map_err(|e| format!("撤销重命名失败: {}", e))?;
    }
    let content = serde_json::to_string_pretty(&entries).map_err(|e| format!("更新日志失败: {}", e))?;
    std::fs::write(&log_path, content).map_err(|e| format!("更新日志失败: {}", e))?;
    Ok((last.old_name, last.new_name))
}
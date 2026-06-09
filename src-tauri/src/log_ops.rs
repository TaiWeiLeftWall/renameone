use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameEntry {
    pub timestamp: String,
    pub old_path: String,
    pub new_path: String,
    pub old_name: String,
    pub new_name: String,
}

fn get_log_path(folder_path: &str) -> String {
    let parent = Path::new(folder_path).parent().unwrap_or(Path::new("."));
    parent.join("_archive_log.json").to_string_lossy().to_string()
}

pub fn append_log(old_path: &str, new_path: &str) -> Result<(), String> {
    let log_path = get_log_path(old_path);
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

    let mut entries: Vec<RenameEntry> = if Path::new(&log_path).exists() {
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

pub fn list_history(folder_path: &str) -> Result<Vec<RenameEntry>, String> {
    let log_path = get_log_path(folder_path);
    if !Path::new(&log_path).exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&log_path).map_err(|e| format!("读取日志失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析日志失败: {}", e))
}

pub fn undo_last(folder_path: &str) -> Result<(String, String), String> {
    let log_path = get_log_path(folder_path);
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

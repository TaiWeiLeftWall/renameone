use std::path::Path;

/// 过滤文件夹名中的非法字符
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 构建目标文件夹名并执行重命名
/// 返回新的完整文件夹路径
pub fn rename_folder(old_path: &str, location: &str, title: &str, date: &str) -> Result<String, String> {
    let old = Path::new(old_path);

    if !old.exists() {
        return Err("原文件夹不存在".to_string());
    }

    let parent = old.parent().ok_or("无法获取上级目录".to_string())?;
    let location = sanitize_name(location);
    let title = sanitize_name(title);

    if location.is_empty() {
        return Err("地点不能为空".to_string());
    }
    if title.is_empty() {
        return Err("标题不能为空".to_string());
    }

    let mut new_name = format!("{}_{}_{}", date, location, title);

    // 检查长度限制 (Windows MAX_PATH = 255)
    let parent_str = parent.to_string_lossy();
    let full_len = parent_str.len() + 1 + new_name.len(); // +1 for backslash
    if full_len > 255 {
        let max_title_len = 255 - parent_str.len() - 1 - date.len() - 1 - location.len() - 1;
        if max_title_len < 1 {
            return Err("路径过长，请缩短地点或标题".to_string());
        }
        new_name = format!("{}_{}_{}", date, location, &title[..max_title_len as usize]);
    }

    // 重名冲突处理
    let mut target = parent.join(&new_name);
    let mut counter = 1;
    while target.exists() {
        let deduped = format!("{}_{:03}", &new_name, counter);
        target = parent.join(&deduped);
        counter += 1;
        if counter > 999 {
            return Err("目录名冲突过多，请更换地点或标题".to_string());
        }
    }

    // 执行重命名
    std::fs::rename(old, &target)
        .map_err(|e| format!("重命名失败: {}", e))?;

    Ok(target.to_string_lossy().to_string())
}

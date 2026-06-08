use chrono::NaiveDate;
use exif::{Reader as ExifReader, Tag};
use std::fs::File;
use std::path::Path;

/// 从图片文件中提取拍摄日期，按优先级降级
pub fn extract_date(path_str: &str) -> Option<NaiveDate> {
    let path = Path::new(path_str);

    // 优先级 1-3: EXIF 元数据
    if let Ok(file) = File::open(path) {
        let reader = ExifReader::new().read_from_container(&mut std::io::BufReader::new(file));
        if let Ok(reader) = reader {
            // 1: DateTimeOriginal
            if let Some(field) = reader.get_field(Tag::DateTimeOriginal, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return Some(date);
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return Some(date);
                }
            }
            // 2: DateTimeDigitized
            if let Some(field) = reader.get_field(Tag::DateTimeDigitized, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return Some(date);
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return Some(date);
                }
            }
            // 3: DateTime
            if let Some(field) = reader.get_field(Tag::DateTime, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return Some(date);
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return Some(date);
                }
            }
        }
    }

    // 4: 文件修改时间
    if let Ok(metadata) = path.metadata() {
        if let Ok(mtime) = metadata.modified() {
            let datetime: chrono::DateTime<chrono::Utc> = mtime.into();
            return Some(datetime.date_naive());
        }
    }

    // 5: 文件名正则提取 (如 IMG_20240315, PXL_20240315_123456)
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        let digits: String = filename.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 8 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                digits[0..4].parse::<i32>(),
                digits[4..6].parse::<u32>(),
                digits[6..8].parse::<u32>(),
            ) {
                if (1..=12).contains(&month) && (1..=31).contains(&day) {
                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                        return Some(date);
                    }
                }
            }
        }
    }

    None
}
use chrono::NaiveDate;
use exif::{Reader as ExifReader, Tag};
use serde::Serialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub struct ExtractedDate {
    pub date: Option<NaiveDate>,
    pub source: Option<String>,
}

/// 从图片文件中提取拍摄日期，按优先级降级，返回日期及来源
pub fn extract_date(path_str: &str) -> ExtractedDate {
    let path = Path::new(path_str);

    // 优先级 1-3: EXIF 元数据
    if let Ok(file) = File::open(path) {
        let reader = ExifReader::new().read_from_container(&mut std::io::BufReader::new(file));
        if let Ok(reader) = reader {
            if let Some(field) = reader.get_field(Tag::DateTimeOriginal, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_original".to_string()) };
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_original".to_string()) };
                }
            }
            if let Some(field) = reader.get_field(Tag::DateTimeDigitized, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_digitized".to_string()) };
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_digitized".to_string()) };
                }
            }
            if let Some(field) = reader.get_field(Tag::DateTime, exif::In::PRIMARY) {
                let val = field.display_value().to_string();
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_dt".to_string()) };
                }
                if let Ok(date) = NaiveDate::parse_from_str(&val, "%Y:%m:%d %H:%M:%S") {
                    return ExtractedDate { date: Some(date), source: Some("exif_dt".to_string()) };
                }
            }
        }
    }

    // 4: 文件修改时间
    if let Ok(metadata) = path.metadata() {
        if let Ok(mtime) = metadata.modified() {
            let datetime: chrono::DateTime<chrono::Utc> = mtime.into();
            return ExtractedDate { date: Some(datetime.date_naive()), source: Some("mtime".to_string()) };
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
                        return ExtractedDate { date: Some(date), source: Some("filename".to_string()) };
                    }
                }
            }
        }
    }

    ExtractedDate { date: None, source: None }
}
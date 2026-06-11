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

    // 6: Video metadata (MP4/MOV)
    if let Some(vdate) = crate::video::extract_video_date(path_str) {
        return ExtractedDate { date: Some(vdate), source: Some("video_metadata".to_string()) };
    }

    ExtractedDate { date: None, source: None }
}
/// 从图片文件中提取 GPS 坐标
pub fn extract_gps(path_str: &str) -> Option<(f64, f64)> {
    let path = Path::new(path_str);
    let file = File::open(path).ok()?;
    let reader = ExifReader::new().read_from_container(&mut std::io::BufReader::new(file)).ok()?;

    let lat_field = reader.get_field(Tag::GPSLatitude, exif::In::PRIMARY)?;
    let lon_field = reader.get_field(Tag::GPSLongitude, exif::In::PRIMARY)?;
    let lat_ref = reader.get_field(Tag::GPSLatitudeRef, exif::In::PRIMARY)?;
    let lon_ref = reader.get_field(Tag::GPSLongitudeRef, exif::In::PRIMARY)?;

    let lat = parse_gps_value(lat_field)?;
    let lon = parse_gps_value(lon_field)?;

    let lat = if lat_ref.display_value().to_string().trim() == "S" { -lat } else { lat };
    let lon = if lon_ref.display_value().to_string().trim() == "W" { -lon } else { lon };

    Some((lat, lon))
}

fn parse_gps_value(field: &exif::Field) -> Option<f64> {
    let display = field.display_value().to_string();
    // Format: "40/1, 45/1, 30/1" or "40, 45, 30"
    let parts: Vec<f64> = display
        .split(|c: char| c == ',' || c == '/')
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { s.parse::<f64>().ok() }
        })
        .collect();

    if parts.len() == 6 {
        // Rational format: num/den, num/den, num/den
        Some(parts[0] / parts[1] + parts[2] / parts[3] / 60.0 + parts[4] / parts[5] / 3600.0)
    } else if parts.len() == 3 {
        // Decimal format: deg, min, sec
        Some(parts[0] + parts[1] / 60.0 + parts[2] / 3600.0)
    } else if parts.len() == 1 {
        Some(parts[0])
    } else {
        None
    }
}

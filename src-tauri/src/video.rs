use chrono::NaiveDate;
use chrono::NaiveDateTime;

pub fn extract_video_date(path_str: &str) -> Option<NaiveDate> {
    let path = std::path::Path::new(path_str);
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "mp4" | "mov" | "m4v" => parse_mp4_creation_date(path_str),
        _ => None,
    }
}

fn parse_mp4_creation_date(path_str: &str) -> Option<NaiveDate> {
    let data = std::fs::read(path_str).ok()?;
    if data.len() < 16 { return None; }
    let seconds = find_mvhd_creation_time(&data, 0)?;
    let base = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(1904, 1, 1)?,
        chrono::NaiveTime::from_hms_opt(0, 0, 0)?
    );
    Some((base + chrono::Duration::seconds(seconds as i64)).date())
}

fn find_mvhd_creation_time(data: &[u8], start: usize) -> Option<u32> {
    let mut pos = start;
    while pos + 8 <= data.len() {
        let size = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        if size == 0 || size < 8 || pos + size > data.len() { break; }
        let box_type = &data[pos+4..pos+8];
        if box_type == b"mvhd" && size >= 20 {
            return Some(u32::from_be_bytes([data[pos+12], data[pos+13], data[pos+14], data[pos+15]]));
        }
        if box_type == b"moov" {
            let result = find_mvhd_creation_time(data, pos + 8);
            if result.is_some() { return result; }
        }
        pos += size;
    }
    None
}

use image::GenericImageView;
use std::io::Cursor;

pub fn generate_thumbnail(path: &str) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("无法读取图片: {}", e))?;
    let (w, h) = img.dimensions();
    let max_size: u32 = 120;
    let (nw, nh) = if w > h {
        (max_size, (h * max_size / w).max(1))
    } else {
        ((w * max_size / h).max(1), max_size)
    };
    let thumb = img.resize_exact(nw, nh, image::imageops::FilterType::CatmullRom);
    let mut buf = Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("编码缩略图失败: {}", e))?;
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());
    Ok(format!("data:image/jpeg;base64,{}", b64))
}
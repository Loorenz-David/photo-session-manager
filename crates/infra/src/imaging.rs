use psm_core::{Result, domain::composition::Frame, ports::images::Imaging};
use std::{fs, path::Path};

pub struct ImageOpsImpl;
impl Imaging for ImageOpsImpl {
    fn dimensions(&self, path: &Path) -> Result<Option<Frame>> {
        Ok(image::image_dimensions(path).ok().map(|(w, h)| Frame {
            w: i64::from(w),
            h: i64::from(h),
        }))
    }
    fn thumbnail(&self, source: &Path, dest: &Path, long_edge: u32) -> Result<bool> {
        let Ok(img) = image::open(source) else {
            return Ok(false);
        };
        let scaled = img.resize(long_edge, long_edge, image::imageops::FilterType::Lanczos3);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = dest.with_extension(format!("{}.tmp", std::process::id()));
        let file = fs::File::create(&temp)?;
        let mut writer = std::io::BufWriter::new(file);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 80);
        if encoder.encode_image(&scaled).is_err() {
            let _ = fs::remove_file(temp);
            return Ok(false);
        }
        drop(writer);
        fs::rename(temp, dest)?;
        Ok(true)
    }
}

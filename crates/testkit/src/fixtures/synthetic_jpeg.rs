pub struct SyntheticJpeg;
impl SyntheticJpeg {
    pub fn rgb(w: u32, h: u32, colour: [u8; 3]) -> Vec<u8> {
        let image = image::RgbImage::from_pixel(w, h, image::Rgb(colour));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90)
            .encode_image(&image)
            .expect("synthetic jpeg");
        bytes
    }
}

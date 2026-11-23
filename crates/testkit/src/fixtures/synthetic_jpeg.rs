use image::ImageEncoder;

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

    /// As [`Self::rgb`], with an Exif APP1 segment whose IFD0 carries the orientation tag
    /// (0x0112, SHORT) set to `orientation` (H3·U1). The pixels are not rotated.
    pub fn with_orientation(w: u32, h: u32, colour: [u8; 3], orientation: u8) -> Vec<u8> {
        let image = image::RgbImage::from_pixel(w, h, image::Rgb(colour));
        // A little-endian TIFF: header, IFD0 at 8 with one entry, no next IFD.
        let mut tiff = vec![b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0];
        tiff.extend_from_slice(&[0x12, 0x01, 3, 0, 1, 0, 0, 0, orientation, 0, 0, 0]);
        tiff.extend_from_slice(&[0, 0, 0, 0]);
        let mut bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90);
        encoder.set_exif_metadata(tiff).expect("exif metadata");
        encoder.encode_image(&image).expect("synthetic jpeg");
        bytes
    }
}

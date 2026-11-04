//! The output renderer of 10D MC-28 (invariant M34). The render contract (P29), verbatim:
//!
//! `Imaging::render_output(master, values, slot, format, dest) -> OutputInfo { width, height, frame }`:
//! 1. Decode the master to 16-bit RGB (`image`, TIFF). `frame = (W0, H0)`. Anything but RGB
//!    8/16-bit → `Err(InvalidData)`.
//! 2. Convert to **linear light** with a 65 536-entry LUT of the sRGB transfer function (the master
//!    is sRGB by recipe).
//! 3. Quarter turns (clockwise) and flip (horizontal, applied **after** the turns, in the turned
//!    frame): pure index remaps. A clockwise quarter turn maps source `(x, y)` to turned
//!    `(H0 − 1 − y, x)` in a frame of `H0 × W0`; the horizontal flip maps turned `x'` to
//!    `W_t − 1 − x'`.
//! 4. The pixel rect of MC-27's pixel rule on the turned frame `W_t × H_t` (`pixel_rect`, then
//!    `split_halves` and the half named by `slot`).
//! 5. Straighten: with `θ = values.straighten` (degrees, clockwise on screen), output pixel `p`
//!    (pixel centre, turned-frame coordinates) samples the turned, flipped source at
//!    `s = c + R(−θ)(p − c)` with `c = (W_t/2, H_t/2)` (the exact formula of P28's contract),
//!    **bicubic** interpolation in linear light; `θ = 0` is a plain copy (no resampling).
//! 6. Resize to the output size with **Lanczos3** when the rect's long edge is greater than
//!    `format.long_edge`; the short edge is `round(long · short_rect / long_rect)`; **never upscale**.
//! 7. Back to sRGB (LUT), 8 bits per channel; encode **once** as JPEG: quality `format.quality`,
//!    chroma **4:4:4**, an sRGB ICC profile in APP2, the EXIF APP1 of step 8; written to a temp
//!    file then renamed to `dest`.
//! 8. Metadata: from the master's EXIF (`kamadak-exif` reads TIFF containers) copy **only** `Make`,
//!    `Model`, `DateTimeOriginal` (fallback `DateTime`), `Copyright` when present; write
//!    `Orientation = 1`; the writer may add the structural tags `ExifVersion`, `ColorSpace = 1`,
//!    `PixelXDimension`, `PixelYDimension`, `ResolutionUnit`, `XResolution`, `YResolution`,
//!    `YCbCrPositioning`; **nothing else** — no GPS IFD, no MakerNote, no XMP packet, no vendor
//!    tags (an allowlist, not a denylist).
//!
//! The master and the RAW are opened read-only; the output is the only file written.

use super::{linear, metadata};
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, images::Image};
use jpeg_encoder::{ColorType, Encoder, SamplingFactor};
use psm_core::{
    Error, Result,
    domain::{
        composition::{CompositionKind, CompositionValues, Flip, Frame, OutputSlot, PixelRect},
        output::{OutputFormat, OutputInfo},
    },
    mechanisms::composition::{pixel_rect, split_halves, turned_frame},
};
use std::{fs, path::Path};

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::InvalidData(error.to_string())
}

/// Step 1: the master as 16-bit RGB samples.
fn decode(master: &Path) -> Result<(u32, u32, Vec<u16>)> {
    let mut reader = image::ImageReader::open(master)?
        .with_guessed_format()
        .map_err(invalid)?;
    reader.no_limits();
    match reader.decode().map_err(invalid)? {
        image::DynamicImage::ImageRgb16(buffer) => {
            let (w, h) = buffer.dimensions();
            Ok((w, h, buffer.into_raw()))
        }
        image::DynamicImage::ImageRgb8(buffer) => {
            let (w, h) = buffer.dimensions();
            Ok((
                w,
                h,
                buffer
                    .into_raw()
                    .into_iter()
                    .map(|v| u16::from(v) * 257)
                    .collect(),
            ))
        }
        other => Err(Error::InvalidData(format!(
            "master is not RGB 8/16-bit: {:?}",
            other.color()
        ))),
    }
}

/// Steps 3: the turned, flipped frame as an index remap over the source samples.
struct Turned<'a> {
    data: &'a [u16],
    w0: usize,
    h0: usize,
    wt: usize,
    ht: usize,
    turns: u8,
    flip: bool,
}
impl Turned<'_> {
    /// The source pixel shown at turned-and-flipped `(u, v)`.
    fn source(&self, u: usize, v: usize) -> usize {
        let x1 = if self.flip { self.wt - 1 - u } else { u };
        let y1 = v;
        let (x, y) = match self.turns % 4 {
            0 => (x1, y1),
            1 => (y1, self.h0 - 1 - x1),
            2 => (self.w0 - 1 - x1, self.h0 - 1 - y1),
            _ => (self.w0 - 1 - y1, x1),
        };
        (y * self.w0 + x) * 3
    }
    fn pixel(&self, u: isize, v: isize) -> [f32; 3] {
        let u = u.clamp(0, self.wt as isize - 1) as usize;
        let v = v.clamp(0, self.ht as isize - 1) as usize;
        let i = self.source(u, v);
        [
            f32::from(self.data[i]),
            f32::from(self.data[i + 1]),
            f32::from(self.data[i + 2]),
        ]
    }
    /// Bicubic (Catmull-Rom) sample at continuous turned coordinates (pixel centres at `n + 0.5`).
    fn bicubic(&self, sx: f64, sy: f64) -> [u16; 3] {
        let fx = sx - 0.5;
        let fy = sy - 0.5;
        let x0 = fx.floor();
        let y0 = fy.floor();
        let tx = (fx - x0) as f32;
        let ty = (fy - y0) as f32;
        let weights = |t: f32| -> [f32; 4] {
            let a = -0.5f32;
            let w = |d: f32| {
                let d = d.abs();
                if d <= 1.0 {
                    (a + 2.0) * d * d * d - (a + 3.0) * d * d + 1.0
                } else if d < 2.0 {
                    a * d * d * d - 5.0 * a * d * d + 8.0 * a * d - 4.0 * a
                } else {
                    0.0
                }
            };
            [w(1.0 + t), w(t), w(1.0 - t), w(2.0 - t)]
        };
        let (wx, wy) = (weights(tx), weights(ty));
        let mut sum = [0f32; 3];
        for (j, wyj) in wy.iter().enumerate() {
            for (i, wxi) in wx.iter().enumerate() {
                let p = self.pixel(x0 as isize + i as isize - 1, y0 as isize + j as isize - 1);
                let w = wxi * wyj;
                for c in 0..3 {
                    sum[c] += p[c] * w;
                }
            }
        }
        sum.map(|v| v.round().clamp(0.0, 65535.0) as u16)
    }
}

/// Step 4: the slot's pixel rect, kept inside the turned frame.
fn slot_rect(values: &CompositionValues, slot: OutputSlot, turned: Frame) -> Result<PixelRect> {
    let pixel = pixel_rect(values.rect, turned);
    let rect = match (values.kind, slot) {
        (CompositionKind::Split, OutputSlot::Left) => split_halves(pixel).0,
        (CompositionKind::Split, OutputSlot::Right) => split_halves(pixel).1,
        _ => pixel,
    };
    let left = rect.left.clamp(0, turned.w);
    let top = rect.top.clamp(0, turned.h);
    let width = rect.width.min(turned.w - left);
    let height = rect.height.min(turned.h - top);
    if width <= 0 || height <= 0 {
        return Err(Error::InvalidData("empty pixel rect".into()));
    }
    Ok(PixelRect {
        left,
        top,
        width,
        height,
    })
}

pub fn render_output(
    master: &Path,
    values: &CompositionValues,
    slot: OutputSlot,
    format: &OutputFormat,
    dest: &Path,
) -> Result<OutputInfo> {
    // Step 8 (read first, so the EXIF buffer is released before the pixels are decoded).
    let meta = metadata::read_master(master);
    // Steps 1–2.
    let (w0, h0, mut data) = decode(master)?;
    let to_linear = linear::srgb16_to_linear16();
    for v in &mut data {
        *v = to_linear[usize::from(*v)];
    }
    let frame = Frame {
        w: i64::from(w0),
        h: i64::from(h0),
    };
    // Step 3.
    let turned_size = turned_frame(frame, values.quarter_turns);
    let turned = Turned {
        data: &data,
        w0: w0 as usize,
        h0: h0 as usize,
        wt: turned_size.w as usize,
        ht: turned_size.h as usize,
        turns: values.quarter_turns,
        flip: values.flip == Flip::Horizontal,
    };
    // Step 4.
    let rect = slot_rect(values, slot, turned_size)?;
    let (cw, ch) = (rect.width as usize, rect.height as usize);
    // Step 5.
    let mut cut = vec![0u16; cw * ch * 3];
    if values.straighten == 0.0 {
        for j in 0..ch {
            for i in 0..cw {
                let s = turned.source(rect.left as usize + i, rect.top as usize + j);
                let o = (j * cw + i) * 3;
                cut[o..o + 3].copy_from_slice(&data[s..s + 3]);
            }
        }
    } else {
        let (sin, cos) = values.straighten.to_radians().sin_cos();
        let cx = turned_size.w as f64 / 2.0;
        let cy = turned_size.h as f64 / 2.0;
        for j in 0..ch {
            let py = rect.top as f64 + j as f64 + 0.5;
            for i in 0..cw {
                let px = rect.left as f64 + i as f64 + 0.5;
                let (dx, dy) = (px - cx, py - cy);
                let sx = cx + cos * dx + sin * dy;
                let sy = cy - sin * dx + cos * dy;
                let o = (j * cw + i) * 3;
                cut[o..o + 3].copy_from_slice(&turned.bicubic(sx, sy));
            }
        }
    }
    drop(data);
    // Step 6.
    let long_rect = cw.max(ch);
    let long_edge = format.long_edge as usize;
    let (ow, oh, pixels) = if long_rect > long_edge {
        let short = ((long_edge * cw.min(ch)) as f64 / long_rect as f64).round() as usize;
        let (ow, oh) = if cw >= ch {
            (long_edge, short.max(1))
        } else {
            (short.max(1), long_edge)
        };
        let bytes: Vec<u8> = cut.iter().flat_map(|v| v.to_ne_bytes()).collect();
        drop(cut);
        let src = Image::from_vec_u8(
            cw as u32,
            ch as u32,
            bytes,
            fast_image_resize::PixelType::U16x3,
        )
        .map_err(invalid)?;
        let mut dst = Image::new(ow as u32, oh as u32, fast_image_resize::PixelType::U16x3);
        Resizer::new()
            .resize(
                &src,
                &mut dst,
                &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3)),
            )
            .map_err(invalid)?;
        let resized: Vec<u16> = dst
            .buffer()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_ne_bytes(*b))
            .collect();
        (ow, oh, resized)
    } else {
        (cw, ch, cut)
    };
    // Step 7.
    let to_srgb = linear::linear16_to_srgb8();
    let rgb: Vec<u8> = pixels.iter().map(|v| to_srgb[usize::from(*v)]).collect();
    let (width, height) = (
        u16::try_from(ow).map_err(invalid)?,
        u16::try_from(oh).map_err(invalid)?,
    );
    let app1 = metadata::exif_tiff(&meta, ow as u32, oh as u32)?;
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir)?;
    }
    let temp = dest.with_extension(format!("jpg.{}.tmp", std::process::id()));
    let mut jpeg = Vec::new();
    let mut encoder = Encoder::new(&mut jpeg, format.quality);
    encoder.set_sampling_factor(SamplingFactor::F_1_1);
    encoder
        .add_icc_profile(&metadata::srgb_profile())
        .map_err(invalid)?;
    encoder.add_exif_metadata(&app1).map_err(invalid)?;
    encoder
        .encode(&rgb, width, height, ColorType::Rgb)
        .map_err(invalid)?;
    if let Err(error) = fs::write(&temp, &jpeg) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    fs::rename(&temp, dest)?;
    Ok(OutputInfo {
        width: ow as u32,
        height: oh as u32,
        frame,
    })
}

//! sRGB ⇄ linear light (IEC 61966-2-1 transfer function) as lookup tables: 16-bit sRGB → 16-bit
//! linear for the master, 16-bit linear → 8-bit sRGB for the encode.

use std::sync::OnceLock;

fn to_linear(encoded: f64) -> f64 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}
fn to_srgb(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// 65 536 entries: a 16-bit sRGB value → its 16-bit linear-light value.
pub fn srgb16_to_linear16() -> &'static [u16] {
    static LUT: OnceLock<Vec<u16>> = OnceLock::new();
    LUT.get_or_init(|| {
        (0..=u16::MAX)
            .map(|v| (to_linear(f64::from(v) / 65535.0) * 65535.0).round() as u16)
            .collect()
    })
}

/// 65 536 entries: a 16-bit linear-light value → its 8-bit sRGB value.
pub fn linear16_to_srgb8() -> &'static [u8] {
    static LUT: OnceLock<Vec<u8>> = OnceLock::new();
    LUT.get_or_init(|| {
        (0..=u16::MAX)
            .map(|v| {
                (to_srgb(f64::from(v) / 65535.0) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            })
            .collect()
    })
}

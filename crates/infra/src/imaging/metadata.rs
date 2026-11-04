//! Output metadata (10D MC-28, render contract step 8): from the master's EXIF only `Make`,
//! `Model`, `DateTimeOriginal` (fallback `DateTime`) and `Copyright` are copied; the APP1 written
//! holds those, `Orientation = 1` and the structural tags of the allowlist — never GPS, MakerNote,
//! XMP or vendor tags. Also the sRGB ICC profile embedded in every output (built here from the
//! IEC 61966-2-1 constants, so no profile file is shipped or read).

use exif::{Field, In, Rational, Tag, Value, experimental::Writer};
use psm_core::{Error, Result};
use std::{fs::File, io::BufReader, path::Path};

/// The copied tags of a master (raw ASCII bytes, as Capture One wrote them).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MasterMeta {
    pub make: Option<Vec<u8>>,
    pub model: Option<Vec<u8>>,
    pub date_time_original: Option<Vec<u8>>,
    pub copyright: Option<Vec<u8>>,
}

/// Reads the allowlisted tags from the master (read-only). A master without EXIF, or one
/// `kamadak-exif` cannot parse, yields no tags: nothing is copied.
pub fn read_master(master: &Path) -> MasterMeta {
    let Ok(file) = File::open(master) else {
        return MasterMeta::default();
    };
    let Ok(exif) = exif::Reader::new().read_from_container(&mut BufReader::new(file)) else {
        return MasterMeta::default();
    };
    let ascii = |tag: Tag| -> Option<Vec<u8>> {
        match &exif.get_field(tag, In::PRIMARY)?.value {
            Value::Ascii(parts) => parts.first().filter(|p| !p.is_empty()).cloned(),
            _ => None,
        }
    };
    MasterMeta {
        make: ascii(Tag::Make),
        model: ascii(Tag::Model),
        date_time_original: ascii(Tag::DateTimeOriginal).or_else(|| ascii(Tag::DateTime)),
        copyright: ascii(Tag::Copyright),
    }
}

fn field(tag: Tag, value: Value) -> Field {
    Field {
        tag,
        ifd_num: In::PRIMARY,
        value,
    }
}

/// The TIFF structure of the output's EXIF APP1 (the `Exif\0\0` header is added by the encoder).
pub fn exif_tiff(meta: &MasterMeta, width: u32, height: u32) -> Result<Vec<u8>> {
    let resolution = || Value::Rational(vec![Rational { num: 72, denom: 1 }]);
    let mut fields = vec![
        field(Tag::Orientation, Value::Short(vec![1])),
        field(Tag::XResolution, resolution()),
        field(Tag::YResolution, resolution()),
        field(Tag::ResolutionUnit, Value::Short(vec![2])),
        field(Tag::YCbCrPositioning, Value::Short(vec![1])),
        field(Tag::ExifVersion, Value::Undefined(b"0230".to_vec(), 0)),
        field(Tag::ColorSpace, Value::Short(vec![1])),
        field(Tag::PixelXDimension, Value::Long(vec![width])),
        field(Tag::PixelYDimension, Value::Long(vec![height])),
    ];
    for (tag, value) in [
        (Tag::Make, &meta.make),
        (Tag::Model, &meta.model),
        (Tag::DateTimeOriginal, &meta.date_time_original),
        (Tag::Copyright, &meta.copyright),
    ] {
        if let Some(bytes) = value {
            fields.push(field(tag, Value::Ascii(vec![bytes.clone()])));
        }
    }
    let mut writer = Writer::new();
    for f in &fields {
        writer.push_field(f);
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    writer
        .write(&mut buf, false)
        .map_err(|e| Error::InvalidData(e.to_string()))?;
    Ok(buf.into_inner())
}

fn s15(v: f64) -> [u8; 4] {
    ((v * 65536.0).round() as i32).to_be_bytes()
}
fn xyz(values: [f64; 3]) -> Vec<u8> {
    let mut out = b"XYZ \0\0\0\0".to_vec();
    for v in values {
        out.extend_from_slice(&s15(v));
    }
    out
}
fn mluc(text: &str) -> Vec<u8> {
    let utf16: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
    let mut out = b"mluc\0\0\0\0".to_vec();
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&12u32.to_be_bytes());
    out.extend_from_slice(b"enUS");
    out.extend_from_slice(&(utf16.len() as u32).to_be_bytes());
    out.extend_from_slice(&28u32.to_be_bytes());
    out.extend_from_slice(&utf16);
    out
}

/// An ICC v4.3 display profile for sRGB IEC61966-2.1: D50 PCS, Bradford-adapted colorants, the
/// exact sRGB transfer function as a parametric curve (type 3).
pub fn srgb_profile() -> Vec<u8> {
    let mut trc = b"para\0\0\0\0".to_vec();
    trc.extend_from_slice(&3u16.to_be_bytes());
    trc.extend_from_slice(&[0, 0]);
    for v in [2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045] {
        trc.extend_from_slice(&s15(v));
    }
    let mut chad = b"sf32\0\0\0\0".to_vec();
    for v in [
        1.047_811_2,
        0.022_886_6,
        -0.050_127_0,
        0.029_542_4,
        0.990_484_4,
        -0.017_049_1,
        -0.009_234_5,
        0.015_043_6,
        0.752_131_6,
    ] {
        chad.extend_from_slice(&s15(v));
    }
    let tags: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"desc", mluc("sRGB IEC61966-2.1")),
        (b"cprt", mluc("No copyright, use freely")),
        (b"wtpt", xyz([0.9642, 1.0, 0.8249])),
        (b"chad", chad),
        (b"rXYZ", xyz([0.436_066, 0.222_488, 0.013_916])),
        (b"gXYZ", xyz([0.385_147, 0.716_873, 0.097_076])),
        (b"bXYZ", xyz([0.143_066, 0.060_608, 0.714_096])),
        (b"rTRC", trc.clone()),
        (b"gTRC", trc.clone()),
        (b"bTRC", trc),
    ];
    let table_len = 4 + 12 * tags.len();
    let mut data = Vec::new();
    let mut entries = Vec::new();
    for (sig, bytes) in &tags {
        while data.len() % 4 != 0 {
            data.push(0);
        }
        entries.push((*sig, 128 + table_len + data.len(), bytes.len()));
        data.extend_from_slice(bytes);
    }
    while data.len() % 4 != 0 {
        data.push(0);
    }
    let size = 128 + table_len + data.len();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&(size as u32).to_be_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&0x0430_0000u32.to_be_bytes());
    out.extend_from_slice(b"mntrRGB XYZ ");
    for v in [2026u16, 11, 4, 0, 0, 0] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    out.extend_from_slice(b"acspAPPL");
    // flags, manufacturer, model, attributes, rendering intent (bytes 44..68), then the D50
    // PCS illuminant at 68.
    out.extend_from_slice(&[0; 24]);
    out.extend_from_slice(&s15(0.9642));
    out.extend_from_slice(&s15(1.0));
    out.extend_from_slice(&s15(0.8249));
    out.resize(128, 0);
    out.extend_from_slice(&(tags.len() as u32).to_be_bytes());
    for (sig, offset, len) in entries {
        out.extend_from_slice(sig);
        out.extend_from_slice(&(offset as u32).to_be_bytes());
        out.extend_from_slice(&(len as u32).to_be_bytes());
    }
    out.extend_from_slice(&data);
    out
}
